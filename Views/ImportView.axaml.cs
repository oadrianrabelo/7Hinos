using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using SevenHinos.ViewModels;

namespace SevenHinos.Views;

public partial class ImportView : UserControl
{
    public ImportView()
    {
        InitializeComponent();
    }

    private async void OnBrowseLouvorJaFolderClick(object? sender, RoutedEventArgs e)
    {
        if (TopLevel.GetTopLevel(this) is not TopLevel topLevel
            || DataContext is not ImportViewModel vm)
        {
            return;
        }

        var folders = await topLevel.StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions
        {
            Title = "Selecionar pasta do LouvorJA",
            AllowMultiple = false
        });

        var picked = folders.FirstOrDefault();
        var localPath = picked?.TryGetLocalPath();
        if (!string.IsNullOrWhiteSpace(localPath))
            vm.SetLouvorJaFolder(localPath);
    }

    private void OnCloseClick(object? sender, RoutedEventArgs e)
    {
        if (TopLevel.GetTopLevel(this) is Window window)
            window.Close();
    }
}
