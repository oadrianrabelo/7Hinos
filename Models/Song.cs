using CommunityToolkit.Mvvm.ComponentModel;

namespace SevenHinos.Models;

public partial class Song : ObservableObject
{
    public int Id { get; set; }
    public string Title { get; set; } = string.Empty;
    public string Album { get; set; } = string.Empty;

    /// <summary>Full lyrics text (concatenated from slides) — used for display.</summary>
    public string Lyrics { get; set; } = string.Empty;

    [ObservableProperty]
    private bool _isFavorite;

    public string? YoutubeUrl { get; set; }
    public string? AudioFilePath { get; set; }
    public string? InstrumentalFilePath { get; set; }
    public DateTime CreatedAt { get; set; } = DateTime.UtcNow;

    public ICollection<SongSlide> Slides { get; set; } = [];
}
