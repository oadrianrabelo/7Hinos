using CommunityToolkit.Mvvm.ComponentModel;
using SevenHinos.Models;

namespace SevenHinos.ViewModels;

// ─── Enums ───────────────────────────────────────────────────────────────────

/// <summary>What a single output screen is currently showing.</summary>
public enum OutputMode
{
    /// <summary>Displays the current song slide with the configured transition.</summary>
    Lyrics,
    /// <summary>Fully white screen (avoid burning projector).</summary>
    Blank,
    /// <summary>Fully black screen.</summary>
    Black,
    /// <summary>Song is playing as backing track — slide shown but labelled "PLAYBACK".</summary>
    Playback,
}

/// <summary>Available slide transition animations.</summary>
public enum SlideTransitionType
{
    CrossFade,
    SlideLeft,
    SlideUp,
    Instant,
}

// ─── Immutable slide payload ──────────────────────────────────────────────────

/// <summary>
/// Changing the record reference (not just its Text) is what triggers
/// TransitioningContentControl to animate.
/// </summary>
public sealed record SlideContent(
    string Text,
    int    Index,
    int    Total,
    string SongTitle,
    string Album);

// ─── Shared singleton state ───────────────────────────────────────────────────

/// <summary>
/// Single source of truth for the live presentation.
/// Registered as singleton in DI — both output windows and the operator panel
/// observe this object directly.
/// </summary>
public sealed partial class PresentationState : ObservableObject
{
    /// <summary>Song currently loaded into the presentation.</summary>
    [ObservableProperty] private Song? _currentSong;

    /// <summary>
    /// The payload shown on screen. Changing the reference triggers the
    /// crossfade/slide animation on the output window.
    /// </summary>
    [ObservableProperty] private SlideContent? _currentSlideContent;

    /// <summary>Text of the NEXT slide — shown only on operator panel.</summary>
    [ObservableProperty] private string _nextSlideText = string.Empty;

    /// <summary>What the public screen is currently showing.</summary>
    [ObservableProperty] private OutputMode _publicMode = OutputMode.Black;

    /// <summary>What the preacher (confidence monitor) screen is showing.</summary>
    [ObservableProperty] private OutputMode _preacherMode = OutputMode.Lyrics;

    /// <summary>Slide transition type for the public screen.</summary>
    [ObservableProperty] private SlideTransitionType _transition = SlideTransitionType.CrossFade;

    /// <summary>True while output windows are open and the presentation is live.</summary>
    [ObservableProperty] private bool _isActive;
}
