namespace SevenHinos.Models;

/// <summary>
/// How a song should be presented during a session.
/// Selected by the operator per-song; not persisted to the database.
/// </summary>
public enum PlayMode
{
    /// <summary>Slide avança automaticamente com o áudio vocal. (padrão)</summary>
    SlideWithAudio    = 0,

    /// <summary>Slide avança automaticamente com o playback instrumental.</summary>
    SlideWithPlayback = 1,

    /// <summary>Slide sem áudio — operador avança manualmente.</summary>
    SlideOnly         = 2,

    /// <summary>Toca só o áudio vocal; sem slides na tela de saída.</summary>
    AudioOnly         = 3,

    /// <summary>Toca só o playback instrumental; sem slides na tela de saída.</summary>
    PlaybackOnly      = 4,

    /// <summary>Exibe letra completa em tela cheia (sem transições de slide).</summary>
    LyricsOnly        = 5,
}
