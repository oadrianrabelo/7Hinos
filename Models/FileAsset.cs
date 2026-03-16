using System;
using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace SevenHinos.Models;

/// <summary>
/// Tracks a remote file that must be present locally for the app to work correctly.
/// The expected hash and size come from the remote manifest; the local status is persisted here.
/// </summary>
public sealed class FileAsset
{
    [Key]
    public int Id { get; set; }

    /// <summary>Relative path from the application assets root (e.g. "hinario/Castelo Forte.mp3").</summary>
    [Required]
    public string RelativePath { get; set; } = string.Empty;

    /// <summary>Display-friendly file name.</summary>
    [Required]
    public string FileName { get; set; } = string.Empty;

    /// <summary>Category / album / folder label.</summary>
    public string? Category { get; set; }

    /// <summary>Remote download URL for this file.</summary>
    [Required]
    public string DownloadUrl { get; set; } = string.Empty;

    /// <summary>Expected SHA-256 hex digest. Empty string means "unknown / skip hash check".</summary>
    [Required]
    public string ExpectedSha256 { get; set; } = string.Empty;

    /// <summary>Expected file size in bytes (0 = unknown / skip size check).</summary>
    public long ExpectedSize { get; set; }

    /// <summary>Manifest version string at the time this record was written.</summary>
    [Required]
    public string ManifestVersion { get; set; } = string.Empty;

    /// <summary>True once the file has been downloaded and verified locally.</summary>
    public bool IsVerified { get; set; }

    /// <summary>Absolute local path where the file was last saved successfully.</summary>
    public string? LocalPath { get; set; }

    /// <summary>UTC timestamp of the last successful validation.</summary>
    public DateTime? LastVerifiedAt { get; set; }

    /// <summary>UTC timestamp when this record was first inserted.</summary>
    public DateTime CreatedAt { get; set; } = DateTime.UtcNow;
}
