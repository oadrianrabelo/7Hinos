using System;
using System.Globalization;
using Avalonia.Data.Converters;
using Avalonia.Media;

namespace SevenHinos.ViewModels;

/// <summary>Maps <see cref="AssetStatus"/> to an emoji icon string.</summary>
public sealed class AssetStatusIconConverter : IValueConverter
{
    public static readonly AssetStatusIconConverter Instance = new();

    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        value is AssetStatus s ? s switch
        {
            AssetStatus.Ok         => "✓",
            AssetStatus.Missing    => "✗",
            AssetStatus.Damaged    => "⚠",
            AssetStatus.Downloading=> "⬇",
            AssetStatus.Failed     => "✗",
            _                      => "?"
        } : "?";

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        throw new NotSupportedException();
}

/// <summary>Returns true when the string is not null/empty.</summary>
public sealed class NullOrEmptyToBoolConverter : IValueConverter
{
    public static readonly NullOrEmptyToBoolConverter Instance = new();

    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        !string.IsNullOrEmpty(value as string);

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        throw new NotSupportedException();
}

/// <summary>Returns true when the status is <see cref="AssetStatus.Downloading"/>.</summary>
public sealed class AssetDownloadingConverter : IValueConverter
{
    public static readonly AssetDownloadingConverter Instance = new();

    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        value is AssetStatus.Downloading;

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        throw new NotSupportedException();
}

/// <summary>Maps <see cref="AssetStatus"/> to a foreground color brush string for display.</summary>
public sealed class AssetStatusColorConverter : IValueConverter
{
    public static readonly AssetStatusColorConverter Instance = new();

    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        value is AssetStatus s ? s switch
        {
            AssetStatus.Ok         => new SolidColorBrush(Color.Parse("#a6e3a1")), // green
            AssetStatus.Missing    => new SolidColorBrush(Color.Parse("#f38ba8")), // red
            AssetStatus.Damaged    => new SolidColorBrush(Color.Parse("#fab387")), // peach
            AssetStatus.Downloading=> new SolidColorBrush(Color.Parse("#89b4fa")), // blue
            AssetStatus.Failed     => new SolidColorBrush(Color.Parse("#f38ba8")), // red
            _                      => new SolidColorBrush(Color.Parse("#cdd6f4"))  // text
        } : new SolidColorBrush(Color.Parse("#cdd6f4"));

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        throw new NotSupportedException();
}
