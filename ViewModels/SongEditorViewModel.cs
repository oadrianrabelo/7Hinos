using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SevenHinos.Models;
using SevenHinos.Services;

namespace SevenHinos.ViewModels;

// ── One editable slide row ────────────────────────────────────────────────────

public sealed partial class SlideEditorItemViewModel : ObservableObject
{
    [ObservableProperty] private string _text = string.Empty;

    public IRelayCommand MoveUpCommand   { get; }
    public IRelayCommand MoveDownCommand { get; }
    public IRelayCommand DeleteCommand   { get; }

    public SlideEditorItemViewModel(
        string text,
        Action<SlideEditorItemViewModel> onMoveUp,
        Action<SlideEditorItemViewModel> onMoveDown,
        Action<SlideEditorItemViewModel> onDelete)
    {
        _text          = text;
        MoveUpCommand   = new RelayCommand(() => onMoveUp(this));
        MoveDownCommand = new RelayCommand(() => onMoveDown(this));
        DeleteCommand   = new RelayCommand(() => onDelete(this));
    }
}

// ── Full song editor ──────────────────────────────────────────────────────────

public sealed partial class SongEditorViewModel : ViewModelBase
{
    private readonly ISongService _songService;
    private Song _song;

    // Fired after a successful save — carries the saved Song back to the list
    public event Action<Song>? Saved;
    public event Action?       Cancelled;

    [ObservableProperty] private string  _title      = string.Empty;
    [ObservableProperty] private string  _album      = string.Empty;
    [ObservableProperty] private string? _youtubeUrl;
    [ObservableProperty] private bool    _isSaving;
    [ObservableProperty] private string  _errorText  = string.Empty;

    public bool IsNew => _song.Id == 0;
    public string EditorTitle => IsNew ? "Nova música" : "Editar música";

    public ObservableCollection<SlideEditorItemViewModel> Slides { get; } = [];

    public SongEditorViewModel(ISongService songService, Song song)
    {
        _songService = songService;
        _song        = song;

        Title      = song.Title;
        Album      = song.Album;
        YoutubeUrl = song.YoutubeUrl;

        foreach (var sl in song.Slides.OrderBy(s => s.Order))
            Slides.Add(MakeItem(sl.Content));

        if (Slides.Count == 0)
            Slides.Add(MakeItem(string.Empty));
    }

    private SlideEditorItemViewModel MakeItem(string text) =>
        new(text, MoveUp, MoveDown, DeleteSlide);

    // ── Slide manipulation ────────────────────────────────────────────────────

    [RelayCommand]
    private void AddSlide() => Slides.Add(MakeItem(string.Empty));

    private void DeleteSlide(SlideEditorItemViewModel item)
    {
        if (Slides.Count > 1) Slides.Remove(item);
    }

    private void MoveUp(SlideEditorItemViewModel item)
    {
        int i = Slides.IndexOf(item);
        if (i > 0) Slides.Move(i, i - 1);
    }

    private void MoveDown(SlideEditorItemViewModel item)
    {
        int i = Slides.IndexOf(item);
        if (i >= 0 && i < Slides.Count - 1) Slides.Move(i, i + 1);
    }

    // ── Save / Cancel ─────────────────────────────────────────────────────────

    [RelayCommand]
    private async Task SaveAsync()
    {
        ErrorText = string.Empty;

        if (string.IsNullOrWhiteSpace(Title))
        {
            ErrorText = "O título é obrigatório.";
            return;
        }

        IsSaving = true;
        try
        {
            _song.Title      = Title.Trim();
            _song.Album      = Album.Trim();
            _song.YoutubeUrl = string.IsNullOrWhiteSpace(YoutubeUrl) ? null : YoutubeUrl.Trim();

            var slideTexts = Slides.Select(s => s.Text).ToList();
            await _songService.SaveWithSlidesAsync(_song, slideTexts);
            Saved?.Invoke(_song);
        }
        catch (Exception ex)
        {
            ErrorText = ex.Message;
        }
        finally
        {
            IsSaving = false;
        }
    }

    [RelayCommand]
    private void Cancel() => Cancelled?.Invoke();
}
