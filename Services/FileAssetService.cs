using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Security.Cryptography;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using SevenHinos.Data;
using SevenHinos.Models;

namespace SevenHinos.Services;

/// <summary>
/// HTTPS-based file asset manager.
/// Improvements over legacy FTP approach:
///  • HTTPS (HttpClient) instead of anonymous FTP
///  • SHA-256 integrity check instead of file-size-only comparison
///  • Streams to temp file → atomic move on success
///  • CancellationToken throughout
///  • Exponential-backoff retry (3 attempts)
///  • No hardcoded FTP credentials — manifest URL is the only external config
/// </summary>
public sealed class FileAssetService : IFileAssetService
{
    private readonly IDbContextFactory<AppDbContext> _dbFactory;
    private readonly HttpClient _httpClient;

    // JSON deserialization options (case-insensitive for flexibility).
    private static readonly JsonSerializerOptions _jsonOptions = new()
    {
        PropertyNameCaseInsensitive = true
    };

    public FileAssetService(IDbContextFactory<AppDbContext> dbFactory)
    {
        _dbFactory = dbFactory;
        _httpClient = new HttpClient { Timeout = TimeSpan.FromSeconds(60) };
    }

    // -------------------------------------------------------------------------
    // Manifest
    // -------------------------------------------------------------------------

    public async Task<DownloadManifest> FetchManifestAsync(string manifestUrl, CancellationToken ct = default)
    {
        var json = await RetryAsync(
            () => _httpClient.GetStringAsync(manifestUrl, ct),
            maxAttempts: 3, ct: ct).ConfigureAwait(false);

        return JsonSerializer.Deserialize<DownloadManifest>(json, _jsonOptions)
               ?? throw new InvalidDataException("The manifest JSON was empty or could not be parsed.");
    }

    public async Task SyncManifestAsync(DownloadManifest manifest, CancellationToken ct = default)
    {
        await using var db = await _dbFactory.CreateDbContextAsync(ct).ConfigureAwait(false);

        foreach (var entry in manifest.Assets)
        {
            var existing = await db.FileAssets
                .FirstOrDefaultAsync(f => f.RelativePath == entry.RelativePath, ct)
                .ConfigureAwait(false);

            if (existing is null)
            {
                db.FileAssets.Add(MapToEntity(entry, manifest.Version));
            }
            else if (existing.ManifestVersion != manifest.Version)
            {
                // New manifest version → reset verification so the file gets re-checked.
                existing.DownloadUrl = entry.Url;
                existing.ExpectedSha256 = entry.Sha256;
                existing.ExpectedSize = entry.Size;
                existing.ManifestVersion = manifest.Version;
                existing.FileName = entry.FileName;
                existing.Category = entry.Category;
                existing.IsVerified = false;
                existing.LastVerifiedAt = null;
            }
        }

        await db.SaveChangesAsync(ct).ConfigureAwait(false);
    }

    // -------------------------------------------------------------------------
    // Validation
    // -------------------------------------------------------------------------

    public async Task<IReadOnlyList<FileAsset>> GetAllAssetsAsync(CancellationToken ct = default)
    {
        await using var db = await _dbFactory.CreateDbContextAsync(ct).ConfigureAwait(false);
        return await db.FileAssets.OrderBy(f => f.Category).ThenBy(f => f.FileName).ToListAsync(ct).ConfigureAwait(false);
    }

    public async Task ValidateLocalFilesAsync(
        IProgress<AssetValidationProgress>? progress = null,
        CancellationToken ct = default)
    {
        await using var db = await _dbFactory.CreateDbContextAsync(ct).ConfigureAwait(false);
        var assets = await db.FileAssets.ToListAsync(ct).ConfigureAwait(false);
        int total = assets.Count;

        for (int i = 0; i < total; i++)
        {
            ct.ThrowIfCancellationRequested();
            var asset = assets[i];
            bool valid = await VerifyLocalFileAsync(asset).ConfigureAwait(false);

            asset.IsVerified = valid;
            asset.LastVerifiedAt = valid ? DateTime.UtcNow : null;

            progress?.Report(new AssetValidationProgress(i + 1, total, asset.FileName, valid));
        }

        await db.SaveChangesAsync(ct).ConfigureAwait(false);
    }

    // -------------------------------------------------------------------------
    // Download
    // -------------------------------------------------------------------------

    public async Task DownloadAssetsAsync(
        IEnumerable<FileAsset> assets,
        string localRootPath,
        IProgress<AssetDownloadProgress>? progress = null,
        CancellationToken ct = default)
    {
        var list = assets.ToList();
        int total = list.Count;

        await using var db = await _dbFactory.CreateDbContextAsync(ct).ConfigureAwait(false);

        for (int i = 0; i < total; i++)
        {
            ct.ThrowIfCancellationRequested();
            var asset = list[i];
            var destPath = Path.Combine(localRootPath, asset.RelativePath);
            var destDir = Path.GetDirectoryName(destPath)!;
            var tempPath = destPath + ".~tmp";

            Directory.CreateDirectory(destDir);

            bool success = false;
            try
            {
                await RetryAsync(async () =>
                {
                    using var response = await _httpClient
                        .GetAsync(asset.DownloadUrl, HttpCompletionOption.ResponseHeadersRead, ct)
                        .ConfigureAwait(false);

                    response.EnsureSuccessStatusCode();

                    long totalBytes = response.Content.Headers.ContentLength
                                      ?? asset.ExpectedSize;

                    await using var source = await response.Content.ReadAsStreamAsync(ct).ConfigureAwait(false);
                    await using var dest = new FileStream(
                        tempPath, FileMode.Create, FileAccess.Write, FileShare.None, bufferSize: 81920, useAsync: true);

                    var buffer = new byte[81920];
                    long received = 0;
                    int read;

                    while ((read = await source.ReadAsync(buffer, ct).ConfigureAwait(false)) > 0)
                    {
                        ct.ThrowIfCancellationRequested();
                        await dest.WriteAsync(buffer.AsMemory(0, read), ct).ConfigureAwait(false);
                        received += read;
                        progress?.Report(new AssetDownloadProgress(
                            i + 1, total, asset.FileName,
                            received, totalBytes,
                            FileCompleted: false, FileFailed: false));
                    }
                }, maxAttempts: 3, ct: ct).ConfigureAwait(false);

                // Verify hash before committing.
                if (!string.IsNullOrEmpty(asset.ExpectedSha256))
                {
                    var actualHash = await ComputeSha256Async(tempPath, ct).ConfigureAwait(false);
                    if (!actualHash.Equals(asset.ExpectedSha256, StringComparison.OrdinalIgnoreCase))
                        throw new InvalidDataException(
                            $"SHA-256 mismatch for '{asset.FileName}'. Expected: {asset.ExpectedSha256}, Got: {actualHash}");
                }

                // Atomic move from temp to final destination.
                if (File.Exists(destPath)) File.Delete(destPath);
                File.Move(tempPath, destPath);

                success = true;
                var tracked = await db.FileAssets.FindAsync([asset.Id], ct).ConfigureAwait(false);
                if (tracked is not null)
                {
                    tracked.IsVerified = true;
                    tracked.LocalPath = destPath;
                    tracked.LastVerifiedAt = DateTime.UtcNow;
                }

                progress?.Report(new AssetDownloadProgress(
                    i + 1, total, asset.FileName,
                    asset.ExpectedSize, asset.ExpectedSize,
                    FileCompleted: true, FileFailed: false));
            }
            catch (OperationCanceledException)
            {
                CleanupTemp(tempPath);
                throw;
            }
            catch
            {
                CleanupTemp(tempPath);
                progress?.Report(new AssetDownloadProgress(
                    i + 1, total, asset.FileName,
                    0, asset.ExpectedSize,
                    FileCompleted: false, FileFailed: true));
            }
            finally
            {
                if (!success) CleanupTemp(tempPath);
            }
        }

        await db.SaveChangesAsync(ct).ConfigureAwait(false);
    }

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    private static async Task<bool> VerifyLocalFileAsync(FileAsset asset)
    {
        var localPath = asset.LocalPath;
        if (string.IsNullOrEmpty(localPath) || !File.Exists(localPath))
            return false;

        var info = new FileInfo(localPath);

        // Size check (quick).
        if (asset.ExpectedSize > 0 && info.Length < asset.ExpectedSize)
            return false;

        // Hash check (thorough).
        if (!string.IsNullOrEmpty(asset.ExpectedSha256))
        {
            var actualHash = await ComputeSha256Async(localPath).ConfigureAwait(false);
            return actualHash.Equals(asset.ExpectedSha256, StringComparison.OrdinalIgnoreCase);
        }

        return true;
    }

    private static async Task<string> ComputeSha256Async(string filePath, CancellationToken ct = default)
    {
        await using var stream = new FileStream(
            filePath, FileMode.Open, FileAccess.Read, FileShare.Read, bufferSize: 81920, useAsync: true);

        var hash = await SHA256.HashDataAsync(stream, ct).ConfigureAwait(false);
        return Convert.ToHexStringLower(hash);
    }

    private static FileAsset MapToEntity(ManifestAsset entry, string manifestVersion) => new()
    {
        RelativePath = entry.RelativePath,
        FileName = entry.FileName,
        Category = entry.Category,
        DownloadUrl = entry.Url,
        ExpectedSha256 = entry.Sha256,
        ExpectedSize = entry.Size,
        ManifestVersion = manifestVersion,
        CreatedAt = DateTime.UtcNow
    };

    /// <summary>Runs <paramref name="operation"/> up to <paramref name="maxAttempts"/> times with exponential back-off.</summary>
    private static async Task<T> RetryAsync<T>(
        Func<Task<T>> operation, int maxAttempts, CancellationToken ct)
    {
        int delay = 1000;
        for (int attempt = 1; ; attempt++)
        {
            ct.ThrowIfCancellationRequested();
            try
            {
                return await operation().ConfigureAwait(false);
            }
            catch (Exception) when (attempt < maxAttempts)
            {
                await Task.Delay(delay, ct).ConfigureAwait(false);
                delay *= 2;
            }
        }
    }

    private static async Task RetryAsync(Func<Task> operation, int maxAttempts, CancellationToken ct)
    {
        await RetryAsync<bool>(async () => { await operation().ConfigureAwait(false); return true; }, maxAttempts, ct)
            .ConfigureAwait(false);
    }

    private static void CleanupTemp(string tempPath)
    {
        try { if (File.Exists(tempPath)) File.Delete(tempPath); } catch { /* best effort */ }
    }
}
