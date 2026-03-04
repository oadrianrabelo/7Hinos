using System;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using SevenHinos.ViewModels;

namespace SevenHinos.Views;

public partial class MainWindow : Window
{
    private MainWindowViewModel? _vm;

    public MainWindow()
    {
        InitializeComponent();
        DataContextChanged += OnDataContextChanged;
    }

    private void OnDataContextChanged(object? sender, EventArgs e)
    {
        if (_vm is not null)
            _vm.OpenPresentationRequested -= OnOpenPresentation;

        _vm = DataContext as MainWindowViewModel;

        if (_vm is not null)
            _vm.OpenPresentationRequested += OnOpenPresentation;
    }

    private void OnOpenPresentation()
    {
        if (Application.Current?.ApplicationLifetime is not IClassicDesktopStyleApplicationLifetime desktop)
            return;

        // Bring existing window to front, or open a new one
        var existing = desktop.Windows.OfType<PresentationWindow>().FirstOrDefault();
        if (existing is not null)
        {
            existing.Show();
            existing.Activate();
            return;
        }

        new PresentationWindow().Show();
    }
}
