using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SevenHinos.Models;
using SevenHinos.Services;

namespace SevenHinos.ViewModels;

public partial class SongListViewModel : ViewModelBase
{
    private readonly ISongService _songService;

    [ObservableProperty]
    private string _searchQuery = string.Empty;

    [ObservableProperty]
    private Song? _selectedSong;

    public ObservableCollection<Song> Songs { get; } = [];

    public SongListViewModel(ISongService songService)
    {
        _songService = songService;
        _ = LoadAsync();
    }

    private async Task LoadAsync()
    {
        var songs = await _songService.GetAllAsync();
        Songs.Clear();
        foreach (var s in songs)
            Songs.Add(s);
        SelectedSong = Songs.FirstOrDefault();
    }

    [RelayCommand]
    private async Task SearchAsync()
    {
        var results = await _songService.SearchAsync(SearchQuery);
        Songs.Clear();
        foreach (var s in results)
            Songs.Add(s);
        SelectedSong = Songs.FirstOrDefault();
    }

    [RelayCommand]
    private async Task ToggleFavoriteAsync(Song? song)
    {
        if (song is null) return;
        await _songService.ToggleFavoriteAsync(song.Id);
        song.IsFavorite = !song.IsFavorite;
    }
}
