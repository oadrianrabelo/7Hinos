using Avalonia;
using Avalonia.Styling;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SevenHinos.Services;

namespace SevenHinos.ViewModels;

public partial class MainWindowViewModel : ViewModelBase
{
    public SongListViewModel SongList { get; }

    [ObservableProperty]
    private ViewModelBase _currentPage;

    [ObservableProperty]
    private bool _isDarkMode = true;

    public MainWindowViewModel(ISongService songService)
    {
        SongList = new SongListViewModel(songService);
        _currentPage = SongList;
    }

    [RelayCommand]
    private void Navigate(ViewModelBase page) => CurrentPage = page;

    [RelayCommand]
    private void ToggleTheme()
    {
        IsDarkMode = !IsDarkMode;
        Application.Current!.RequestedThemeVariant =
            IsDarkMode ? ThemeVariant.Dark : ThemeVariant.Light;
    }
}
