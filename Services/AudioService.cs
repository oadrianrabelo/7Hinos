using LibVLCSharp.Shared;

namespace SevenHinos.Services;

public sealed class AudioService : IAudioService
{
    private readonly LibVLC _libVlc;
    private readonly MediaPlayer _player;
    private System.Timers.Timer? _positionTimer;

    public event EventHandler<TimeSpan>? PositionChanged;
    public event EventHandler? PlaybackStarted;
    public event EventHandler? PlaybackEnded;

    public bool IsPlaying => _player.IsPlaying;

    public TimeSpan Duration =>
        _player.Length > 0 ? TimeSpan.FromMilliseconds(_player.Length) : TimeSpan.Zero;

    public TimeSpan Position =>
        _player.Time > 0 ? TimeSpan.FromMilliseconds(_player.Time) : TimeSpan.Zero;

    public float Volume
    {
        get => _player.Volume / 100f;
        set => _player.Volume = (int)Math.Clamp(value * 100, 0, 100);
    }

    public AudioService()
    {
        Core.Initialize();
        _libVlc = new LibVLC(enableDebugLogs: false);
        _player = new MediaPlayer(_libVlc) { Volume = 80 };

        _player.Playing   += (_, _) => { StartTimer(); PlaybackStarted?.Invoke(this, EventArgs.Empty); };
        _player.EndReached += (_, _) => { StopTimer();  PlaybackEnded?.Invoke(this, EventArgs.Empty); };
        _player.Stopped   += (_, _) => StopTimer();
        _player.Paused    += (_, _) => StopTimer();
    }

    public void Play(string filePath)
    {
        var uri = Path.IsPathRooted(filePath)
            ? new Uri(filePath)
            : new Uri(Path.GetFullPath(filePath));

        using var media = new Media(_libVlc, uri);
        _player.Play(media);
    }

    public void Pause()  { if (_player.CanPause) _player.Pause(); }
    public void Resume() { if (_player.State == VLCState.Paused) _player.Play(); }
    public void Stop()   => _player.Stop();

    public void Seek(double fraction)
    {
        if (_player.IsSeekable)
            _player.Position = (float)Math.Clamp(fraction, 0.0, 1.0);
    }

    private void StartTimer()
    {
        StopTimer();
        _positionTimer = new System.Timers.Timer(500);
        _positionTimer.Elapsed += (_, _) => PositionChanged?.Invoke(this, Position);
        _positionTimer.Start();
    }

    private void StopTimer()
    {
        _positionTimer?.Dispose();
        _positionTimer = null;
    }

    public void Dispose()
    {
        StopTimer();
        _player.Stop();
        _player.Dispose();
        _libVlc.Dispose();
    }
}
