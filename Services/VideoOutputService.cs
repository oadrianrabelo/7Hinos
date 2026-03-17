using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Platform;
using Avalonia.Threading;
using LibVLCSharp.Shared;
using SevenHinos.Views;

namespace SevenHinos.Services;

public sealed class VideoOutputService : IVideoOutputService
{
    private sealed record OutputSession(VideoOutputWindow Window, MediaPlayer Player, bool HasAudio);

    private readonly LibVLC _libVlc;
    private readonly Dictionary<int, OutputSession> _sessions = [];

    public bool IsActive => _sessions.Count > 0;

    public event Action? OutputsStopped;

    public VideoOutputService()
    {
        Core.Initialize();
        _libVlc = new LibVLC(enableDebugLogs: false);
    }

    public async Task ShowVideoAsync(
        string filePath,
        IReadOnlyList<int> monitorIndices,
        CancellationToken ct = default)
    {
        if (string.IsNullOrWhiteSpace(filePath))
            throw new InvalidOperationException("Selecione um vídeo para exibir.");

        var absolutePath = Path.GetFullPath(filePath);
        if (!File.Exists(absolutePath))
            throw new FileNotFoundException("Arquivo de vídeo não encontrado.", absolutePath);

        var targets = monitorIndices
            .Where(i => i >= 0)
            .Distinct()
            .OrderBy(i => i)
            .ToList();

        if (targets.Count == 0)
            throw new InvalidOperationException("Selecione pelo menos uma tela para exibir o vídeo.");

        ct.ThrowIfCancellationRequested();
        await Dispatcher.UIThread.InvokeAsync(() => ShowInternal(absolutePath, targets));
    }

    public void StopAll()
    {
        if (Dispatcher.UIThread.CheckAccess())
            StopAllInternal();
        else
            Dispatcher.UIThread.Post(StopAllInternal);
    }

    public void ShowIdentificationWindow(int monitorIndex)
    {
        if (Dispatcher.UIThread.CheckAccess())
            ShowIdentificationInternal(monitorIndex);
        else
            Dispatcher.UIThread.Post(() => ShowIdentificationInternal(monitorIndex));
    }

    private void ShowIdentificationInternal(int monitorIndex)
    {
        var screens = GetScreens();
        if (monitorIndex < 0 || monitorIndex >= screens.Count)
            return;

        var screen = screens[monitorIndex];

        var window = new Window
        {
            Title = $"Monitor {monitorIndex + 1}",
            Width = screen.WorkingArea.Width,
            Height = screen.WorkingArea.Height,
            WindowStartupLocation = WindowStartupLocation.Manual,
            Background = new Avalonia.Media.SolidColorBrush(Avalonia.Media.Colors.Black),
            Content = new Canvas
            {
                Background = new Avalonia.Media.SolidColorBrush(Avalonia.Media.Colors.Black),
                Children =
                {
                    new TextBlock
                    {
                        Text = $"Tela {monitorIndex + 1}",
                        FontSize = 200,
                        FontWeight = Avalonia.Media.FontWeight.Bold,
                        Foreground = new Avalonia.Media.SolidColorBrush(Avalonia.Media.Colors.White),
                        TextAlignment = Avalonia.Media.TextAlignment.Center,
                        VerticalAlignment = Avalonia.Layout.VerticalAlignment.Center,
                        HorizontalAlignment = Avalonia.Layout.HorizontalAlignment.Center
                    }
                }
            }
        };

        window.Position = new Avalonia.PixelPoint(
            (int)screen.WorkingArea.X,
            (int)screen.WorkingArea.Y);

        window.KeyDown += (_, e) =>
        {
            if (e.Key == Avalonia.Input.Key.Escape)
                window.Close();
        };

        window.Show();

        // Auto-close after 5 seconds
        Dispatcher.UIThread.Post(() =>
        {
            Task.Delay(5000).ContinueWith(_ =>
            {
                Dispatcher.UIThread.Post(() =>
                {
                    try { window.Close(); } catch { }
                });
            });
        }, DispatcherPriority.Background);
    }

    private void ShowInternal(string absolutePath, IReadOnlyList<int> monitorIndices)
    {
        StopAllInternal();

        var screens = GetScreens();
        if (screens.Count == 0)
            throw new InvalidOperationException("Nenhum monitor disponível para saída de vídeo.");

        var audioAssigned = false;
        var orderedSessions = new List<OutputSession>(monitorIndices.Count);

        foreach (var index in monitorIndices)
        {
            if (index < 0 || index >= screens.Count)
                continue;

            var screen = screens[index];
            var hasAudio = !audioAssigned;

            var player = new MediaPlayer(_libVlc)
            {
                EnableHardwareDecoding = true,
                Mute = !hasAudio
            };

            if (hasAudio)
                player.Volume = 100;

            var window = new VideoOutputWindow
            {
                ShowInTaskbar = false,
                CanResize = false,
                SystemDecorations = SystemDecorations.None
            };

            window.PlaceOnScreen(screen);
            window.Show();
            window.AttachPlayer(player);
            window.EscapePressed += OnWindowEscapePressed;

            var session = new OutputSession(window, player, hasAudio);
            _sessions[index] = session;
            orderedSessions.Add(session);
            audioAssigned = true;
        }

        if (_sessions.Count == 0)
            throw new InvalidOperationException("As telas selecionadas não estão disponíveis no momento.");

        // Start silent outputs first, then the audio source.
        foreach (var session in orderedSessions.Where(s => !s.HasAudio))
            PlaySession(session, absolutePath);

        var audioSession = orderedSessions.FirstOrDefault(s => s.HasAudio);
        if (audioSession is not null)
            PlaySession(audioSession, absolutePath);
    }

    private void PlaySession(OutputSession session, string absolutePath)
    {
        using var media = new Media(_libVlc, new Uri(absolutePath));

        // Secondary outputs should never compete for the audio device.
        if (!session.HasAudio)
            media.AddOption(":no-audio");

        session.Player.Play(media);

        if (session.HasAudio)
        {
            session.Player.Mute = false;
            session.Player.Volume = 100;
        }
    }

    private void OnWindowEscapePressed(object? sender, EventArgs e)
    {
        StopAll();
    }

    private static IReadOnlyList<Screen> GetScreens()
    {
        if (Application.Current?.ApplicationLifetime is not IClassicDesktopStyleApplicationLifetime desktop
            || desktop.MainWindow is null)
        {
            return [];
        }

        return desktop.MainWindow.Screens.All;
    }

    private void StopAllInternal()
    {
        if (_sessions.Count == 0)
            return;

        var sessions = _sessions.Values.ToList();
        _sessions.Clear();

        foreach (var session in sessions)
        {
            session.Window.EscapePressed -= OnWindowEscapePressed;

            // Detach the native video surface before closing/disposal to avoid
            // LibVLC access violations in VideoView.Detach on Windows.
            try { session.Window.DetachPlayer(); } catch { }
            try { session.Player.Stop(); } catch { }
        }

        foreach (var session in sessions)
        {
            void DisposePlayer()
            {
                try { session.Player.Dispose(); } catch { }
            }

            void OnWindowClosed(object? sender, EventArgs e)
            {
                session.Window.Closed -= OnWindowClosed;
                DisposePlayer();
            }

            if (!session.Window.IsVisible)
            {
                DisposePlayer();
                continue;
            }

            try
            {
                session.Window.Closed += OnWindowClosed;
                session.Window.Close();
            }
            catch
            {
                session.Window.Closed -= OnWindowClosed;
                DisposePlayer();
            }
        }

        OutputsStopped?.Invoke();
    }

    public void Dispose()
    {
        if (Dispatcher.UIThread.CheckAccess())
            StopAllInternal();
        else
            Dispatcher.UIThread.InvokeAsync(StopAllInternal).GetAwaiter().GetResult();

        _libVlc.Dispose();
    }
}
