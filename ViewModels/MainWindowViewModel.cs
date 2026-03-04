using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Styling;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SevenHinos.Services;

namespace SevenHinos.ViewModels;

public partial class MainWindowViewModel : ViewModelBase
{
    public PlayerViewModel         Player         { get; }
    public FileValidationViewModel FileValidation { get; }
    public ImportViewModel         Import         { get; }
    public SongManagerViewModel    SongManager    { get; }

    [ObservableProperty] private ViewModelBase _currentPage;
    [ObservableProperty] private bool _isDarkMode = true;

    /// <summary>Raised when the user clicks "Apresentação". View opens the PresentationWindow.</summary>
    public event Action? OpenPresentationRequested;

    public MainWindowViewModel(
        PlayerViewModel player,
        FileValidationViewModel fileValidation,
        ImportViewModel import,
        SongManagerViewModel songManager)
    {
        Player         = player;
        FileValidation = fileValidation;
        Import         = import;
        SongManager    = songManager;
        _currentPage   = SongManager;
    }

    [RelayCommand]
    private void Navigate(ViewModelBase page) => CurrentPage = page;

    [RelayCommand]
    private void OpenPresentation() => OpenPresentationRequested?.Invoke();

    [RelayCommand]
    private void ToggleTheme()
    {
        IsDarkMode = !IsDarkMode;
        Application.Current!.RequestedThemeVariant =
            IsDarkMode ? ThemeVariant.Dark : ThemeVariant.Light;
    }
}
