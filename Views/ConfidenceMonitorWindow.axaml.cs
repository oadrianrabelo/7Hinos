using Avalonia.Controls;
using Microsoft.Extensions.DependencyInjection;
using SevenHinos.ViewModels;

namespace SevenHinos.Views;

public partial class ConfidenceMonitorWindow : Window
{
    public ConfidenceMonitorWindow()
    {
        InitializeComponent();
        DataContext = App.Services.GetRequiredService<PresentationState>();
    }
}
