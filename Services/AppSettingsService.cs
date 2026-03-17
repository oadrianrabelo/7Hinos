using Microsoft.EntityFrameworkCore;
using SevenHinos.Data;
using SevenHinos.Models;

namespace SevenHinos.Services;

public sealed class AppSettingsService(IDbContextFactory<AppDbContext> dbFactory) : IAppSettingsService
{
    public async Task<string> GetThemeAsync(CancellationToken ct = default)
    {
        await using var db = await dbFactory.CreateDbContextAsync(ct);
        var settings = await db.AppSettings.FirstOrDefaultAsync(ct);
        return settings?.Theme ?? "Dark";
    }

    public async Task SetThemeAsync(string theme, CancellationToken ct = default)
    {
        if (string.IsNullOrWhiteSpace(theme))
            throw new ArgumentException("Theme cannot be empty.", nameof(theme));

        await using var db = await dbFactory.CreateDbContextAsync(ct);
        var settings = await db.AppSettings.FirstOrDefaultAsync(ct)
            ?? new AppSettings();

        settings.Theme = theme;
        settings.UpdatedAt = DateTime.UtcNow;

        if (settings.Id == 0)
            db.AppSettings.Add(settings);

        await db.SaveChangesAsync(ct);
    }
}
