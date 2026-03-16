using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using Avalonia.VisualTree;
using SevenHinos.ViewModels;

namespace SevenHinos.Views;

public partial class VideosView : UserControl
{
    public VideosView()
    {
        InitializeComponent();
        DataContextChanged += (_, _) => RefreshMonitors();
    }

    protected override void OnAttachedToVisualTree(VisualTreeAttachmentEventArgs e)
    {
        base.OnAttachedToVisualTree(e);
        RefreshMonitors();
    }

    private void RefreshMonitors()
    {
        if (DataContext is VideosViewModel vm
            && this.GetVisualRoot() is Window window)
        {
            vm.RefreshMonitors(window.Screens.All);
        }
    }

    private async void OnAddVideoClick(object? sender, RoutedEventArgs e)
    {
        if (TopLevel.GetTopLevel(this) is not TopLevel topLevel
            || DataContext is not VideosViewModel vm)
        {
            return;
        }

        var files = await topLevel.StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions
        {
            Title = "Selecionar vídeo",
            AllowMultiple = false,
            FileTypeFilter =
            [
                new FilePickerFileType("Arquivos de vídeo")
                {
                    Patterns = ["*.mp4", "*.mkv", "*.mov", "*.avi", "*.wmv", "*.webm", "*.m4v"]
                },
                FilePickerFileTypes.All
            ]
        });

        var picked = files.FirstOrDefault();
        if (picked is null)
            return;

        var localPath = picked.TryGetLocalPath();
        if (!string.IsNullOrWhiteSpace(localPath))
            await vm.AddVideoAsync(localPath);
    }

    private void OnRefreshMonitorsClick(object? sender, RoutedEventArgs e)
    {
        RefreshMonitors();
    }
}
