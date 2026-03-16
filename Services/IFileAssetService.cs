using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using SevenHinos.Models;

namespace SevenHinos.Services;

/// <summary>
/// Manages remote manifest fetching, local file validation, and downloads.
/// </summary>
public interface IFileAssetService
{
    /// <summary>Fetches and deserializes the JSON manifest from <paramref name="manifestUrl"/>.</summary>
    Task<DownloadManifest> FetchManifestAsync(string manifestUrl, CancellationToken ct = default);

    /// <summary>
    /// Merges manifest entries into the local database.
    /// New entries are inserted; existing ones are updated if the manifest version changed.
    /// </summary>
    Task SyncManifestAsync(DownloadManifest manifest, CancellationToken ct = default);

    /// <summary>
    /// Returns all <see cref="FileAsset"/> records from the local database.
    /// </summary>
    Task<IReadOnlyList<FileAsset>> GetAllAssetsAsync(CancellationToken ct = default);

    /// <summary>
    /// Verifies each asset on disk: checks existence, size, and SHA-256 hash.
    /// Updates <see cref="FileAsset.IsVerified"/> and <see cref="FileAsset.LastVerifiedAt"/> in the database.
    /// </summary>
    /// <param name="progress">Reports (currentIndex, totalCount, fileName) for each file checked.</param>
    Task ValidateLocalFilesAsync(
        IProgress<AssetValidationProgress>? progress = null,
        CancellationToken ct = default);

    /// <summary>
    /// Downloads the given assets into <paramref name="localRootPath"/>.
    /// Each file is first written to a temp file, then moved on success.
    /// </summary>
    /// <param name="assets">Subset of assets to download.</param>
    /// <param name="localRootPath">Root folder; files are placed at <c>localRootPath/asset.RelativePath</c>.</param>
    /// <param name="progress">Reports per-file and overall download progress.</param>
    Task DownloadAssetsAsync(
        IEnumerable<FileAsset> assets,
        string localRootPath,
        IProgress<AssetDownloadProgress>? progress = null,
        CancellationToken ct = default);
}

/// <summary>Progress snapshot while validating local files.</summary>
public sealed record AssetValidationProgress(int Current, int Total, string FileName, bool IsValid);

/// <summary>Progress snapshot while downloading a file.</summary>
public sealed record AssetDownloadProgress(
    int FileIndex,
    int FileCount,
    string FileName,
    long BytesReceived,
    long TotalBytes,
    bool FileCompleted,
    bool FileFailed);
