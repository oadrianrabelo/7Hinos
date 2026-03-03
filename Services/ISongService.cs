using SevenHinos.Models;

namespace SevenHinos.Services;

public interface ISongService
{
    Task<IEnumerable<Song>> GetAllAsync();
    Task<IEnumerable<Song>> SearchAsync(string query);
    Task<Song> AddAsync(Song song);
    Task UpdateAsync(Song song);
    Task DeleteAsync(int id);
    Task ToggleFavoriteAsync(int id);
    Task<IEnumerable<Song>> GetFavoritesAsync();
}
