using System;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Platform;
using LibVLCSharp.Shared;

namespace SevenHinos.Views;

public partial class VideoOutputWindow : Window
{
    public VideoOutputWindow()
    {
        InitializeComponent();
    }

    public void AttachPlayer(MediaPlayer? mediaPlayer)
    {
        VideoSurface.MediaPlayer = mediaPlayer;
    }

    public void DetachPlayer()
    {
        VideoSurface.MediaPlayer = null;
    }

    public void PlaceOnScreen(Screen screen)
    {
        var scaling = screen.Scaling <= 0 ? 1.0 : screen.Scaling;

        WindowStartupLocation = WindowStartupLocation.Manual;
        WindowState = WindowState.Normal;
        Position = new PixelPoint(screen.Bounds.X, screen.Bounds.Y);
        Width = screen.Bounds.Width / scaling;
        Height = screen.Bounds.Height / scaling;
        Topmost = true;
    }

    protected override void OnClosed(EventArgs e)
    {
        DetachPlayer();
        base.OnClosed(e);
    }
}
