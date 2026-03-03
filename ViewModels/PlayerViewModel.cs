using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SevenHinos.Models;
using SevenHinos.Services;

namespace SevenHinos.ViewModels;

public partial class PlayerViewModel : ViewModelBase
{
    private readonly IAudioService _audio;
    private bool _updatingFromAudio;

    [ObservableProperty] private Song?  _currentSong;
    [ObservableProperty] private bool   _isPlaying;
    [ObservableProperty] private double _position;           // 0.0 – 1.0
    [ObservableProperty] private double _volume = 0.8;       // 0.0 – 1.0
    [ObservableProperty] private string _currentTimeText = "0:00";
    [ObservableProperty] private string _totalTimeText   = "0:00";

    public PlayerViewModel(IAudioService audio)
    {
        _audio = audio;
        _audio.PositionChanged  += OnAudioPositionChanged;
        _audio.PlaybackStarted  += OnAudioPlaybackStarted;
        _audio.PlaybackEnded    += OnAudioPlaybackEnded;
    }

    /// <summary>Called from SongListViewModel when the user clicks "Reproduzir".</summary>
    public void Play(Song song)
    {
        if (string.IsNullOrWhiteSpace(song.AudioFilePath)) return;
        CurrentSong = song;
        _audio.Play(song.AudioFilePath);
    }

    [RelayCommand]
    private void TogglePlayPause()
    {
        if (_audio.IsPlaying) _audio.Pause();
        else                   _audio.Resume();
    }

    [RelayCommand]
    private void Stop()
    {
        _audio.Stop();
        IsPlaying = false;

        _updatingFromAudio = true;
        Position = 0;
        _updatingFromAudio = false;

        CurrentTimeText = "0:00";
    }

    // When the user drags the slider OnPositionChanged fires → seek audio
    partial void OnPositionChanged(double value)
    {
        if (!_updatingFromAudio)
            _audio.Seek(value);
    }

    // When the user moves the volume slider → update audio volume
    partial void OnVolumeChanged(double value)
        => _audio.Volume = (float)value;

    // ── Audio event handlers ──────────────────────────────────────────────────

    private void OnAudioPositionChanged(object? sender, TimeSpan position)
    {
        var duration = _audio.Duration;
        Dispatcher.UIThread.Post(() =>
        {
            _updatingFromAudio = true;
            IsPlaying       = _audio.IsPlaying;
            Position        = duration.TotalMilliseconds > 0
                                ? position.TotalMilliseconds / duration.TotalMilliseconds
                                : 0;
            CurrentTimeText = FormatTime(position);
            TotalTimeText   = FormatTime(duration);
            _updatingFromAudio = false;
        });
    }

    private void OnAudioPlaybackStarted(object? sender, EventArgs e)
        => Dispatcher.UIThread.Post(() =>
        {
            IsPlaying     = true;
            TotalTimeText = FormatTime(_audio.Duration);
        });

    private void OnAudioPlaybackEnded(object? sender, EventArgs e)
        => Dispatcher.UIThread.Post(() =>
        {
            IsPlaying = false;
            _updatingFromAudio = true;
            Position = 0;
            _updatingFromAudio = false;
            CurrentTimeText = "0:00";
        });

    private static string FormatTime(TimeSpan ts)
        => ts.TotalHours >= 1 ? ts.ToString(@"h\:mm\:ss") : ts.ToString(@"m\:ss");
}
