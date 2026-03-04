using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SevenHinos.Services;

namespace SevenHinos.ViewModels;

public sealed partial class ImportViewModel : ViewModelBase
{
    private readonly ILouvorJaImportService _importer;

    [ObservableProperty] private string  _dbPath        = ILouvorJaImportService.DefaultDbPath;
    [ObservableProperty] private string  _musicasFolder = ILouvorJaImportService.DefaultMusicasFolder;
    [ObservableProperty] private bool    _isImporting;
    [ObservableProperty] private bool    _isDone;
    [ObservableProperty] private double  _progress;        // 0–100
    [ObservableProperty] private string  _statusText  = string.Empty;
    [ObservableProperty] private string  _resultText  = string.Empty;
    [ObservableProperty] private bool    _hasErrors;
    [ObservableProperty] private bool    _dbFound;

    public ObservableCollection<string> ErrorLines { get; } = [];

    public ImportViewModel(ILouvorJaImportService importer)
    {
        _importer = importer;
        RefreshDbFound();
    }

    partial void OnDbPathChanged(string value) => RefreshDbFound();

    private void RefreshDbFound() =>
        DbFound = _importer.IsAvailable(DbPath);

    // ── Commands ──────────────────────────────────────────────────────────────

    [RelayCommand(CanExecute = nameof(CanImport))]
    private async Task ImportAsync(CancellationToken ct)
    {
        IsImporting = true;
        IsDone      = false;
        Progress    = 0;
        StatusText  = "Iniciando importação...";
        ResultText  = string.Empty;
        ErrorLines.Clear();
        HasErrors = false;

        ImportCommand.NotifyCanExecuteChanged();

        try
        {
            var prog = new Progress<ImportProgress>(p =>
            {
                Progress   = p.Total > 0 ? (double)p.Done / p.Total * 100.0 : 0;
                StatusText = $"[{p.Done}/{p.Total}]  {p.CurrentSong}";
            });

            var result = await _importer.ImportAsync(DbPath, MusicasFolder, prog, ct);

            Progress   = 100;
            ResultText = $"✓ {result.Imported} importadas   •   {result.Skipped} já existiam   •   {result.Failed} erros";
            HasErrors  = result.Errors.Count > 0;
            StatusText = "Concluído.";
            IsDone     = true;

            foreach (var e in result.Errors)
                ErrorLines.Add(e);
        }
        catch (OperationCanceledException)
        {
            StatusText = "Importação cancelada.";
            Progress   = 0;
        }
        catch (Exception ex)
        {
            StatusText = $"Erro: {ex.Message}";
            HasErrors  = true;
            ErrorLines.Add(ex.ToString());
        }
        finally
        {
            IsImporting = false;
            ImportCommand.NotifyCanExecuteChanged();
        }
    }

    private bool CanImport => DbFound && !IsImporting;

    [RelayCommand]
    private void ResetResult()
    {
        IsDone     = false;
        ResultText = string.Empty;
        ErrorLines.Clear();
        HasErrors  = false;
        Progress   = 0;
        StatusText = string.Empty;
    }
}
