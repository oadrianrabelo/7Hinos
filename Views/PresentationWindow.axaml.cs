using Avalonia.Controls;
using Microsoft.Extensions.DependencyInjection;
using SevenHinos.ViewModels;

namespace SevenHinos.Views;

public partial class PresentationWindow : Window
{
    public PresentationWindow()
    {
        InitializeComponent();
        // PresentationViewModel is a singleton — share state across the app.
        DataContext = App.Services.GetRequiredService<PresentationViewModel>();
    }
}
