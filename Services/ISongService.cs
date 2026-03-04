using SevenHinos.Models;

namespace SevenHinos.Services;

public interface ISongService
{
    Task<IEnumerable<Song>> GetAllAsync();
    Task<IEnumerable<Song>> SearchAsync(string query);
    Task<Song?> GetWithSlidesAsync(int id);
    Task<Song> AddAsync(Song song);
    Task UpdateAsync(Song song);
    Task SaveWithSlidesAsync(Song song, IReadOnlyList<string> slideTexts);
    Task DeleteAsync(int id);
    Task ToggleFavoriteAsync(int id);
    Task<IEnumerable<Song>> GetFavoritesAsync();
}
