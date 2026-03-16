namespace SevenHinos.Services;

public record ImportProgress(int Done, int Total, string CurrentSong);

public record ImportResult(int Imported, int Skipped, int Failed, List<string> Errors);

public interface ILouvorJaImportService
{
    /// <summary>Default path of the LouvorJA SQLite database.</summary>
    static string DefaultDbPath =>
        @"C:\Program Files (x86)\Louvor JA\config\database.db";

    /// <summary>Default folder where the MP3s are stored locally.</summary>
    static string DefaultMusicasFolder =>
        @"C:\Program Files (x86)\Louvor JA\config\musicas";

    /// <returns>true if the LouvorJA database file exists at the given path.</returns>
    bool IsAvailable(string dbPath);

    /// <summary>
    /// Imports all songs from the LouvorJA SQLite database into the 7Hinos database.
    /// Already-imported songs (same Title + Album) are skipped.
    /// </summary>
    Task<ImportResult> ImportAsync(
        string dbPath,
        string musicasFolder,
        IProgress<ImportProgress>? progress = null,
        CancellationToken ct = default);
}
