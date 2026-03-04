using System.ComponentModel;
using Avalonia;
using Avalonia.Controls;
using Avalonia.VisualTree;
using SevenHinos.ViewModels;

namespace SevenHinos.Views;

public partial class PresentationView : UserControl
{
    private PresentationViewModel? _vm;

    public PresentationView()
    {
        InitializeComponent();
        DataContextChanged += OnDataContextChanged;
    }

    private void OnDataContextChanged(object? sender, EventArgs e)
    {
        if (_vm is not null)
            _vm.PropertyChanged -= OnVmPropertyChanged;

        _vm = DataContext as PresentationViewModel;

        if (_vm is not null)
            _vm.PropertyChanged += OnVmPropertyChanged;
    }

    // Auto-scroll the slide list to keep the active slide visible
    private void OnVmPropertyChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(PresentationViewModel.SelectedSlide)
            && SlideList.SelectedItem is not null)
        {
            SlideList.ScrollIntoView(SlideList.SelectedItem);
        }
    }

    // ── Called when the view is placed inside the visual tree (Window ready) ──

    protected override void OnAttachedToVisualTree(VisualTreeAttachmentEventArgs e)
    {
        base.OnAttachedToVisualTree(e);

        if (DataContext is PresentationViewModel vm
            && this.GetVisualRoot() is Window window)
        {
            vm.InitScreens(window.Screens.All);
        }
    }
}
