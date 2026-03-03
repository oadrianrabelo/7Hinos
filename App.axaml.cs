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

    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    public override void OnFrameworkInitializationCompleted()
    {
        _services = BuildServices();

        // Ensure DB schema is created on first launch
        using var scope = _services.CreateScope();
        scope.ServiceProvider.GetRequiredService<AppDbContext>().Database.EnsureCreated();

        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            DisableAvaloniaDataAnnotationValidation();
            desktop.MainWindow = new MainWindow
            {
                DataContext = _services.GetRequiredService<MainWindowViewModel>(),
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

        // ViewModels
        services.AddSingleton<PlayerViewModel>();
        services.AddTransient<SongListViewModel>();
        services.AddTransient<FileValidationViewModel>();
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
}
