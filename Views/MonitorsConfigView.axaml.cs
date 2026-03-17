using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.VisualTree;
using SevenHinos.ViewModels;

namespace SevenHinos.Views;

public partial class MonitorsConfigView : UserControl
{
    public MonitorsConfigView()
    {
        InitializeComponent();
        DataContextChanged += async (_, _) => await OnDataContextChangedAsync();
    }

    protected override void OnAttachedToVisualTree(VisualTreeAttachmentEventArgs e)
    {
        base.OnAttachedToVisualTree(e);
        DataContextChanged += async (_, _) => await OnDataContextChangedAsync();
    }

    private async Task OnDataContextChangedAsync()
    {
        if (DataContext is MonitorsConfigViewModel vm
            && this.GetVisualRoot() is Window window)
        {
            await vm.LoadMonitorsAsync(window.Screens.All);
        }
    }
}
