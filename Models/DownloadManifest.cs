using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace SevenHinos.Models;

/// <summary>
/// Root object of the remote JSON manifest file.
/// Example URL: https://example.com/7hinos/manifest.json
/// </summary>
public sealed class DownloadManifest
{
    /// <summary>Semver string, e.g. "1.0.0".</summary>
    [JsonPropertyName("version")]
    public string Version { get; set; } = string.Empty;

    /// <summary>Human-readable release notes (optional).</summary>
    [JsonPropertyName("releaseNotes")]
    public string? ReleaseNotes { get; set; }

    /// <summary>List of files in this manifest.</summary>
    [JsonPropertyName("assets")]
    public List<ManifestAsset> Assets { get; set; } = [];
}

/// <summary>
/// A single file entry inside <see cref="DownloadManifest"/>.
/// </summary>
public sealed class ManifestAsset
{
    /// <summary>Relative path, e.g. "hinario/Castelo Forte.mp3".</summary>
    [JsonPropertyName("relativePath")]
    public string RelativePath { get; set; } = string.Empty;

    /// <summary>Display file name.</summary>
    [JsonPropertyName("fileName")]
    public string FileName { get; set; } = string.Empty;

    /// <summary>Category / album / folder label (optional).</summary>
    [JsonPropertyName("category")]
    public string? Category { get; set; }

    /// <summary>HTTPS URL to download this file.</summary>
    [JsonPropertyName("url")]
    public string Url { get; set; } = string.Empty;

    /// <summary>Lowercase hex SHA-256 digest. Empty string = skip hash verification.</summary>
    [JsonPropertyName("sha256")]
    public string Sha256 { get; set; } = string.Empty;

    /// <summary>File size in bytes (0 = unknown).</summary>
    [JsonPropertyName("size")]
    public long Size { get; set; }
}
