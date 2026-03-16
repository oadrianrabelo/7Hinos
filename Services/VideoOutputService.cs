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
    private sealed record OutputSession(VideoOutputWindow Window, MediaPlayer Player);

    private readonly LibVLC _libVlc;
    private readonly Dictionary<int, OutputSession> _sessions = [];

    public bool IsActive => _sessions.Count > 0;

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

    private void ShowInternal(string absolutePath, IReadOnlyList<int> monitorIndices)
    {
        StopAllInternal();

        var screens = GetScreens();
        if (screens.Count == 0)
            throw new InvalidOperationException("Nenhum monitor disponível para saída de vídeo.");

        var unmutedAssigned = false;

        foreach (var index in monitorIndices)
        {
            if (index < 0 || index >= screens.Count)
                continue;

            var screen = screens[index];
            var player = new MediaPlayer(_libVlc)
            {
                EnableHardwareDecoding = true,
                Mute = unmutedAssigned
            };

            if (!unmutedAssigned)
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

            _sessions[index] = new OutputSession(window, player);
            unmutedAssigned = true;
        }

        if (_sessions.Count == 0)
            throw new InvalidOperationException("As telas selecionadas não estão disponíveis no momento.");

        foreach (var session in _sessions.Values)
        {
            using var media = new Media(_libVlc, new Uri(absolutePath));
            session.Player.Play(media);
        }
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
