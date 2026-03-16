namespace SevenHinos.Models;

/// <summary>One slide of a song (a strophe/chorus block) — used in presentation mode.</summary>
public class SongSlide
{
    public int Id { get; set; }
    public int SongId { get; set; }
    public Song Song { get; set; } = null!;

    public int Order { get; set; }
    public string Content { get; set; } = string.Empty;
    public string? AuxContent { get; set; }
    public TimeSpan Time { get; set; }
    public bool ShowSlide { get; set; } = true;
}
