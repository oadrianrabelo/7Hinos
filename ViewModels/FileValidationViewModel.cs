using System;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SevenHinos.Models;
using SevenHinos.Services;

namespace SevenHinos.ViewModels;

// ─────────────────────────────────────────────────────────────────────────────
// Per-file display model
// ─────────────────────────────────────────────────────────────────────────────

public sealed partial class AssetItemViewModel : ObservableObject
{
    public int Id { get; }
    public string FileName { get; }
    public string? Category { get; }
    public string RelativePath { get; }
    public long ExpectedSize { get; }

    [ObservableProperty] private AssetStatus _status;
    [ObservableProperty] private double _downloadProgress; // 0–1
    [ObservableProperty] private string _statusText = string.Empty;

    public AssetItemViewModel(FileAsset asset)
    {
        Id = asset.Id;
        FileName = asset.FileName;
        Category = asset.Category;
        RelativePath = asset.RelativePath;
        ExpectedSize = asset.ExpectedSize;
        ApplyStatus(asset);
    }

    internal void ApplyStatus(FileAsset asset)
    {
        if (asset.IsVerified)
        {
            Status = AssetStatus.Ok;
            StatusText = "OK";
        }
        else if (!string.IsNullOrEmpty(asset.LocalPath) && File.Exists(asset.LocalPath))
        {
            Status = AssetStatus.Damaged;
            StatusText = "Danificado";
        }
        else
        {
            Status = AssetStatus.Missing;
            StatusText = "Faltando";
        }
    }
}

public enum AssetStatus { Unknown, Ok, Missing, Damaged, Downloading, Failed }

// ─────────────────────────────────────────────────────────────────────────────
// Page ViewModel
// ─────────────────────────────────────────────────────────────────────────────

public sealed partial class FileValidationViewModel : ViewModelBase
{
    // Configured at the application level.  Can be overridden from settings.
    public const string DefaultManifestUrl =
        "https://objectstorage.sa-saopaulo-1.oraclecloud.com/n/greafinzbo0u/b/7hinos-asset/o/manifest.json";

    private readonly IFileAssetService _assetService;
    private CancellationTokenSource? _cts;

    // ── Observable state ────────────────────────────────────────────────────

    [ObservableProperty] private string _manifestUrl = DefaultManifestUrl;
    [ObservableProperty] private bool _isBusy;
    [ObservableProperty] private string _statusMessage = "Pronto.";
    [ObservableProperty] private double _overallProgress; // 0–1
    [ObservableProperty] private bool _canCancel;
    [ObservableProperty] private int _missingCount;
    [ObservableProperty] private int _okCount;

    public ObservableCollection<AssetItemViewModel> Assets { get; } = [];

    // ── Local root for downloads ──────────────────────────────────────────
    private static string LocalAssetsRoot =>
        Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
            "7Hinos", "assets");

    public FileValidationViewModel(IFileAssetService assetService)
    {
        _assetService = assetService;
    }

    // ── Commands ─────────────────────────────────────────────────────────────

    /// <summary>Loads the manifest from the configured URL and syncs the local DB.</summary>
    [RelayCommand(CanExecute = nameof(IsNotBusy))]
    private async Task FetchManifestAsync()
    {
        await RunOperationAsync(async ct =>
        {
            StatusMessage = "Buscando manifesto...";
            var manifest = await _assetService.FetchManifestAsync(ManifestUrl, ct);
            StatusMessage = $"Sincronizando manifesto v{manifest.Version}...";
            await _assetService.SyncManifestAsync(manifest, ct);
            await ReloadListAsync(ct);
            StatusMessage = $"Manifesto v{manifest.Version} sincronizado. {Assets.Count} arquivo(s) encontrado(s).";
        });
    }

    /// <summary>Verifies every asset file on disk without downloading.</summary>
    [RelayCommand(CanExecute = nameof(IsNotBusy))]
    private async Task ValidateFilesAsync()
    {
        await RunOperationAsync(async ct =>
        {
            await ReloadListAsync(ct);

            var progress = new Progress<AssetValidationProgress>(p =>
            {
                var vm = Assets.FirstOrDefault(a => a.FileName == p.FileName);
                if (vm is not null)
                {
                    vm.Status = p.IsValid ? AssetStatus.Ok : AssetStatus.Missing;
                    vm.StatusText = p.IsValid ? "OK" : "Faltando";
                }

                OverallProgress = (double)p.Current / p.Total;
                StatusMessage = $"Verificando {p.Current}/{p.Total}: {p.FileName}";
            });

            await _assetService.ValidateLocalFilesAsync(progress, ct);
            await ReloadListAsync(ct);
            UpdateCounts();
            StatusMessage = $"Verificação concluída — {OkCount} OK, {MissingCount} faltando.";
        });
    }

    /// <summary>Downloads only the assets that are missing or damaged.</summary>
    [RelayCommand(CanExecute = nameof(IsNotBusy))]
    private async Task DownloadMissingAsync()
    {
        var missing = Assets
            .Where(a => a.Status is AssetStatus.Missing or AssetStatus.Damaged or AssetStatus.Failed)
            .ToList();

        if (missing.Count == 0)
        {
            StatusMessage = "Nenhum arquivo para baixar.";
            return;
        }

        await DownloadItemsAsync(missing);
    }

    /// <summary>Downloads ALL assets, replacing any existing files.</summary>
    [RelayCommand(CanExecute = nameof(IsNotBusy))]
    private async Task DownloadAllAsync()
    {
        if (Assets.Count == 0)
        {
            StatusMessage = "Nenhum arquivo no manifesto. Busque o manifesto primeiro.";
            return;
        }

        await DownloadItemsAsync(Assets.ToList());
    }

    [RelayCommand(CanExecute = nameof(CanCancel))]
    private void Cancel()
    {
        _cts?.Cancel();
        StatusMessage = "Operação cancelada pelo usuário.";
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    private bool IsNotBusy => !IsBusy;

    private async Task DownloadItemsAsync(System.Collections.Generic.List<AssetItemViewModel> items)
    {
        // Resolve FileAsset entities from DB for the selected items.
        var allAssets = await _assetService.GetAllAssetsAsync();
        var idSet = items.Select(i => i.Id).ToHashSet();
        var toDownload = allAssets.Where(a => idSet.Contains(a.Id)).ToList();

        await RunOperationAsync(async ct =>
        {
            // Mark them all as "downloading" in the UI.
            foreach (var vm in items)
            {
                vm.Status = AssetStatus.Downloading;
                vm.StatusText = "Aguardando...";
                vm.DownloadProgress = 0;
            }

            var progress = new Progress<AssetDownloadProgress>(p =>
            {
                var vm = items.FirstOrDefault(a => a.FileName == p.FileName);
                if (vm is null) return;

                if (p.FileFailed)
                {
                    vm.Status = AssetStatus.Failed;
                    vm.StatusText = "Falha";
                    vm.DownloadProgress = 0;
                }
                else if (p.FileCompleted)
                {
                    vm.Status = AssetStatus.Ok;
                    vm.StatusText = "OK";
                    vm.DownloadProgress = 1;
                }
                else if (p.TotalBytes > 0)
                {
                    double fraction = (double)p.BytesReceived / p.TotalBytes;
                    vm.DownloadProgress = fraction;
                    vm.StatusText = $"{p.BytesReceived / 1024} KB / {p.TotalBytes / 1024} KB";
                }

                OverallProgress = (double)(p.FileIndex - 1 + vm.DownloadProgress) / p.FileCount;
                StatusMessage = $"Baixando {p.FileIndex}/{p.FileCount}: {p.FileName}";
            });

            await _assetService.DownloadAssetsAsync(toDownload, LocalAssetsRoot, progress, ct);
            UpdateCounts();
            StatusMessage = $"Download concluído — {OkCount} OK, {MissingCount} faltando.";
        });
    }

    private async Task RunOperationAsync(Func<CancellationToken, Task> work)
    {
        _cts = new CancellationTokenSource();
        IsBusy = true;
        CanCancel = true;
        OverallProgress = 0;
        FetchManifestCommand.NotifyCanExecuteChanged();
        ValidateFilesCommand.NotifyCanExecuteChanged();
        DownloadMissingCommand.NotifyCanExecuteChanged();
        DownloadAllCommand.NotifyCanExecuteChanged();

        try
        {
            await work(_cts.Token);
        }
        catch (OperationCanceledException)
        {
            StatusMessage = "Operação cancelada.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"Erro: {ex.Message}";
        }
        finally
        {
            IsBusy = false;
            CanCancel = false;
            OverallProgress = IsBusy ? OverallProgress : 0;
            FetchManifestCommand.NotifyCanExecuteChanged();
            ValidateFilesCommand.NotifyCanExecuteChanged();
            DownloadMissingCommand.NotifyCanExecuteChanged();
            DownloadAllCommand.NotifyCanExecuteChanged();
            _cts?.Dispose();
            _cts = null;
        }
    }

    private async Task ReloadListAsync(CancellationToken ct = default)
    {
        var assets = await _assetService.GetAllAssetsAsync(ct);
        await Dispatcher.UIThread.InvokeAsync(() =>
        {
            Assets.Clear();
            foreach (var a in assets)
                Assets.Add(new AssetItemViewModel(a));
        });
        UpdateCounts();
    }

    private void UpdateCounts()
    {
        OkCount = Assets.Count(a => a.Status == AssetStatus.Ok);
        MissingCount = Assets.Count(a => a.Status is AssetStatus.Missing or AssetStatus.Damaged or AssetStatus.Failed);
    }
}
