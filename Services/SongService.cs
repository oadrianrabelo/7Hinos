using SevenHinos.Models;

namespace SevenHinos.Services;

/// <summary>Fallback in-memory service used only for design-time / tests.</summary>
public class SongService : ISongService
{
    private readonly List<Song> _songs = [];

    public Task<IEnumerable<Song>> GetAllAsync() =>
        Task.FromResult<IEnumerable<Song>>(_songs);

    public Task<IEnumerable<Song>> SearchAsync(string query) =>
        Task.FromResult<IEnumerable<Song>>(
            string.IsNullOrWhiteSpace(query) ? _songs
                : _songs.Where(s => s.Title.Contains(query, StringComparison.OrdinalIgnoreCase)));

    public Task<Song> AddAsync(Song song) { _songs.Add(song); return Task.FromResult(song); }

    public Task<Song?> GetWithSlidesAsync(int id) =>
        Task.FromResult<Song?>(_songs.FirstOrDefault(s => s.Id == id));

    public Task SaveWithSlidesAsync(Song song, IReadOnlyList<string> slideTexts)
    {
        song.Lyrics = string.Join("\n\n", slideTexts);
        song.Slides = slideTexts.Select((t, i) =>
            new SongSlide { Order = i, Content = t, ShowSlide = true }).ToList();
        if (song.Id == 0) _songs.Add(song);
        else { var i = _songs.FindIndex(s => s.Id == song.Id); if (i >= 0) _songs[i] = song; }
        return Task.CompletedTask;
    }

    public Task UpdateAsync(Song song)
    {
        var i = _songs.FindIndex(s => s.Id == song.Id);
        if (i >= 0) _songs[i] = song;
        return Task.CompletedTask;
    }

    public Task DeleteAsync(int id) { _songs.RemoveAll(s => s.Id == id); return Task.CompletedTask; }

    public Task ToggleFavoriteAsync(int id)
    {
        var song = _songs.FirstOrDefault(s => s.Id == id);
        if (song is not null) song.IsFavorite = !song.IsFavorite;
        return Task.CompletedTask;
    }

    public Task<IEnumerable<Song>> GetFavoritesAsync() =>
        Task.FromResult<IEnumerable<Song>>(_songs.Where(s => s.IsFavorite));
}
