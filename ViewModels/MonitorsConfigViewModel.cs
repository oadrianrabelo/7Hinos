using Avalonia.Platform;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SevenHinos.Services;

namespace SevenHinos.ViewModels;

public sealed record MonitorAccessViewModel(
    int Index,
    string Label,
    string CustomName);

public sealed partial class MonitorsConfigViewModel : ViewModelBase
{
    private readonly IMonitorDeviceService _monitorDeviceService;
    private readonly IVideoOutputService _videoOutputService;

    [ObservableProperty] private bool _isIdentifyingMonitors;
    [ObservableProperty] private string _statusText = "Pronto para gerenciar monitores.";
    [ObservableProperty] private List<MonitorAccessViewModel> _monitors = [];

    public IAsyncRelayCommand StartMonitorIdentificationCommand { get; }
    public IAsyncRelayCommand StopMonitorIdentificationCommand { get; }

    public MonitorsConfigViewModel(
        IMonitorDeviceService monitorDeviceService,
        IVideoOutputService videoOutputService)
    {
        _monitorDeviceService = monitorDeviceService;
        _videoOutputService = videoOutputService;

        StartMonitorIdentificationCommand = new AsyncRelayCommand(StartMonitorIdentificationAsync);
        StopMonitorIdentificationCommand = new AsyncRelayCommand(StopMonitorIdentificationAsync);
    }

    public async Task LoadMonitorsAsync(IReadOnlyList<Screen> screens)
    {
        try
        {
            var monitors = new List<MonitorAccessViewModel>();

            foreach (var (index, screen) in screens.Select((s, i) => (i, s)))
            {
                // Ensure monitor exists in database
                await _monitorDeviceService.EnsureMonitorExistsAsync(index);

                var customName = await _monitorDeviceService.GetMonitorNameAsync(index);
                var label = $"{screen.WorkingArea.Width}×{screen.WorkingArea.Height}";

                monitors.Add(new MonitorAccessViewModel(
                    Index: index,
                    Label: label,
                    CustomName: customName));
            }

            Monitors = monitors;
            StatusText = $"{monitors.Count} tela(s) detectada(s)";
        }
        catch (Exception ex)
        {
            StatusText = $"Erro ao carregar monitores: {ex.Message}";
        }
    }

    public async Task UpdateMonitorNameAsync(int monitorIndex, string newName)
    {
        try
        {
            await _monitorDeviceService.SetMonitorNameAsync(monitorIndex, newName);

            // Update the local monitor list with the new name
            var monitor = Monitors.FirstOrDefault(m => m.Index == monitorIndex);
            if (monitor != null)
            {
                var updatedMonitors = Monitors.ToList();
                var index = updatedMonitors.IndexOf(monitor);
                updatedMonitors[index] = monitor with { CustomName = newName };
                Monitors = updatedMonitors;
            }
            
            StatusText = $"Nome do monitor {monitorIndex + 1} atualizado.";
        }
        catch (Exception ex)
        {
            StatusText = $"Erro ao salvar nome: {ex.Message}";
        }
    }

    private async Task StartMonitorIdentificationAsync()
    {
        try
        {
            IsIdentifyingMonitors = true;
            StatusText = "Exibindo número em cada monitor... Clique em 'Parar identificação' para encerrar.";

            foreach (var monitor in Monitors)
            {
                _videoOutputService.ShowIdentificationWindow(monitor.Index);
                await Task.Delay(500); // Small delay between showing screens
            }
        }
        catch (Exception ex)
        {
            StatusText = $"Erro durante identificação: {ex.Message}";
            IsIdentifyingMonitors = false;
        }
    }

    private Task StopMonitorIdentificationAsync()
    {
        try
        {
            _videoOutputService.StopAll();
            IsIdentifyingMonitors = false;
            StatusText = "Identificação interrompida.";
        }
        catch (Exception ex)
        {
            StatusText = $"Erro ao parar: {ex.Message}";
        }

        return Task.CompletedTask;
    }
}
