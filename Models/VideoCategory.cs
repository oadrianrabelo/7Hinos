using System.ComponentModel.DataAnnotations;

namespace SevenHinos.Models;

public sealed class VideoCategory
{
    [Key]
    public int Id { get; set; }

    [Required]
    public string Name { get; set; } = string.Empty;

    /// <summary>
    /// Comma-separated monitor indices used as a quick preset for videos
    /// in this category (example: "0,2").
    /// </summary>
    [Required]
    public string MonitorPreset { get; set; } = string.Empty;

    public DateTime CreatedAt { get; set; } = DateTime.UtcNow;

    public ICollection<VideoConfig> Videos { get; set; } = [];
}
