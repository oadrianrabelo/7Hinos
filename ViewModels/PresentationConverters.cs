using System;
using System.Globalization;
using System.Linq;
using Avalonia.Data;
using Avalonia.Data.Converters;
using SevenHinos.Models;

namespace SevenHinos.ViewModels;

/// <summary>
/// Converts an <see cref="OutputMode"/> to bool/visibility.
/// Parameter is a comma-separated list of mode names that should return true.
/// Example: ConverterParameter='Lyrics,Playback'
/// </summary>
public sealed class OutputModeToVisibleConverter : IValueConverter
{
    public static readonly OutputModeToVisibleConverter Instance = new();

    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value is not OutputMode mode) return false;
        var allowed = (parameter as string ?? string.Empty)
            .Split(',', StringSplitOptions.RemoveEmptyEntries);
        return allowed.Any(a =>
            Enum.TryParse<OutputMode>(a.Trim(), ignoreCase: true, out var parsed)
            && parsed == mode);
    }

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

/// <summary>Converts SlideTransitionType to a display-friendly string.</summary>
public sealed class TransitionTypeToLabelConverter : IValueConverter
{
    public static readonly TransitionTypeToLabelConverter Instance = new();

    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        value is SlideTransitionType t ? t switch
        {
            SlideTransitionType.CrossFade  => "Cross-fade",
            SlideTransitionType.SlideLeft  => "Deslizar",
            SlideTransitionType.SlideUp    => "Subir",
            SlideTransitionType.Instant    => "Instantâneo",
            _                              => t.ToString()
        } : string.Empty;

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

/// <summary>Converts OutputMode to a display-friendly label.</summary>
public sealed class OutputModeToLabelConverter : IValueConverter
{
    public static readonly OutputModeToLabelConverter Instance = new();

    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        value is OutputMode m ? m switch
        {
            OutputMode.Lyrics   => "Letra",
            OutputMode.Blank    => "Branco",
            OutputMode.Black    => "Preto",
            OutputMode.Playback => "Playback",
            _                   => m.ToString()
        } : string.Empty;

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

/// <summary>Converts bool IsAutoSync → short label shown on the toggle button.</summary>
public sealed class BoolToSyncLabelConverter : IValueConverter
{
    public static readonly BoolToSyncLabelConverter Instance = new();

    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        value is true ? "AUTO" : "MANUAL";

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

/// <summary>Converts a <see cref="PlayMode"/> to a short Portuguese display label.</summary>
public sealed class PlayModeToLabelConverter : IValueConverter
{
    public static readonly PlayModeToLabelConverter Instance = new();

    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        value is PlayMode m ? m switch
        {
            PlayMode.SlideWithAudio    => "Slide Cantado",
            PlayMode.SlideWithPlayback => "Slide Playback",
            PlayMode.SlideOnly         => "Slide s/ Áudio",
            PlayMode.AudioOnly         => "Só Áudio",
            PlayMode.PlaybackOnly      => "Só Playback",
            PlayMode.LyricsOnly        => "Letra",
            _                          => m.ToString()
        } : string.Empty;

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

/// <summary>
/// Returns true when the bound value equals the ConverterParameter.
/// Used by RadioButton IsChecked bindings for enum/int selection.
/// ConvertBack: returns the parameter when checked (true), unsetvalue otherwise.
/// </summary>
public sealed class IsEqualConverter : IValueConverter
{
    public static readonly IsEqualConverter Instance = new();

    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
        => Equals(value, parameter);

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => value is true ? parameter! : BindingOperations.DoNothing;
}
