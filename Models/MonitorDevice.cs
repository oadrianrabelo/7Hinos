using System.ComponentModel.DataAnnotations;

namespace SevenHinos.Models;

/// <summary>
/// Stores user-friendly names and configurations for detected monitors.
/// MonitorIndex corresponds to Screen index from Avalonia.
/// </summary>
public sealed class MonitorDevice
{
    [Key]
    public int Id { get; set; }

    /// <summary>
    /// Monitor index from Screen.All enumeration (0-based).
    /// This is the unique identifier across sessions.
    /// </summary>
    public int MonitorIndex { get; set; }

    /// <summary>
    /// User-friendly/custom name for the monitor (e.g., "Projetor", "TV Lateral").
    /// If empty, displays as "Tela {MonitorIndex + 1}".
    /// </summary>
    [Required]
    public string CustomName { get; set; } = string.Empty;

    public DateTime CreatedAt { get; set; } = DateTime.UtcNow;
    public DateTime UpdatedAt { get; set; } = DateTime.UtcNow;
}
