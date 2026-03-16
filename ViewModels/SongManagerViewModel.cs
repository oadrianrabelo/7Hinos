using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SevenHinos.Models;
using SevenHinos.Services;

namespace SevenHinos.ViewModels;

public sealed partial class SongManagerViewModel : ViewModelBase
{
    private readonly ISongService _songService;

    [ObservableProperty] private string               _searchQuery   = string.Empty;
    [ObservableProperty] private Song?                _selectedSong;
    [ObservableProperty] private SongEditorViewModel? _editor;        // null = list view
    [ObservableProperty] private bool                 _isLoading;
    [ObservableProperty] private bool                 _isConfirmingDelete;
    [ObservableProperty] private object?              _treeSelectedItem;

    public ObservableCollection<Song>        Songs  { get; } = [];
    public ObservableCollection<AlbumGroup>  Albums { get; } = [];

    public bool EditorIsOpen => Editor is not null;

    partial void OnTreeSelectedItemChanged(object? value)
    {
        if (value is Song s && !ReferenceEquals(s, SelectedSong))
            SelectedSong = s;
    }

    partial void OnSelectedSongChanged(Song? value)
    {
        if (value is not null && !ReferenceEquals(value, TreeSelectedItem))
            TreeSelectedItem = value;
    }

    public SongManagerViewModel(ISongService songService)
    {
        _songService = songService;
        _ = LoadAsync();
    }

    public Task ReloadAsync() => LoadAsync();

    // ── Load / search ─────────────────────────────────────────────────────────

    [RelayCommand]
    private async Task LoadAsync()
    {
        IsLoading = true;
        try
        {
            var songs = string.IsNullOrWhiteSpace(SearchQuery)
                ? await _songService.GetAllAsync()
                : await _songService.SearchAsync(SearchQuery);

            Songs.Clear();
            foreach (var s in songs) Songs.Add(s);
            RebuildAlbums(songs);
            SelectedSong = Songs.FirstOrDefault();
        }
        finally { IsLoading = false; }
    }

    partial void OnSearchQueryChanged(string value) => _ = LoadAsync();

    // ── Add ───────────────────────────────────────────────────────────────────

    [RelayCommand]
    private void AddNew()
    {
        var blank = new Song();
        OpenEditor(blank);
    }

    // ── Edit ──────────────────────────────────────────────────────────────────

    [RelayCommand]
    private async Task EditSelected()
    {
        if (SelectedSong is null) return;
        var song = await _songService.GetWithSlidesAsync(SelectedSong.Id)
                   ?? SelectedSong;
        OpenEditor(song);
    }

    // ── Delete ────────────────────────────────────────────────────────────────

    [RelayCommand]
    private void RequestDelete() =>
        IsConfirmingDelete = SelectedSong is not null;

    [RelayCommand]
    private async Task ConfirmDelete()
    {
        if (SelectedSong is null) { IsConfirmingDelete = false; return; }
        await _songService.DeleteAsync(SelectedSong.Id);
        Songs.Remove(SelectedSong);
        RebuildAlbums(Songs);
        SelectedSong         = Songs.FirstOrDefault();
        IsConfirmingDelete   = false;
    }

    [RelayCommand]
    private void CancelDelete() => IsConfirmingDelete = false;

    // ── Editor helpers ────────────────────────────────────────────────────────

    private void OpenEditor(Song song)
    {
        var vm = new SongEditorViewModel(_songService, song);
        vm.Saved += OnEditorSaved;
        vm.Cancelled += CloseEditor;
        Editor = vm;
        OnPropertyChanged(nameof(EditorIsOpen));
    }

    private void OnEditorSaved(Song saved)
    {
        // Refresh list and select the saved song
        _ = ReloadAndSelectAsync(saved.Id);
        CloseEditor();
    }

    private void CloseEditor()
    {
        Editor = null;
        OnPropertyChanged(nameof(EditorIsOpen));
    }

    private async Task ReloadAndSelectAsync(int id)
    {
        await LoadAsync();
        SelectedSong = Songs.FirstOrDefault(s => s.Id == id);
    }

    // ── Album grouping ────────────────────────────────────────────────────────

    private void RebuildAlbums(IEnumerable<Song> songs)
    {
        Albums.Clear();
        foreach (var ag in AlbumGroup.BuildAll(songs))
            Albums.Add(ag);
    }
}
