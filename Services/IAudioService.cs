namespace SevenHinos.Services;

public interface IAudioService : IDisposable
{
    bool IsPlaying { get; }
    TimeSpan Duration { get; }
    TimeSpan Position { get; }
    float Volume { get; set; }

    void Play(string filePath);
    void Pause();
    void Resume();
    void Stop();
    void Seek(double fraction);    // 0.0 – 1.0
    void SeekToTime(TimeSpan time); // absolute position

    event EventHandler<TimeSpan>? PositionChanged;
    event EventHandler? PlaybackStarted;
    event EventHandler? PlaybackEnded;
}
