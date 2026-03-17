using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Data.Core.Plugins;
using System.Linq;
using Avalonia.Markup.Xaml;
using Avalonia.Styling;
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

            desktop.MainWindow.Opened += async (_, _) =>
            {
                if (desktop.MainWindow is null)
                    return;

                try
                {
                    // Load and apply stored theme preference
                    var appSettingsService = _services.GetRequiredService<IAppSettingsService>();
                    var theme = await appSettingsService.GetThemeAsync();
                    var mainVm = desktop.MainWindow.DataContext as MainWindowViewModel;
                    if (mainVm != null)
                    {
                        mainVm.IsDarkMode = theme == "Dark";
                        Application.Current!.RequestedThemeVariant =
                            theme == "Dark" ? ThemeVariant.Dark : ThemeVariant.Light;
                    }

                    await _services
                        .GetRequiredService<IAppUpdateService>()
                        .TryCheckAndPromptAsync(desktop.MainWindow);
                }
                catch
                {
                    // Startup must stay resilient even if update check or theme load fails.
                }
            };

            // Close any open output windows when the app exits (e.g. killed from terminal).
            desktop.ShutdownRequested += (_, _) =>
            {
                _services.GetRequiredService<PresentationViewModel>().StopPresentationCommand.Execute(null);
                _services.GetRequiredService<IVideoOutputService>().StopAll();
            };
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
        services.AddSingleton<INativeHymnImportService, NativeHymnImportService>();
        services.AddSingleton<IVideoConfigService, VideoConfigService>();
        services.AddSingleton<IVideoOutputService, VideoOutputService>();
        services.AddSingleton<IAppUpdateService, GitHubUpdateService>();
        services.AddSingleton<IAppSettingsService, AppSettingsService>();
        services.AddSingleton<IMonitorDeviceService, MonitorDeviceService>();

        // ViewModels
        services.AddSingleton<PlayerViewModel>();
        services.AddTransient<SongListViewModel>();
        services.AddTransient<FileValidationViewModel>();
        services.AddTransient<ImportViewModel>();
        services.AddSingleton<VideosViewModel>();
        services.AddTransient<SongManagerViewModel>();
        services.AddSingleton<PresentationState>();
        services.AddSingleton<PresentationViewModel>();
        services.AddSingleton<MonitorsConfigViewModel>();
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

        db.Database.ExecuteSqlRaw("""
            CREATE TABLE IF NOT EXISTS "VideoCategories" (
                "Id"        INTEGER NOT NULL CONSTRAINT "PK_VideoCategories" PRIMARY KEY AUTOINCREMENT,
                "Name"      TEXT    NOT NULL,
                "MonitorPreset" TEXT NOT NULL DEFAULT '',
                "CreatedAt" TEXT    NOT NULL
            );
            """);

        db.Database.ExecuteSqlRaw("""
            CREATE UNIQUE INDEX IF NOT EXISTS "IX_VideoCategories_Name"
                ON "VideoCategories" ("Name");
            """);

        db.Database.ExecuteSqlRaw("""
            CREATE TABLE IF NOT EXISTS "VideoConfigs" (
                "Id"         INTEGER NOT NULL CONSTRAINT "PK_VideoConfigs" PRIMARY KEY AUTOINCREMENT,
                "FilePath"   TEXT    NOT NULL,
                "VideoName"  TEXT    NOT NULL,
                "DisplayOrder" INTEGER NOT NULL DEFAULT 0,
                "CategoryId" INTEGER,
                "CreatedAt"  TEXT    NOT NULL,
                CONSTRAINT "FK_VideoConfigs_VideoCategories_CategoryId"
                    FOREIGN KEY ("CategoryId") REFERENCES "VideoCategories" ("Id") ON DELETE SET NULL
            );
            """);

        db.Database.ExecuteSqlRaw("""
            CREATE UNIQUE INDEX IF NOT EXISTS "IX_VideoConfigs_FilePath"
                ON "VideoConfigs" ("FilePath");
            """);

        db.Database.ExecuteSqlRaw("""
            CREATE INDEX IF NOT EXISTS "IX_VideoConfigs_CategoryId_DisplayOrder"
                ON "VideoConfigs" ("CategoryId", "DisplayOrder");
            """);

        db.Database.ExecuteSqlRaw("""
            CREATE TABLE IF NOT EXISTS "VideoMonitorTargets" (
                "Id"            INTEGER NOT NULL CONSTRAINT "PK_VideoMonitorTargets" PRIMARY KEY AUTOINCREMENT,
                "VideoConfigId" INTEGER NOT NULL,
                "MonitorIndex"  INTEGER NOT NULL,
                CONSTRAINT "FK_VideoMonitorTargets_VideoConfigs_VideoConfigId"
                    FOREIGN KEY ("VideoConfigId") REFERENCES "VideoConfigs" ("Id") ON DELETE CASCADE
            );
            """);

        db.Database.ExecuteSqlRaw("""
            CREATE UNIQUE INDEX IF NOT EXISTS "IX_VideoMonitorTargets_VideoConfigId_MonitorIndex"
                ON "VideoMonitorTargets" ("VideoConfigId", "MonitorIndex");
            """);

        db.Database.ExecuteSqlRaw("""
            CREATE TABLE IF NOT EXISTS "AppSettings" (
                "Id"        INTEGER NOT NULL CONSTRAINT "PK_AppSettings" PRIMARY KEY AUTOINCREMENT,
                "Theme"     TEXT    NOT NULL DEFAULT 'Dark',
                "UpdatedAt" TEXT    NOT NULL
            );
            """);

        db.Database.ExecuteSqlRaw("""
            CREATE TABLE IF NOT EXISTS "MonitorDevices" (
                "Id"           INTEGER NOT NULL CONSTRAINT "PK_MonitorDevices" PRIMARY KEY AUTOINCREMENT,
                "MonitorIndex" INTEGER NOT NULL,
                "CustomName"   TEXT    NOT NULL DEFAULT '',
                "CreatedAt"    TEXT    NOT NULL,
                "UpdatedAt"    TEXT    NOT NULL
            );
            """);

        db.Database.ExecuteSqlRaw("""
            CREATE UNIQUE INDEX IF NOT EXISTS "IX_MonitorDevices_MonitorIndex"
                ON "MonitorDevices" ("MonitorIndex");
            """);

        // Additive column migrations (ALTER TABLE has no IF NOT EXISTS in SQLite, so we catch the error).
        try { db.Database.ExecuteSqlRaw(@"ALTER TABLE ""Songs"" ADD COLUMN ""DefaultPlayMode"" INTEGER NOT NULL DEFAULT 0"); }
        catch { /* column already exists — safe to ignore */ }

        try { db.Database.ExecuteSqlRaw(@"ALTER TABLE ""VideoCategories"" ADD COLUMN ""MonitorPreset"" TEXT NOT NULL DEFAULT ''"); }
        catch { /* column already exists — safe to ignore */ }

        try { db.Database.ExecuteSqlRaw(@"ALTER TABLE ""VideoConfigs"" ADD COLUMN ""DisplayOrder"" INTEGER NOT NULL DEFAULT 0"); }
        catch { /* column already exists — safe to ignore */ }
    }
}
