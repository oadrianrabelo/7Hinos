using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Data.Core.Plugins;
using System.Linq;
using Avalonia.Markup.Xaml;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.DependencyInjection;
using SevenHinos.Data;
using SevenHinos.Services;
using SevenHinos.ViewModels;
using SevenHinos.Views;

namespace SevenHinos;

public partial class App : Application
{
    private IServiceProvider? _services;

    /// <summary>Global service locator — only used by output windows created via <c>new T()</c>.</summary>
    public static IServiceProvider Services { get; private set; } = null!;

    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    public override void OnFrameworkInitializationCompleted()
    {
        _services = BuildServices();
        Services   = _services;

        // Ensure DB schema is created on first launch.
        // EnsureCreated() only creates tables when the DB has ZERO tables, so on databases
        // that were partially reset (e.g. by Reset-And-Reimport.ps1) EF silently skips.
        // We follow up with explicit IF-NOT-EXISTS statements for every table.
        using var scope = _services.CreateScope();
        var db = scope.ServiceProvider.GetRequiredService<AppDbContext>();
        db.Database.EnsureCreated();
        EnsureAllTablesExist(db);

        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            DisableAvaloniaDataAnnotationValidation();
            desktop.MainWindow = new MainWindow
            {
                DataContext = _services.GetRequiredService<MainWindowViewModel>(),
            };

            // Close any open output windows when the app exits (e.g. killed from terminal).
            desktop.ShutdownRequested += (_, _) =>
                _services.GetRequiredService<PresentationViewModel>().StopPresentationCommand.Execute(null);
        }

        base.OnFrameworkInitializationCompleted();
    }

    private static IServiceProvider BuildServices()
    {
        var services = new ServiceCollection();

        // Database
        var dbPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
            "7Hinos", "7hinos.db");
        Directory.CreateDirectory(Path.GetDirectoryName(dbPath)!);

        services.AddDbContext<AppDbContext>(o => o.UseSqlite($"Data Source={dbPath}"));
        services.AddDbContextFactory<AppDbContext>(o => o.UseSqlite($"Data Source={dbPath}"));

        // Infrastructure
        services.AddSingleton<ISongService, EfSongService>();
        services.AddSingleton<IAudioService, AudioService>();
        services.AddSingleton<IFileAssetService, FileAssetService>();
        services.AddSingleton<ILouvorJaImportService, LouvorJaImportService>();

        // ViewModels
        services.AddSingleton<PlayerViewModel>();
        services.AddTransient<SongListViewModel>();
        services.AddTransient<FileValidationViewModel>();
        services.AddTransient<ImportViewModel>();
        services.AddTransient<SongManagerViewModel>();
        services.AddSingleton<PresentationState>();
        services.AddSingleton<PresentationViewModel>();
        services.AddTransient<MainWindowViewModel>();

        return services.BuildServiceProvider();
    }

    private static void DisableAvaloniaDataAnnotationValidation()
    {
        var dataValidationPluginsToRemove =
            BindingPlugins.DataValidators.OfType<DataAnnotationsValidationPlugin>().ToArray();

        foreach (var plugin in dataValidationPluginsToRemove)
            BindingPlugins.DataValidators.Remove(plugin);
    }

    /// <summary>
    /// Creates any tables that EnsureCreated() may have skipped because other tables already existed.
    /// Uses raw SQL with IF NOT EXISTS so it is safe to call on every startup.
    /// </summary>
    private static void EnsureAllTablesExist(AppDbContext db)
    {
        db.Database.ExecuteSqlRaw("""
            CREATE TABLE IF NOT EXISTS "FileAssets" (
                "Id"              INTEGER NOT NULL CONSTRAINT "PK_FileAssets" PRIMARY KEY AUTOINCREMENT,
                "RelativePath"    TEXT    NOT NULL,
                "FileName"        TEXT    NOT NULL,
                "Category"        TEXT,
                "DownloadUrl"     TEXT    NOT NULL,
                "ExpectedSha256"  TEXT    NOT NULL,
                "ExpectedSize"    INTEGER NOT NULL,
                "ManifestVersion" TEXT    NOT NULL,
                "IsVerified"      INTEGER NOT NULL DEFAULT 0,
                "LocalPath"       TEXT,
                "LastVerifiedAt"  TEXT,
                "CreatedAt"       TEXT    NOT NULL
            );
            """);

        db.Database.ExecuteSqlRaw("""
            CREATE UNIQUE INDEX IF NOT EXISTS "IX_FileAssets_RelativePath"
                ON "FileAssets" ("RelativePath");
            """);

        // Additive column migrations (ALTER TABLE has no IF NOT EXISTS in SQLite, so we catch the error).
        try { db.Database.ExecuteSqlRaw(@"ALTER TABLE ""Songs"" ADD COLUMN ""DefaultPlayMode"" INTEGER NOT NULL DEFAULT 0"); }
        catch { /* column already exists — safe to ignore */ }
    }
}
