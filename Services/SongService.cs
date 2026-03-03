using SevenHinos.Models;

namespace SevenHinos.Services;

public class SongService : ISongService
{
    private readonly List<Song> _songs =
    [
        new Song { Title = "Santo, Santo, Santo",    Artist = "Hino Clássico",   IsFavorite = true,  Lyrics  = "Santo, Santo, Santo\nSenhor Deus todo-poderoso\nTu és digno de louvor" },
        new Song { Title = "Castelo Forte",          Artist = "Martinho Lutero", IsFavorite = false, Lyrics  = "Castelo forte é nosso Deus\nBom escudo e bom escudo" },
        new Song { Title = "Grande é o Senhor",      Artist = "Hino Evangélico", IsFavorite = true,  Lyrics  = "Grande é o Senhor e mui digno de louvor" },
        new Song { Title = "Eu Me Rendo",            Artist = "Judson Van DeVenter",IsFavorite = false,Lyrics = "Eu me rendo, eu me rendo\nA teus pés me prostro, ó Senhor" },
        new Song { Title = "Quão Grande És Tu",      Artist = "Stuart K. Hine",  IsFavorite = true,  Lyrics  = "Senhor meu Deus, quando eu maravilhado\nContemple os mundos que criastes" },
        new Song { Title = "Graça Sublime É",        Artist = "John Newton",     IsFavorite = false, Lyrics  = "Graça sublime é, que a mim salvou\nCom ela recebi a luz" },
    ];

    public Task<IEnumerable<Song>> GetAllAsync() =>
        Task.FromResult<IEnumerable<Song>>(_songs);

    public Task<IEnumerable<Song>> SearchAsync(string query) =>
        Task.FromResult<IEnumerable<Song>>(
            string.IsNullOrWhiteSpace(query)
                ? _songs
                : _songs.Where(s =>
                    s.Title.Contains(query, StringComparison.OrdinalIgnoreCase) ||
                    s.Artist.Contains(query, StringComparison.OrdinalIgnoreCase)));

    public Task<Song> AddAsync(Song song)
    {
        _songs.Add(song);
        return Task.FromResult(song);
    }

    public Task UpdateAsync(Song song)
    {
        var i = _songs.FindIndex(s => s.Id == song.Id);
        if (i >= 0) _songs[i] = song;
        return Task.CompletedTask;
    }

    public Task DeleteAsync(Guid id)
    {
        _songs.RemoveAll(s => s.Id == id);
        return Task.CompletedTask;
    }

    public Task ToggleFavoriteAsync(Guid id)
    {
        var song = _songs.FirstOrDefault(s => s.Id == id);
        if (song is not null) song.IsFavorite = !song.IsFavorite;
        return Task.CompletedTask;
    }

    public Task<IEnumerable<Song>> GetFavoritesAsync() =>
        Task.FromResult<IEnumerable<Song>>(_songs.Where(s => s.IsFavorite));
}
