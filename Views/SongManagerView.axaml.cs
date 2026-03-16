using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Microsoft.Extensions.DependencyInjection;
using SevenHinos.ViewModels;

namespace SevenHinos.Views;

public partial class SongManagerView : UserControl
{
    public SongManagerView()
    {
        InitializeComponent();
    }

    private async void OnImportHymnsClick(object? sender, RoutedEventArgs e)
    {
        if (TopLevel.GetTopLevel(this) is not Window owner)
            return;

        var vm = App.Services.GetRequiredService<ImportViewModel>();
        var dialog = new ImportWindow
        {
            DataContext = vm
        };

        await dialog.ShowDialog(owner);

        if (vm.HasImportedChanges && DataContext is SongManagerViewModel songManager)
            await songManager.ReloadAsync();
    }
}
