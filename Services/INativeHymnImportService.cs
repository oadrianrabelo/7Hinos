namespace SevenHinos.Services;

public interface INativeHymnImportService
{
    string DefaultLibraryPath { get; }

    bool IsAvailable();

    Task<ImportResult> ImportAsync(
        IProgress<ImportProgress>? progress = null,
        CancellationToken ct = default);
}