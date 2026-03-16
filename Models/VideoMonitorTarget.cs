using System.ComponentModel.DataAnnotations;

namespace SevenHinos.Models;

public sealed class VideoMonitorTarget
{
    [Key]
    public int Id { get; set; }

    public int VideoConfigId { get; set; }
    public VideoConfig VideoConfig { get; set; } = null!;

    public int MonitorIndex { get; set; }
}
