using LibVLCSharp.Shared;

namespace SevenHinos.Services;

public sealed class LibVlcMediaEngine : IMediaEngine
{
    private readonly LibVLC _libVlc;

    public LibVlcMediaEngine()
    {
        Core.Initialize();
        _libVlc = new LibVLC(enableDebugLogs: false);
    }

    public MediaPlayer CreatePlayer() => new(_libVlc);

    public Media CreateMedia(Uri uri) => new(_libVlc, uri);

    public void Dispose()
    {
        _libVlc.Dispose();
    }
}
