using System.IO;
using System.Collections.ObjectModel;
using System.Linq;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SevenHinos.Services;

namespace SevenHinos.ViewModels;

public enum HymnImportSource
{
    NativeLibrary,
    LouvorJaInstallation
}

public sealed partial class ImportViewModel : ViewModelBase
{
    private readonly ILouvorJaImportService _louvorJaImporter;
    private readonly INativeHymnImportService _nativeImporter;

    [ObservableProperty] private HymnImportSource _selectedSource;
    [ObservableProperty] private string _louvorJaFolder = string.Empty;
    [ObservableProperty] private string _louvorJaDbPath = string.Empty;
    [ObservableProperty] private string _louvorJaMusicasFolder = string.Empty;
    [ObservableProperty] private string _nativeLibraryPath = string.Empty;
    [ObservableProperty] private bool _louvorJaAvailable;
    [ObservableProperty] private bool _nativeLibraryAvailable;
    [ObservableProperty] private bool _isImporting;
    [ObservableProperty] private bool _isDone;
    [ObservableProperty] private double _progress;
    [ObservableProperty] private string _statusText = "Selecione a origem dos hinos.";
    [ObservableProperty] private string _resultText = string.Empty;
    [ObservableProperty] private bool _hasErrors;
    [ObservableProperty] private bool _hasImportedChanges;

    public ObservableCollection<string> ErrorLines { get; } = [];

    public bool IsNativeLibrarySelected => SelectedSource == HymnImportSource.NativeLibrary;
    public bool IsLouvorJaSelected => SelectedSource == HymnImportSource.LouvorJaInstallation;
    public bool CurrentSourceAvailable =>
        IsNativeLibrarySelected ? NativeLibraryAvailable : LouvorJaAvailable;

    public string SelectedSourceTitle => IsNativeLibrarySelected
        ? "Biblioteca nativa do 7Hinos"
        : "Instalação local do LouvorJA";

    public string SelectedSourceDescription => IsNativeLibrarySelected
        ? "Importa o catálogo offline incluído com o 7Hinos. Quando os áudios locais já existirem, eles são vinculados automaticamente ao hino importado."
        : "Importa músicas, letras, slides e caminhos de áudio da instalação local do LouvorJA. O processo funciona totalmente offline.";

    public string SourceAvailabilityText => IsNativeLibrarySelected
        ? (NativeLibraryAvailable
            ? "Biblioteca nativa encontrada no pacote do aplicativo."
            : "Biblioteca nativa não encontrada no pacote atual.")
        : (LouvorJaAvailable
            ? "Instalação LouvorJA encontrada e pronta para importar."
            : "Selecione a pasta do LouvorJA ou use a detecção automática.");

    public ImportViewModel(
        ILouvorJaImportService louvorJaImporter,
        INativeHymnImportService nativeImporter)
    {
        _louvorJaImporter = louvorJaImporter;
        _nativeImporter = nativeImporter;

        RefreshNativeLibraryAvailability();
        AutoDetectLouvorJa();

        SelectedSource = NativeLibraryAvailable
            ? HymnImportSource.NativeLibrary
            : HymnImportSource.LouvorJaInstallation;

        NotifySourceChanged();
    }

    partial void OnSelectedSourceChanged(HymnImportSource value) => NotifySourceChanged();

    partial void OnLouvorJaFolderChanged(string value)
    {
        RefreshLouvorJaAvailability();
        ImportCommand.NotifyCanExecuteChanged();
    }

    public void SetLouvorJaFolder(string folderPath)
    {
        LouvorJaFolder = (folderPath ?? string.Empty).Trim();
    }

    private void RefreshNativeLibraryAvailability()
    {
        NativeLibraryPath = _nativeImporter.DefaultLibraryPath;
        NativeLibraryAvailable = _nativeImporter.IsAvailable();
    }

    private void RefreshLouvorJaAvailability()
    {
        var resolved = ResolveLouvorJaInstallation(LouvorJaFolder);
        LouvorJaDbPath = resolved.DbPath;
        LouvorJaMusicasFolder = resolved.MusicasFolder;
        LouvorJaAvailable = resolved.IsAvailable;
        OnPropertyChanged(nameof(CurrentSourceAvailable));
        OnPropertyChanged(nameof(SourceAvailabilityText));
    }

    private void NotifySourceChanged()
    {
        OnPropertyChanged(nameof(IsNativeLibrarySelected));
        OnPropertyChanged(nameof(IsLouvorJaSelected));
        OnPropertyChanged(nameof(CurrentSourceAvailable));
        OnPropertyChanged(nameof(SelectedSourceTitle));
        OnPropertyChanged(nameof(SelectedSourceDescription));
        OnPropertyChanged(nameof(SourceAvailabilityText));
        ImportCommand.NotifyCanExecuteChanged();
    }

    // ── Commands ──────────────────────────────────────────────────────────────

    [RelayCommand]
    private void SelectNativeLibrarySource()
    {
        SelectedSource = HymnImportSource.NativeLibrary;
    }

    [RelayCommand]
    private void SelectLouvorJaSource()
    {
        SelectedSource = HymnImportSource.LouvorJaInstallation;
    }

    [RelayCommand]
    private void AutoDetectLouvorJa()
    {
        var detected = TryDetectLouvorJaFolder();
        LouvorJaFolder = string.IsNullOrWhiteSpace(detected)
            ? GetDefaultLouvorJaFolder()
            : detected;

        StatusText = LouvorJaAvailable
            ? $"LouvorJA detectado em: {LouvorJaFolder}"
            : "Nenhuma instalação do LouvorJA foi detectada automaticamente.";
    }

    [RelayCommand(CanExecute = nameof(CanImport))]
    private async Task ImportAsync(CancellationToken ct)
    {
        ResetResultCore();
        IsImporting = true;
        Progress = 0;
        StatusText = "Iniciando importação...";

        ImportCommand.NotifyCanExecuteChanged();

        try
        {
            var prog = new Progress<ImportProgress>(p =>
            {
                Progress   = p.Total > 0 ? (double)p.Done / p.Total * 100.0 : 0;
                StatusText = $"[{p.Done}/{p.Total}]  {p.CurrentSong}";
            });

            var result = SelectedSource switch
            {
                HymnImportSource.NativeLibrary =>
                    await _nativeImporter.ImportAsync(prog, ct),
                HymnImportSource.LouvorJaInstallation =>
                    await _louvorJaImporter.ImportAsync(LouvorJaDbPath, LouvorJaMusicasFolder, prog, ct),
                _ => throw new InvalidOperationException("Origem de importação inválida.")
            };

            Progress = 100;
            ResultText = $"✓ {result.Imported} importadas   •   {result.Skipped} já existiam   •   {result.Failed} erros";
            HasErrors = result.Errors.Count > 0;
            StatusText = "Concluído.";
            IsDone = true;
            HasImportedChanges = result.Imported > 0;

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

    private bool CanImport => CurrentSourceAvailable && !IsImporting;

    [RelayCommand]
    private void ResetResult()
    {
        ResetResultCore();
        StatusText = "Selecione a origem dos hinos.";
    }

    private void ResetResultCore()
    {
        IsDone = false;
        ResultText = string.Empty;
        ErrorLines.Clear();
        HasErrors = false;
        HasImportedChanges = false;
        Progress = 0;
    }

    private static string GetDefaultLouvorJaFolder()
    {
        var configFolder = Path.GetDirectoryName(ILouvorJaImportService.DefaultDbPath) ?? string.Empty;
        return Directory.GetParent(configFolder)?.FullName ?? configFolder;
    }

    private static string? TryDetectLouvorJaFolder()
    {
        var roots = new[]
        {
            Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86),
            Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles)
        };

        foreach (var root in roots.Where(r => !string.IsNullOrWhiteSpace(r)))
        {
            foreach (var folderName in new[] { "Louvor JA", "LouvorJA" })
            {
                var candidate = Path.Combine(root, folderName);
                if (ResolveLouvorJaInstallation(candidate).IsAvailable)
                    return candidate;
            }
        }

        return null;
    }

    private static (string DbPath, string MusicasFolder, bool IsAvailable) ResolveLouvorJaInstallation(string folderPath)
    {
        if (string.IsNullOrWhiteSpace(folderPath))
            return (string.Empty, string.Empty, false);

        var absoluteFolder = Path.GetFullPath(folderPath.Trim());
        var candidateFolders = new[]
        {
            absoluteFolder,
            Path.Combine(absoluteFolder, "config")
        };

        foreach (var candidate in candidateFolders.Distinct(StringComparer.OrdinalIgnoreCase))
        {
            var dbPath = Path.Combine(candidate, "database.db");
            var musicasFolder = Path.Combine(candidate, "musicas");
            if (File.Exists(dbPath))
                return (dbPath, musicasFolder, true);
        }

        return (
            Path.Combine(absoluteFolder, "config", "database.db"),
            Path.Combine(absoluteFolder, "config", "musicas"),
            false);
    }
}
