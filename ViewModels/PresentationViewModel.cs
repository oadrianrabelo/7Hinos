using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Platform;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SevenHinos.Models;
using SevenHinos.Services;
using SevenHinos.Views;

namespace SevenHinos.ViewModels;

// ─── Per-slide item shown in the operator slide list ─────────────────────────

public sealed class SlideItemViewModel
{
    public int      Index     { get; init; }
    public string   Text      { get; init; } = string.Empty;
    public TimeSpan Time      { get; init; }
    public string   ShortText => Text.Length > 50 ? Text[..50] + "…" : Text;
    public string   TimeText  => Time > TimeSpan.Zero ? Time.ToString(@"m\:ss") : string.Empty;
    public bool     HasTime   => Time > TimeSpan.Zero;
}

// ─── Per-screen item shown in the screen picker ───────────────────────────────

public sealed class ScreenItemViewModel
{
    public int    Index      { get; init; }
    public string Name       { get; init; } = string.Empty;
    public string Resolution { get; init; } = string.Empty;
    public string Label      => $"Monitor {Index + 1} — {Name} ({Resolution})";
}

// ─── Main operator ViewModel ─────────────────────────────────────────────────

public sealed partial class PresentationViewModel : ViewModelBase
{
    private readonly ISongService      _songService;
    private readonly PresentationState _state;
    private readonly IAudioService     _audio;
    private readonly PlayerViewModel   _player;

    // Local root for Oracle-downloaded audio files
    private static readonly string LocalAssetsRoot = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
        "7Hinos", "assets");

    // Output window references (null when not active)
    private Window? _publicWindow;
    private Window? _preacherWindow;

    // Parsed slide texts and timecodes for the current song
    private readonly List<string>   _slides     = [];
    private readonly List<TimeSpan> _slideTimes = [];
    private bool _updatingSlide;   // prevents reentrancy in SelectedSlide ↔ ApplySlide

    // All songs — filtered/sorted into Albums for the tree view
    private List<Song> _allSongs = [];

    // ── Observable state ─────────────────────────────────────────────────────

    [ObservableProperty] private Song?                _selectedSong;
    [ObservableProperty] private int                  _currentSlideIndex = -1;
    [ObservableProperty] private bool                 _isPickingScreens;
    [ObservableProperty] private int                  _selectedPublicScreenIndex   = 0;
    [ObservableProperty] private int                  _selectedPreacherScreenIndex = 1; // -1 = none
    [ObservableProperty] private object?              _treeSelectedItem;
    [ObservableProperty] private string               _searchQuery = string.Empty;
    [ObservableProperty] private bool                 _isAutoSync  = true;
    [ObservableProperty] private SlideItemViewModel?  _selectedSlide;
    [ObservableProperty] private double               _currentSlideProgress;  // 0.0 – 1.0
    [ObservableProperty] private PlayMode             _selectedPlayMode = PlayMode.SlideWithAudio;
    [ObservableProperty] private LibrarySection       _activeSection    = LibrarySection.Hymnal;

    /// <summary>True when at least one slide for the current song carries a timing marker.</summary>
    public bool HasTimings { get; private set; }

    partial void OnSearchQueryChanged(string value)  => RefreshActiveAlbums();
    partial void OnActiveSectionChanged(LibrarySection value) => RefreshActiveAlbums();

    /// <summary>All section infos — bound to the top navigation button list.</summary>
    public IReadOnlyList<LibrarySectionInfo> Sections => AlbumGroup.Sections;

    public PresentationState          State        => _state;
    /// <summary>Exposed so PresentationWindow can bind PlayerView to it.</summary>
    public PlayerViewModel             Player       => _player;
    public ObservableCollection<Song>               Songs        { get; } = [];
    /// <summary>Albums shown in the TreeView (search-filtered, smart-sorted).</summary>
    public ObservableCollection<AlbumGroup>         Albums       { get; } = [];
    public ObservableCollection<SlideItemViewModel> Slides       { get; } = [];
    public ObservableCollection<ScreenItemViewModel> AvailableScreens { get; } = [];

    // Exposed for binding in the operator panel
    public OutputMode[]         PublicModes   { get; } = Enum.GetValues<OutputMode>();
    public OutputMode[]         PreacherModes { get; } = Enum.GetValues<OutputMode>();
    public SlideTransitionType[] Transitions  { get; } = Enum.GetValues<SlideTransitionType>();

    // ── Constructor ──────────────────────────────────────────────────────────

    public PresentationViewModel(ISongService songService, PresentationState state, IAudioService audio, PlayerViewModel player)
    {
        _songService = songService;
        _state       = state;
        _audio       = audio;
        _player      = player;
        _audio.PositionChanged += OnAudioPositionChanged;
        _ = LoadSongsAsync();
    }

    // ── Song / slide loading ─────────────────────────────────────────────────

    private async Task LoadSongsAsync()
    {
        var songs = await _songService.GetAllAsync();
        Songs.Clear();
        _allSongs.Clear();
        foreach (var s in songs) { Songs.Add(s); _allSongs.Add(s); }
        RefreshActiveAlbums();
    }

    // ── Library refresh ────────────────────────────────────────────────────

    private void RefreshActiveAlbums()
    {
        Albums.Clear();
        foreach (var ag in AlbumGroup.BuildSection(_allSongs, ActiveSection, SearchQuery))
            Albums.Add(ag);
    }

    [RelayCommand]
    private void SelectSection(LibrarySectionInfo info) => ActiveSection = info.Section;

    partial void OnTreeSelectedItemChanged(object? value)
    {
        if (value is Song s && !ReferenceEquals(s, SelectedSong))
            SelectedSong = s;
    }

    partial void OnSelectedSongChanged(Song? value)
    {
        SelectedPlayMode           = PlayMode.SlideWithAudio;
        _state.CurrentSong         = value;
        _state.CurrentSlideContent = null;
        _state.NextSlideText       = string.Empty;
        _ = LoadSlidesForSongAsync(value);

        // Auto-play the audio as soon as a song is selected
        if (value is not null) AutoPlay(value);
        else _player.StopPlayback();
    }

    // ── Audio path resolution ────────────────────────────────────────────────

    /// <summary>
    /// Finds an audio file for <paramref name="song"/> using these strategies in order:
    /// 1. song.AudioFilePath (if set and file exists on disk)
    /// 2. {assetsRoot}/{album}/{rawTitle}.mp3  where rawTitle strips any leading "N - " hymnal number
    /// 3. Case-insensitive scan of the album folder
    /// </summary>
    private static string? FindAudioPath(Song song)
    {
        // 1. Stored path (LouvorJA install)
        if (!string.IsNullOrWhiteSpace(song.AudioFilePath) && File.Exists(song.AudioFilePath))
            return song.AudioFilePath;

        // Strip leading number prefix added to hymnal titles: "42 - Title" → "Title"
        var rawTitle = StripHymnalNumber(song.Title);

        // 2. Canonical Oracle-assets path
        var candidate = Path.Combine(LocalAssetsRoot, song.Album, rawTitle + ".mp3");
        if (File.Exists(candidate)) return candidate;

        // 3. Case-insensitive scan of the album folder
        var folder = Path.Combine(LocalAssetsRoot, song.Album);
        if (Directory.Exists(folder))
        {
            var want = rawTitle + ".mp3";
            var hit = Directory.EnumerateFiles(folder, "*.mp3")
                               .FirstOrDefault(f => string.Equals(
                                   Path.GetFileName(f), want,
                                   StringComparison.OrdinalIgnoreCase));
            if (hit is not null) return hit;
        }

        return null;
    }

    /// <summary>
    /// Like FindAudioPath but looks for the playback/instrumental file first,
    /// falling back to the normal audio file.
    /// Instrumental convention: "{Title} - PB.mp3"
    /// </summary>
    private static string? FindInstrumentalPath(Song song)
    {
        // Stored instrumental path
        if (!string.IsNullOrWhiteSpace(song.InstrumentalFilePath) && File.Exists(song.InstrumentalFilePath))
            return song.InstrumentalFilePath;

        var rawTitle = StripHymnalNumber(song.Title);

        // Oracle assets " - PB" convention
        var pbCandidate = Path.Combine(LocalAssetsRoot, song.Album, rawTitle + " - PB.mp3");
        if (File.Exists(pbCandidate)) return pbCandidate;

        // Case-insensitive scan
        var folder = Path.Combine(LocalAssetsRoot, song.Album);
        if (Directory.Exists(folder))
        {
            var want = rawTitle + " - PB.mp3";
            var hit = Directory.EnumerateFiles(folder, "*.mp3")
                               .FirstOrDefault(f => string.Equals(
                                   Path.GetFileName(f), want,
                                   StringComparison.OrdinalIgnoreCase));
            if (hit is not null) return hit;
        }

        // Fall back to regular audio
        return FindAudioPath(song);
    }

    /// <summary>
    /// Strips a leading hymnal track number prefix from a title.
    /// "42 - Santo, Santo" → "Santo, Santo"
    /// "Alegria" → "Alegria"  (unchanged)
    /// </summary>
    private static string StripHymnalNumber(string title)
    {
        var m = System.Text.RegularExpressions.Regex.Match(title, @"^\d+ - (.*)");
        return m.Success ? m.Groups[1].Value : title;
    }

    private void AutoPlay(Song song)
    {
        // Modes that don't need audio
        if (SelectedPlayMode is PlayMode.SlideOnly or PlayMode.LyricsOnly)
        {
            _player.SetCurrentSong(song);
            _player.StopPlayback();
            return;
        }

        bool useInstrumental = SelectedPlayMode is PlayMode.SlideWithPlayback or PlayMode.PlaybackOnly;

        string? path = useInstrumental
            ? FindInstrumentalPath(song)
            : FindAudioPath(song);

        if (path is not null)
            _player.PlayPath(song, path);
        else
            _player.SetCurrentSong(song); // show song info but no audio
    }

    private async Task LoadSlidesForSongAsync(Song? song)
    {
        _slides.Clear();
        Slides.Clear();

        if (song is null) return;

        // Load full song with per-slide data
        var fullSong = await _songService.GetWithSlidesAsync(song.Id) ?? song;
        LoadSlides(fullSong);

        // also notify HasSlides-based commands after loading
        if (_slides.Count > 0)
        {
            ApplySlide(0);
            RestartSongCommand.NotifyCanExecuteChanged();
            // Set public output mode based on play mode when live
            if (_state.IsActive)
            {
                _state.PublicMode = SelectedPlayMode switch
                {
                    PlayMode.AudioOnly    => OutputMode.Blank,
                    PlayMode.PlaybackOnly => OutputMode.Blank,
                    _                    => OutputMode.Lyrics,
                };
            }
        }
    }

    private void LoadSlides(Song? song)
    {
        _slides.Clear();
        _slideTimes.Clear();
        Slides.Clear();
        HasTimings = false;
        if (song is null) return;

        // Prefer SongSlide entities; fall back to parsing Lyrics by double-newline
        if (song.Slides.Any(s => s.ShowSlide))
        {
            var visibleSlides = song.Slides.Where(s => s.ShowSlide).OrderBy(s => s.Order).ToList();
            _slides.AddRange(visibleSlides.Select(s => s.Content));
            _slideTimes.AddRange(visibleSlides.Select(s => s.Time));
        }
        else
        {
            var parsed = ParseLyrics(song.Lyrics);
            _slides.AddRange(parsed);
            _slideTimes.AddRange(Enumerable.Repeat(TimeSpan.Zero, parsed.Count));
        }

        HasTimings = _slideTimes.Any(t => t > TimeSpan.Zero);
        OnPropertyChanged(nameof(HasTimings));

        for (int i = 0; i < _slides.Count; i++)
            Slides.Add(new SlideItemViewModel { Index = i, Text = _slides[i], Time = _slideTimes[i] });
    }

    private static List<string> ParseLyrics(string lyrics)
    {
        if (string.IsNullOrWhiteSpace(lyrics)) return [];
        return lyrics
            .Split(["\r\n\r\n", "\n\n"], StringSplitOptions.RemoveEmptyEntries)
            .Select(s => s.Trim())
            .Where(s => !string.IsNullOrEmpty(s))
            .ToList();
    }

    // ── Album grouping ──────────────────────────────────────────────────

    // ── Play mode ────────────────────────────────────────────────────────────

    /// <summary>
    /// Re-triggers playback immediately when the operator switches mode.
    /// This allows switching between vocal/playback/slide-only without re-selecting a song.
    /// Play mode is per-session only — not persisted to the database.
    /// </summary>
    partial void OnSelectedPlayModeChanged(PlayMode value)
    {
        if (SelectedSong is { } song)
            AutoPlay(song);
    }

    // ── Slide navigation ─────────────────────────────────────────────────

    [RelayCommand(CanExecute = nameof(CanGoNext))]
    private void NextSlide()
    {
        if (CurrentSlideIndex < _slides.Count - 1)
            ApplySlide(CurrentSlideIndex + 1, seekAudio: true);
    }
    private bool CanGoNext => CurrentSlideIndex < _slides.Count - 1;

    [RelayCommand(CanExecute = nameof(CanGoPrev))]
    private void PreviousSlide()
    {
        if (CurrentSlideIndex > 0)
            ApplySlide(CurrentSlideIndex - 1, seekAudio: true);
    }
    private bool CanGoPrev => CurrentSlideIndex > 0;

    [RelayCommand]
    private void GoToSlide(SlideItemViewModel item) => ApplySlide(item.Index, seekAudio: true);

    /// <summary>Restarts from the beginning: seeks audio to 0:00 and jumps to slide 0.</summary>
    [RelayCommand(CanExecute = nameof(HasSlides))]
    private void RestartSong()
    {
        _player.SeekToTime(TimeSpan.Zero);
        ApplySlide(0, seekAudio: false);
    }
    private bool HasSlides => _slides.Count > 0;

    // Called when operator clicks a slide in the list
    partial void OnSelectedSlideChanged(SlideItemViewModel? value)
    {
        if (_updatingSlide || value is null) return;
        if (value.Index != CurrentSlideIndex)
            ApplySlide(value.Index, seekAudio: true);
    }

    private void ApplySlide(int index, bool seekAudio = false)
    {
        if (index < 0 || index >= _slides.Count) return;

        CurrentSlideIndex = index;

        _updatingSlide = true;
        SelectedSlide  = index < Slides.Count ? Slides[index] : null;
        _updatingSlide = false;

        _state.CurrentSlideContent = new SlideContent(
            Text:      _slides[index],
            Index:     index + 1,
            Total:     _slides.Count,
            SongTitle: SelectedSong?.Title  ?? string.Empty,
            Album:     SelectedSong?.Album  ?? string.Empty);

        _state.NextSlideText = index + 1 < _slides.Count
            ? _slides[index + 1]
            : string.Empty;

        // Only seek when the operator explicitly navigated — never on auto-sync advance
        if (seekAudio && index < _slideTimes.Count)
        {
            var t = _slideTimes[index];
            if (t > TimeSpan.Zero)
                _player.SeekToTime(t);
        }

        NextSlideCommand.NotifyCanExecuteChanged();
        PreviousSlideCommand.NotifyCanExecuteChanged();
        RestartSongCommand.NotifyCanExecuteChanged();
    }

    // ── Audio-sync ────────────────────────────────────────────────────────────

    private void OnAudioPositionChanged(object? sender, TimeSpan position)
    {
        if (!IsAutoSync || !HasTimings || _slideTimes.Count == 0) return;

        // Find the last slide whose non-zero timecode has been reached
        int targetIndex = 0;
        for (int i = 0; i < _slideTimes.Count; i++)
        {
            if (_slideTimes[i] > TimeSpan.Zero && position >= _slideTimes[i])
                targetIndex = i;
        }

        // Calculate progress between current slide start and next slide start
        TimeSpan currentStart = _slideTimes[targetIndex];
        TimeSpan nextStart    = TimeSpan.MaxValue;
        for (int j = targetIndex + 1; j < _slideTimes.Count; j++)
        {
            if (_slideTimes[j] > TimeSpan.Zero) { nextStart = _slideTimes[j]; break; }
        }

        double progress = nextStart != TimeSpan.MaxValue && nextStart > currentStart
            ? Math.Clamp((position - currentStart).TotalMilliseconds / (nextStart - currentStart).TotalMilliseconds, 0, 1)
            : currentStart > TimeSpan.Zero && position >= currentStart ? 1.0 : 0.0;

        Dispatcher.UIThread.Post(() =>
        {
            CurrentSlideProgress = progress;
            if (targetIndex != CurrentSlideIndex)
                ApplySlide(targetIndex, seekAudio: false); // do NOT seek — audio is already playing
        });
    }

    // ── Output mode controls ─────────────────────────────────────────────────

    [RelayCommand]
    private void SetPublicMode(OutputMode mode) => _state.PublicMode = mode;

    [RelayCommand]
    private void SetPreacherMode(OutputMode mode) => _state.PreacherMode = mode;

    [RelayCommand]
    private void SetTransition(SlideTransitionType t) => _state.Transition = t;

    // ── Screen picker ─────────────────────────────────────────────────────────

    /// <summary>Called from View code-behind with the list of physical screens.</summary>
    public void InitScreens(IReadOnlyList<Screen> screens)
    {
        AvailableScreens.Clear();
        for (int i = 0; i < screens.Count; i++)
        {
            var s = screens[i];
            AvailableScreens.Add(new ScreenItemViewModel
            {
                Index      = i,
                Name       = $"Monitor {i + 1}",
                Resolution = $"{s.Bounds.Width}×{s.Bounds.Height}"
            });
        }

        // Default: second screen for public, third for preacher (if available)
        SelectedPublicScreenIndex   = Math.Min(1, screens.Count - 1);
        SelectedPreacherScreenIndex = screens.Count > 2 ? 2 : -1;
    }

    [RelayCommand(CanExecute = nameof(IsNotActive))]
    private void RequestStartPresentation() => IsPickingScreens = true;
    private bool IsNotActive => !_state.IsActive;

    [RelayCommand]
    private async Task ConfirmStartPresentationAsync()
    {
        IsPickingScreens = false;
        await Dispatcher.UIThread.InvokeAsync(OpenOutputWindows);
    }

    [RelayCommand(CanExecute = nameof(IsActive))]
    private void StopPresentation()
    {
        _publicWindow?.Close();
        _preacherWindow?.Close();
        _publicWindow   = null;
        _preacherWindow = null;
        _state.IsActive = false;
        RequestStartPresentationCommand.NotifyCanExecuteChanged();
        StopPresentationCommand.NotifyCanExecuteChanged();
    }
    private bool IsActive => _state.IsActive;

    // ── Window management ────────────────────────────────────────────────────

    private void OpenOutputWindows()
    {
        var screens = AvailableScreens.ToList();
        if (screens.Count == 0) return;

        // Public output window
        var pubIdx = Math.Clamp(SelectedPublicScreenIndex, 0, screens.Count - 1);
        _publicWindow = CreateOutputWindow<PublicOutputWindow>(pubIdx);

        // Preacher / confidence monitor (optional)
        if (SelectedPreacherScreenIndex >= 0 && SelectedPreacherScreenIndex < screens.Count
            && SelectedPreacherScreenIndex != pubIdx)
        {
            _preacherWindow = CreateOutputWindow<ConfidenceMonitorWindow>(SelectedPreacherScreenIndex);
        }

        _state.PublicMode = OutputMode.Lyrics;
        _state.IsActive   = true;
        RequestStartPresentationCommand.NotifyCanExecuteChanged();
        StopPresentationCommand.NotifyCanExecuteChanged();
    }

    private static T CreateOutputWindow<T>(int screenIndex) where T : Window, new()
    {
        var app     = Avalonia.Application.Current!;
        var desktop = (Avalonia.Controls.ApplicationLifetimes.IClassicDesktopStyleApplicationLifetime)
                      app.ApplicationLifetime!;

        // Get screens from the main window
        var mainWindow = desktop.MainWindow!;
        var screens    = mainWindow.Screens.All;
        var screen     = screenIndex < screens.Count ? screens[screenIndex] : screens[0];

        var win = new T
        {
            WindowStartupLocation = WindowStartupLocation.Manual,
            Position              = new PixelPoint(screen.Bounds.X, screen.Bounds.Y),
        };

        win.Show();
        win.WindowState = WindowState.FullScreen;
        return win;
    }
}
