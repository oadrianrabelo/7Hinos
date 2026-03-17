using System.Text.Json;
using Microsoft.EntityFrameworkCore;
using SevenHinos.Data;
using SevenHinos.Models;

namespace SevenHinos.Services;

public sealed class NativeHymnImportService(IDbContextFactory<AppDbContext> dbFactory)
    : INativeHymnImportService
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNameCaseInsensitive = true
    };

    public string DefaultLibraryPath => Path.Combine(AppContext.BaseDirectory, "manifest.json");

    public bool IsAvailable() => File.Exists(DefaultLibraryPath);

    public async Task<ImportResult> ImportAsync(
        IProgress<ImportProgress>? progress = null,
        CancellationToken ct = default)
    {
        if (!IsAvailable())
            throw new FileNotFoundException("A biblioteca nativa não foi encontrada no pacote do aplicativo.", DefaultLibraryPath);

        await using var stream = File.OpenRead(DefaultLibraryPath);
        var manifest = await JsonSerializer.DeserializeAsync<DownloadManifest>(stream, JsonOptions, ct)
            ?? throw new InvalidDataException("O catálogo nativo do 7Hinos está vazio ou inválido.");

        var entries = BuildSongEntries(manifest.Assets, Path.GetDirectoryName(DefaultLibraryPath) ?? AppContext.BaseDirectory);
        if (entries.Count == 0)
            return new ImportResult(0, 0, 0, []);

        await using var db = await dbFactory.CreateDbContextAsync(ct);
        var existingSongs = await db.Songs
            .AsNoTracking()
            .Select(s => new { s.Title, s.Album })
            .ToListAsync(ct);

        var existingKeys = existingSongs
            .Select(s => MakeSongKey(s.Title, s.Album))
            .ToHashSet(StringComparer.OrdinalIgnoreCase);

        var errors = new List<string>();
        var imported = 0;
        var skipped = 0;
        var failed = 0;
        var done = 0;

        foreach (var entry in entries)
        {
            ct.ThrowIfCancellationRequested();
            done++;
            progress?.Report(new ImportProgress(done, entries.Count, entry.Title));

            var key = MakeSongKey(entry.Title, entry.Album);
            if (existingKeys.Contains(key))
            {
                skipped++;
                continue;
            }

            try
            {
                db.Songs.Add(new Song
                {
                    Title = entry.Title,
                    Album = entry.Album,
                    Lyrics = string.Empty,
                    AudioFilePath = entry.AudioFilePath,
                    InstrumentalFilePath = entry.InstrumentalFilePath
                });

                existingKeys.Add(key);
                imported++;

                if (imported % 200 == 0)
                    await db.SaveChangesAsync(ct);
            }
            catch (Exception ex)
            {
                failed++;
                errors.Add($"{entry.Title}: {ex.Message}");
            }
        }

        await db.SaveChangesAsync(ct);
        return new ImportResult(imported, skipped, failed, errors);
    }

    private static List<NativeSongEntry> BuildSongEntries(
        IEnumerable<ManifestAsset> assets,
        string manifestDirectory)
    {
        return assets
            .Where(asset => !string.IsNullOrWhiteSpace(asset.FileName))
            .Select(asset => MapAsset(asset, manifestDirectory))
            .Where(entry => !string.IsNullOrWhiteSpace(entry.Title))
            .GroupBy(entry => MakeSongKey(entry.Title, entry.Album), StringComparer.OrdinalIgnoreCase)
            .Select(group =>
            {
                var first = group.First();
                return new NativeSongEntry(
                    first.Title,
                    first.Album,
                    group.Select(item => item.AudioFilePath).FirstOrDefault(path => !string.IsNullOrWhiteSpace(path)),
                    group.Select(item => item.InstrumentalFilePath).FirstOrDefault(path => !string.IsNullOrWhiteSpace(path)));
            })
            .OrderBy(entry => entry.Album)
            .ThenBy(entry => entry.Title)
            .ToList();
    }

    private static NativeSongEntry MapAsset(ManifestAsset asset, string manifestDirectory)
    {
        var rawTitle = Path.GetFileNameWithoutExtension(asset.FileName).Trim();
        var isInstrumental = rawTitle.EndsWith(" - PB", StringComparison.OrdinalIgnoreCase);
        var title = isInstrumental
            ? rawTitle[..^5].TrimEnd()
            : rawTitle;

        var album = string.IsNullOrWhiteSpace(asset.Category)
            ? "Biblioteca nativa"
            : asset.Category.Trim();

        var localPath = ResolveLocalAssetPath(manifestDirectory, asset.RelativePath);
        return new NativeSongEntry(
            title,
            album,
            isInstrumental ? null : localPath,
            isInstrumental ? localPath : null);
    }

    private static string? ResolveLocalAssetPath(string manifestDirectory, string relativePath)
    {
        if (string.IsNullOrWhiteSpace(relativePath))
            return null;

        var normalizedRelative = relativePath
            .Replace('/', Path.DirectorySeparatorChar)
            .TrimStart(Path.DirectorySeparatorChar);

        var roots = new[]
        {
            Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
                "7Hinos",
                "assets"),
            Path.Combine(manifestDirectory, "assets"),
            manifestDirectory
        };

        foreach (var root in roots.Distinct(StringComparer.OrdinalIgnoreCase))
        {
            var candidate = Path.Combine(root, normalizedRelative);
            if (File.Exists(candidate))
                return candidate;
        }

        return null;
    }

    private static string MakeSongKey(string title, string album)
    {
        return $"{title.Trim()}::{album.Trim()}";
    }

    private sealed record NativeSongEntry(
        string Title,
        string Album,
        string? AudioFilePath,
        string? InstrumentalFilePath);
}
