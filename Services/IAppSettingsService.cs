namespace SevenHinos.Services;

public interface IAppSettingsService
{
    Task<string> GetThemeAsync(CancellationToken ct = default);
    Task SetThemeAsync(string theme, CancellationToken ct = default);
}
