using Avalonia.Controls;
using Avalonia.Interactivity;

namespace SevenHinos.Views;

public partial class UpdatePromptWindow : Window
{
    private bool _accepted;

    public UpdatePromptWindow()
    {
        InitializeComponent();
    }

    public async Task<bool> ShowDialogAsync(Window owner, string currentVersion, string latestVersion)
    {
        _accepted = false;
        MessageText.Text =
            $"Versão atual: {currentVersion}  |  Nova versão: {latestVersion}\n\nDeseja baixar e instalar agora?";

        await ShowDialog(owner);
        return _accepted;
    }

    private void OnUpdateClick(object? sender, RoutedEventArgs e)
    {
        _accepted = true;
        Close();
    }

    private void OnLaterClick(object? sender, RoutedEventArgs e)
    {
        _accepted = false;
        Close();
    }
}
