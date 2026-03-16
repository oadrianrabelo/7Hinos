using Microsoft.EntityFrameworkCore;
using SevenHinos.Data;
using SevenHinos.Models;

namespace SevenHinos.Services;

public class EfSongService(IDbContextFactory<AppDbContext> factory) : ISongService
{
    public async Task<IEnumerable<Song>> GetAllAsync()
    {
        await using var db = await factory.CreateDbContextAsync();
        return await db.Songs.OrderBy(s => s.Title).ToListAsync();
    }

    public async Task<IEnumerable<Song>> SearchAsync(string query)
    {
        if (string.IsNullOrWhiteSpace(query))
            return await GetAllAsync();

        await using var db = await factory.CreateDbContextAsync();
        var lower = query.ToLower();
        return await db.Songs
            .Where(s => s.Title.ToLower().Contains(lower) ||
                        s.Album.ToLower().Contains(lower))
            .OrderBy(s => s.Title)
            .ToListAsync();
    }

    public async Task<Song?> GetWithSlidesAsync(int id)
    {
        await using var db = await factory.CreateDbContextAsync();
        return await db.Songs
            .Include(s => s.Slides.OrderBy(sl => sl.Order))
            .FirstOrDefaultAsync(s => s.Id == id);
    }

    public async Task SaveWithSlidesAsync(Song song, IReadOnlyList<string> slideTexts)
    {
        await using var db = await factory.CreateDbContextAsync();

        Song entity;
        if (song.Id == 0)
        {
            entity = new Song { Title = song.Title, Album = song.Album, YoutubeUrl = song.YoutubeUrl };
            db.Songs.Add(entity);
        }
        else
        {
            entity = await db.Songs.Include(s => s.Slides).FirstAsync(s => s.Id == song.Id);
            entity.Title      = song.Title;
            entity.Album      = song.Album;
            entity.YoutubeUrl = song.YoutubeUrl;
            db.SongSlides.RemoveRange(entity.Slides);
            entity.Slides.Clear();
        }

        for (int i = 0; i < slideTexts.Count; i++)
        {
            var text = slideTexts[i].Trim();
            if (string.IsNullOrEmpty(text)) continue;
            entity.Slides.Add(new SongSlide { Order = i, Content = text, ShowSlide = true });
        }

        entity.Lyrics = string.Join("\n\n", entity.Slides.Select(s => s.Content));

        await db.SaveChangesAsync();
        song.Id = entity.Id;
    }

    public async Task<Song> AddAsync(Song song)
    {
        await using var db = await factory.CreateDbContextAsync();
        db.Songs.Add(song);
        await db.SaveChangesAsync();
        return song;
    }

    public async Task UpdateAsync(Song song)
    {
        await using var db = await factory.CreateDbContextAsync();
        db.Songs.Update(song);
        await db.SaveChangesAsync();
    }

    public async Task DeleteAsync(int id)
    {
        await using var db = await factory.CreateDbContextAsync();
        await db.Songs.Where(s => s.Id == id).ExecuteDeleteAsync();
    }

    public async Task ToggleFavoriteAsync(int id)
    {
        await using var db = await factory.CreateDbContextAsync();
        await db.Songs
            .Where(s => s.Id == id)
            .ExecuteUpdateAsync(s => s.SetProperty(x => x.IsFavorite, x => !x.IsFavorite));
    }

    public async Task<IEnumerable<Song>> GetFavoritesAsync()
    {
        await using var db = await factory.CreateDbContextAsync();
        return await db.Songs.Where(s => s.IsFavorite).OrderBy(s => s.Title).ToListAsync();
    }
}
