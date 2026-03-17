using System.ComponentModel.DataAnnotations;

namespace SevenHinos.Models;

public sealed class AppSettings
{
    [Key]
    public int Id { get; set; }

    /// <summary>
    /// Stored theme preference: "Dark" or "Light"
    /// </summary>
    [Required]
    public string Theme { get; set; } = "Dark";

    public DateTime UpdatedAt { get; set; } = DateTime.UtcNow;
}
