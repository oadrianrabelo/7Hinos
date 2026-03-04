using Avalonia.Controls;
using Microsoft.Extensions.DependencyInjection;
using SevenHinos.ViewModels;

namespace SevenHinos.Views;

public partial class PublicOutputWindow : Window
{
    public PublicOutputWindow()
    {
        InitializeComponent();
        DataContext = App.Services.GetRequiredService<PresentationState>();
    }
}
