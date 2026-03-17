using LibVLCSharp.Shared;

namespace SevenHinos.Services;

/// <summary>
/// Shared media factory backed by a single LibVLC instance.
/// Services create players/media through this abstraction to avoid engine conflicts.
/// </summary>
public interface IMediaEngine : IDisposable
{
    MediaPlayer CreatePlayer();
    Media CreateMedia(Uri uri);
}
