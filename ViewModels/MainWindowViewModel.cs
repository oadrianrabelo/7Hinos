using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Styling;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SevenHinos.Services;
using System.Reflection;

namespace SevenHinos.ViewModels;

public partial class MainWindowViewModel : ViewModelBase
{
    private readonly IAppSettingsService _appSettingsService;

    public PlayerViewModel         Player         { get; }
    public FileValidationViewModel FileValidation { get; }
    public VideosViewModel         Videos         { get; }
    public SongManagerViewModel    SongManager    { get; }
    public string                  AppVersion     { get; } = ResolveAppVersion();

    [ObservableProperty] private ViewModelBase _currentPage;
    [ObservableProperty] private bool _isDarkMode = true;

    /// <summary>Raised when the user clicks "Apresentação". View opens the PresentationWindow.</summary>
    public event Action? OpenPresentationRequested;

    public MainWindowViewModel(
        PlayerViewModel player,
        FileValidationViewModel fileValidation,
        VideosViewModel videos,
        SongManagerViewModel songManager,
        IAppSettingsService appSettingsService)
    {
        Player         = player;
        FileValidation = fileValidation;
        Videos         = videos;
        SongManager    = songManager;
        _appSettingsService = appSettingsService;
        _currentPage   = SongManager;
    }

    [RelayCommand]
    private void Navigate(ViewModelBase page) => CurrentPage = page;

    [RelayCommand]
    private void OpenPresentation() => OpenPresentationRequested?.Invoke();

    [RelayCommand]
    private async void ToggleTheme()
    {
        IsDarkMode = !IsDarkMode;
        Application.Current!.RequestedThemeVariant =
            IsDarkMode ? ThemeVariant.Dark : ThemeVariant.Light;

        try
        {
            await _appSettingsService.SetThemeAsync(IsDarkMode ? "Dark" : "Light");
        }
        catch
        {
            // Silently fail if unable to save preference
        }
    }

    private static string ResolveAppVersion()
    {
        var asm = Assembly.GetExecutingAssembly();
        var informational = asm
            .GetCustomAttribute<AssemblyInformationalVersionAttribute>()?
            .InformationalVersion;

        var clean = informational?.Split('+')[0];
        if (string.IsNullOrWhiteSpace(clean))
            clean = asm.GetName().Version?.ToString(3);

        // Additional fallback: try to read from git tag if running locally
        if (string.IsNullOrWhiteSpace(clean) || clean == "0.0.0")
        {
            try
            {
                var gitDir = FindGitDirectory(AppContext.BaseDirectory);
                if (gitDir != null)
                {
                    var version = ReadGitDescribe(gitDir);
                    if (!string.IsNullOrWhiteSpace(version))
                        clean = version;
                }
            }
            catch
            {
                // Ignore git read errors, use default
            }
        }

        return $"v{(string.IsNullOrWhiteSpace(clean) ? "0.0.0" : clean)}";
    }

    private static string? FindGitDirectory(string startPath)
    {
        var current = new DirectoryInfo(startPath);
        while (current != null)
        {
            var gitDir = Path.Combine(current.FullName, ".git");
            if (Directory.Exists(gitDir))
                return gitDir;
            current = current.Parent;
        }
        return null;
    }

    private static string? ReadGitDescribe(string gitDir)
    {
        try
        {
            var packedRefs = Path.Combine(gitDir, "packed-refs");
            if (File.Exists(packedRefs))
            {
                var lines = File.ReadAllLines(packedRefs);
                var tagLine = lines
                    .Where(l => !l.StartsWith("#") && l.Contains("refs/tags/v"))
                    .OrderByDescending(l => l)
                    .FirstOrDefault();
                
                if (tagLine != null)
                {
                    var parts = tagLine.Split(' ');
                    var tag = parts[1]?.Replace("refs/tags/v", "");
                    return tag;
                }
            }

            // Also check loose refs
            var refsTagsDir = Path.Combine(gitDir, "refs", "tags");
            if (Directory.Exists(refsTagsDir))
            {
                var tags = Directory.GetFiles(refsTagsDir)
                    .OrderByDescending(f => f)
                    .FirstOrDefault();
                
                if (tags != null)
                    return Path.GetFileName(tags);
            }
        }
        catch
        {
            // Ignore errors
        }
        return null;
    }
}
