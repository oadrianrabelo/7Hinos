using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;
using SevenHinos.Data;
using SevenHinos.Models;

namespace SevenHinos.Services;

public sealed class LouvorJaImportService(IDbContextFactory<AppDbContext> dbFactory)
    : ILouvorJaImportService
{
    public bool IsAvailable(string dbPath) => File.Exists(dbPath);

    public async Task<ImportResult> ImportAsync(
        string dbPath,
        string musicasFolder,
        IProgress<ImportProgress>? progress = null,
        CancellationToken ct = default)
    {
        var errors   = new List<string>();
        int imported = 0, skipped = 0, failed = 0;

        // ── 1. Read everything from LouvorJA (read-only) ─────────────────────
        var ljaConStr = new SqliteConnectionStringBuilder
        {
            DataSource = dbPath,
            Mode       = SqliteOpenMode.ReadOnly
        }.ToString();

        List<LjaSong> ljaSongs;
        using (var ljaConn = new SqliteConnection(ljaConStr))
        {
            await ljaConn.OpenAsync(ct);
            var albums = await ReadAlbumsAsync(ljaConn, ct);
            ljaSongs   = await ReadSongsAsync(ljaConn, albums, musicasFolder, ct);
        }

        // ── 2. Get existing (Title, Album) pairs to skip duplicates ──────────
        await using var db = await dbFactory.CreateDbContextAsync(ct);
        var existing = await db.Songs
            .Select(s => new { s.Title, s.Album })
            .ToHashSetAsync(ct);

        // ── 3. Import in batches ─────────────────────────────────────────────
        int total = ljaSongs.Count;
        int done  = 0;

        foreach (var lja in ljaSongs)
        {
            ct.ThrowIfCancellationRequested();
            done++;
            progress?.Report(new ImportProgress(done, total, lja.Title));

            var key = new { lja.Title, lja.Album };
            if (existing.Contains(key))
            {
                skipped++;
                continue;
            }

            try
            {
                var song = new Song
                {
                    Title                  = lja.Title,
                    Album                  = lja.Album,
                    Lyrics                 = string.Join("\n\n", lja.Slides.Select(s => s.Letra)),
                    AudioFilePath          = lja.AudioFilePath,
                    InstrumentalFilePath   = lja.InstrumentalFilePath,
                    Slides                 = lja.Slides.Select((s, i) => new SongSlide
                    {
                        Order      = s.Order,
                        Content    = s.Letra,
                        AuxContent = s.LetraAux,
                        Time       = s.Tempo,
                        ShowSlide  = true,
                    }).ToList()
                };

                db.Songs.Add(song);
                imported++;
            }
            catch (Exception ex)
            {
                failed++;
                errors.Add($"{lja.Title}: {ex.Message}");
            }

            // Save every 200 songs to avoid giant change-tracker
            if (imported % 200 == 0 && imported > 0)
                await db.SaveChangesAsync(ct);
        }

        await db.SaveChangesAsync(ct);
        return new ImportResult(imported, skipped, failed, errors);
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    private static async Task<Dictionary<int, string>> ReadAlbumsAsync(
        SqliteConnection conn, CancellationToken ct)
    {
        var dict = new Dictionary<int, string>();
        await using var cmd = conn.CreateCommand();
        cmd.CommandText = "SELECT ID, NOME_COM FROM ALBUM";
        await using var r = await cmd.ExecuteReaderAsync(ct);
        while (await r.ReadAsync(ct))
        {
            var id   = r.GetInt32(0);
            var nome = r.IsDBNull(1) ? string.Empty : r.GetString(1);
            dict[id] = nome;
        }
        return dict;
    }

    private static async Task<List<LjaSong>> ReadSongsAsync(
        SqliteConnection conn,
        Dictionary<int, string> albums,
        string musicasFolder,
        CancellationToken ct)
    {
        var songs = new Dictionary<int, LjaSong>();

        // Read all MUSICAS rows
        await using (var cmd = conn.CreateCommand())
        {
            cmd.CommandText =
                "SELECT ID, NOME, ALBUM, URL, URL_INSTRUMENTAL " +
                "FROM MUSICAS ORDER BY ALBUM, NOME";

            await using var r = await cmd.ExecuteReaderAsync(ct);
            while (await r.ReadAsync(ct))
            {
                int    id      = r.GetInt32(0);
                string title   = r.IsDBNull(1) ? string.Empty : r.GetString(1);
                int    albumId = r.IsDBNull(2) ? 0            : r.GetInt32(2);
                string url     = r.IsDBNull(3) ? string.Empty : r.GetString(3);
                string urlPb   = r.IsDBNull(4) ? string.Empty : r.GetString(4);

                albums.TryGetValue(albumId, out var albumName);

                songs[id] = new LjaSong
                {
                    Title                = title,
                    Album                = albumName ?? string.Empty,
                    AudioFilePath        = ResolveLocalPath(musicasFolder, url),
                    InstrumentalFilePath = ResolveLocalPath(musicasFolder, urlPb),
                };
            }
        }

        // Read all MUSICAS_LETRA (slides) in one pass
        await using (var cmd = conn.CreateCommand())
        {
            cmd.CommandText =
                "SELECT MUSICA, ORDEM, LETRA, LETRA_AUX, TEMPO " +
                "FROM MUSICAS_LETRA ORDER BY MUSICA, ORDEM";

            await using var r = await cmd.ExecuteReaderAsync(ct);
            while (await r.ReadAsync(ct))
            {
                int    musicaId = r.GetInt32(0);
                int    ordem    = r.IsDBNull(1) ? 0            : r.GetInt32(1);
                string letra    = r.IsDBNull(2) ? string.Empty : r.GetString(2);
                string letraAux = r.IsDBNull(3) ? string.Empty : r.GetString(3);
                // TEMPO stored as milliseconds integer in LouvorJA
                var tempo = r.IsDBNull(4)
                    ? TimeSpan.Zero
                    : TimeSpan.FromMilliseconds(r.GetInt64(4));

                if (!songs.TryGetValue(musicaId, out var song)) continue;

                song.Slides.Add(new LjaSlide
                {
                    Order    = ordem,
                    Letra    = letra,
                    LetraAux = letraAux,
                    Tempo    = tempo,
                });
            }
        }

        return [.. songs.Values];
    }

    private static string? ResolveLocalPath(string musicasFolder, string url)
    {
        if (string.IsNullOrWhiteSpace(url)) return null;
        var path = Path.Combine(musicasFolder, url.Replace('/', Path.DirectorySeparatorChar));
        return File.Exists(path) ? path : null;
    }

    // ── Private data transfer objects ─────────────────────────────────────────

    private sealed class LjaSong
    {
        public string  Title                { get; set; } = string.Empty;
        public string  Album                { get; set; } = string.Empty;
        public string? AudioFilePath        { get; set; }
        public string? InstrumentalFilePath { get; set; }
        public List<LjaSlide> Slides        { get; } = [];
    }

    private sealed class LjaSlide
    {
        public int      Order    { get; set; }
        public string   Letra    { get; set; } = string.Empty;
        public string   LetraAux { get; set; } = string.Empty;
        public TimeSpan Tempo    { get; set; }
    }
}
