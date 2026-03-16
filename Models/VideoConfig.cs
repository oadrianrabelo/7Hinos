using System.ComponentModel.DataAnnotations;

namespace SevenHinos.Models;

public sealed class VideoConfig
{
    [Key]
    public int Id { get; set; }

    [Required]
    public string FilePath { get; set; } = string.Empty;

    [Required]
    public string VideoName { get; set; } = string.Empty;

    /// <summary>Order used for manual organization inside the selected category.</summary>
    public int DisplayOrder { get; set; }

    public int? CategoryId { get; set; }
    public VideoCategory? Category { get; set; }

    public DateTime CreatedAt { get; set; } = DateTime.UtcNow;

    public ICollection<VideoMonitorTarget> MonitorTargets { get; set; } = [];
}
