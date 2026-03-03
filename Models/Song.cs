namespace SevenHinos.Models;

public class Song
{
    public Guid Id { get; init; } = Guid.NewGuid();
    public string Title { get; set; } = string.Empty;
    public string Artist { get; set; } = string.Empty;
    public string Lyrics { get; set; } = string.Empty;
    public bool IsFavorite { get; set; }
    public string? YoutubeUrl { get; set; }
    public string? AudioFilePath { get; set; }
    public DateTime CreatedAt { get; init; } = DateTime.UtcNow;
}
