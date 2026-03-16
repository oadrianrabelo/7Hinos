using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using Avalonia.Platform;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SevenHinos.Models;
using SevenHinos.Services;

namespace SevenHinos.ViewModels;

public sealed record MonitorInfo(int Index, string Label, string Resolution);

public sealed record CategoryOptionViewModel(
    int? CategoryId,
    string Name,
    IReadOnlyList<int> MonitorPreset);

public enum VideoCategoryFilterKind
{
    All,
    Uncategorized,
    Category
}

public sealed record CategoryFilterItemViewModel(
    VideoCategoryFilterKind Kind,
    int? CategoryId,
    string Name);

public sealed partial class VideoMonitorSelectionViewModel : ObservableObject
{
    public int Index { get; init; }
    public string Label { get; init; } = string.Empty;

    [ObservableProperty] private bool _isSelected;

    public event Action? SelectionChanged;

    partial void OnIsSelectedChanged(bool value) => SelectionChanged?.Invoke();
}

public sealed partial class VideoCardViewModel : ObservableObject
{
    private readonly Func<VideoCardViewModel, Task<bool>> _onSaveAsync;
    private readonly Func<VideoCardViewModel, Task> _onRemoveAsync;
    private readonly Func<VideoCardViewModel, Task> _onPlayAsync;
    private readonly Action _onStop;
    private readonly Func<VideoCardViewModel, Task> _onMoveUpAsync;
    private readonly Func<VideoCardViewModel, Task> _onMoveDownAsync;
    private readonly Func<VideoCardViewModel, Task> _onApplyCategoryPresetAsync;

    private List<int> _selectedMonitorIndices = [];
    private int? _selectedCategoryId;

    public int VideoId { get; }
    public string VideoName { get; }
    public string FilePath { get; }

    [ObservableProperty] private bool _isEditing;
    [ObservableProperty] private bool _isPlaying;
    [ObservableProperty] private string _selectedMonitorsSummary = "Nenhuma tela selecionada";
    [ObservableProperty] private CategoryOptionViewModel? _selectedCategoryOption;

    public string EditButtonText => IsEditing ? "Salvar" : "Editar";
    public string SelectedCategoryName => SelectedCategoryOption?.Name ?? "Sem categoria";

    public ObservableCollection<VideoMonitorSelectionViewModel> MonitorOptions { get; } = [];
    public ObservableCollection<CategoryOptionViewModel> CategoryOptions { get; } = [];

    public IAsyncRelayCommand ToggleEditCommand { get; }
    public IAsyncRelayCommand RemoveCommand { get; }
    public IAsyncRelayCommand PlayCommand { get; }
    public IRelayCommand StopCommand { get; }
    public IAsyncRelayCommand MoveUpCommand { get; }
    public IAsyncRelayCommand MoveDownCommand { get; }
    public IAsyncRelayCommand ApplyCategoryPresetCommand { get; }

    public VideoCardViewModel(
        int videoId,
        string videoName,
        string filePath,
        IEnumerable<int> selectedMonitorIndices,
        int? selectedCategoryId,
        Func<VideoCardViewModel, Task<bool>> onSaveAsync,
        Func<VideoCardViewModel, Task> onRemoveAsync,
        Func<VideoCardViewModel, Task> onPlayAsync,
        Action onStop,
        Func<VideoCardViewModel, Task> onMoveUpAsync,
        Func<VideoCardViewModel, Task> onMoveDownAsync,
        Func<VideoCardViewModel, Task> onApplyCategoryPresetAsync)
    {
        VideoId = videoId;
        VideoName = videoName;
        FilePath = filePath;
        _selectedCategoryId = selectedCategoryId;
        _selectedMonitorIndices = selectedMonitorIndices
            .Where(i => i >= 0)
            .Distinct()
            .OrderBy(i => i)
            .ToList();

        _onSaveAsync = onSaveAsync;
        _onRemoveAsync = onRemoveAsync;
        _onPlayAsync = onPlayAsync;
        _onStop = onStop;
        _onMoveUpAsync = onMoveUpAsync;
        _onMoveDownAsync = onMoveDownAsync;
        _onApplyCategoryPresetAsync = onApplyCategoryPresetAsync;

        ToggleEditCommand = new AsyncRelayCommand(ToggleEditAsync);
        RemoveCommand = new AsyncRelayCommand(() => _onRemoveAsync(this));
        PlayCommand = new AsyncRelayCommand(() => _onPlayAsync(this));
        StopCommand = new RelayCommand(() => _onStop());
        MoveUpCommand = new AsyncRelayCommand(() => _onMoveUpAsync(this));
        MoveDownCommand = new AsyncRelayCommand(() => _onMoveDownAsync(this));
        ApplyCategoryPresetCommand = new AsyncRelayCommand(() => _onApplyCategoryPresetAsync(this));

        UpdateSummary();
    }

    partial void OnIsEditingChanged(bool value) => OnPropertyChanged(nameof(EditButtonText));

    partial void OnSelectedCategoryOptionChanged(CategoryOptionViewModel? value)
    {
        _selectedCategoryId = value?.CategoryId;
        OnPropertyChanged(nameof(SelectedCategoryName));
    }

    private async Task ToggleEditAsync()
    {
        if (!IsEditing)
        {
            IsEditing = true;
            return;
        }

        var saved = await _onSaveAsync(this);
        if (saved)
            IsEditing = false;
    }

    public IReadOnlyList<int> GetSelectedMonitorIndices() => _selectedMonitorIndices;

    public int? GetSelectedCategoryId() => SelectedCategoryOption?.CategoryId;

    public void SetPlaying(bool isPlaying)
    {
        IsPlaying = isPlaying;
    }

    public void SetCategoryOptions(IReadOnlyList<CategoryOptionViewModel> options)
    {
        CategoryOptions.Clear();
        foreach (var option in options)
            CategoryOptions.Add(option);

        var selected = CategoryOptions
            .FirstOrDefault(o => o.CategoryId == _selectedCategoryId)
            ?? CategoryOptions.FirstOrDefault();

        SelectedCategoryOption = selected;
    }

    public void SetCategorySelection(int? categoryId)
    {
        _selectedCategoryId = categoryId;

        if (CategoryOptions.Count == 0)
            return;

        SelectedCategoryOption = CategoryOptions
            .FirstOrDefault(o => o.CategoryId == categoryId)
            ?? CategoryOptions.FirstOrDefault();
    }

    public void SetSelectedMonitorIndices(IEnumerable<int> monitorIndices)
    {
        _selectedMonitorIndices = monitorIndices
            .Where(i => i >= 0)
            .Distinct()
            .OrderBy(i => i)
            .ToList();

        var selectedSet = _selectedMonitorIndices.ToHashSet();
        foreach (var option in MonitorOptions)
            option.IsSelected = selectedSet.Contains(option.Index);

        UpdateSummary();
    }

    public void SyncMonitors(IReadOnlyList<MonitorInfo> monitors)
    {
        var selectedSet = _selectedMonitorIndices.ToHashSet();

        MonitorOptions.Clear();
        foreach (var monitor in monitors)
        {
            var option = new VideoMonitorSelectionViewModel
            {
                Index = monitor.Index,
                Label = $"{monitor.Label} ({monitor.Resolution})",
                IsSelected = selectedSet.Contains(monitor.Index)
            };

            option.SelectionChanged += OnMonitorSelectionChanged;
            MonitorOptions.Add(option);
        }

        SyncSelectionFromOptions();
        UpdateSummary();
    }

    private void OnMonitorSelectionChanged() => SyncSelectionFromOptions();

    private void SyncSelectionFromOptions()
    {
        _selectedMonitorIndices = MonitorOptions
            .Where(m => m.IsSelected)
            .Select(m => m.Index)
            .OrderBy(i => i)
            .ToList();

        UpdateSummary();
    }

    private void UpdateSummary()
    {
        var selected = _selectedMonitorIndices;
        SelectedMonitorsSummary = selected.Count == 0
            ? "Nenhuma tela selecionada"
            : string.Join(", ", selected.Select(i => $"Tela {i + 1}"));
    }
}

public sealed partial class VideosViewModel : ViewModelBase
{
    private readonly IVideoConfigService _videoConfigService;
    private readonly IVideoOutputService _videoOutputService;
    private readonly List<MonitorInfo> _monitors = [];
    private readonly List<CategoryOptionViewModel> _categoryOptions = [];
    private readonly List<VideoCardViewModel> _allCards = [];

    private int? _playingVideoId;

    [ObservableProperty] private string _monitorStatusText = "Nenhuma tela detectada.";
    [ObservableProperty] private string _statusText = "Carregando vídeos...";
    [ObservableProperty] private string _newCategoryName = string.Empty;
    [ObservableProperty] private CategoryFilterItemViewModel? _selectedCategoryFilter;
    [ObservableProperty] private bool _isBusy;

    [ObservableProperty] private CategoryOptionViewModel? _selectedCategoryForManagement;
    [ObservableProperty] private string _categoryEditorName = string.Empty;

    public ObservableCollection<VideoCardViewModel> Videos { get; } = [];
    public ObservableCollection<CategoryFilterItemViewModel> CategoryFilters { get; } = [];
    public ObservableCollection<CategoryOptionViewModel> CategoryManagementOptions { get; } = [];
    public ObservableCollection<VideoMonitorSelectionViewModel> CategoryPresetMonitorOptions { get; } = [];

    public bool HasVideos => Videos.Count > 0;
    public bool IsAnyVideoPlaying => _allCards.Any(v => v.IsPlaying);
    public bool HasManageableCategories => CategoryManagementOptions.Count > 0;
    public bool CanEditSelectedCategory => SelectedCategoryForManagement is not null;

    public VideosViewModel(
        IVideoConfigService videoConfigService,
        IVideoOutputService videoOutputService)
    {
        _videoConfigService = videoConfigService;
        _videoOutputService = videoOutputService;
        _videoOutputService.OutputsStopped += OnVideoOutputsStopped;

        Videos.CollectionChanged += OnVideosCollectionChanged;
        CategoryManagementOptions.CollectionChanged += OnCategoryManagementCollectionChanged;

        _ = LoadAsync();
    }

    partial void OnSelectedCategoryFilterChanged(CategoryFilterItemViewModel? value) => ApplyFilter();

    partial void OnSelectedCategoryForManagementChanged(CategoryOptionViewModel? value)
    {
        LoadCategoryEditorFromSelection();
        OnPropertyChanged(nameof(CanEditSelectedCategory));
    }

    [RelayCommand]
    private async Task CreateCategoryAsync()
    {
        var name = NewCategoryName.Trim();
        if (string.IsNullOrWhiteSpace(name))
        {
            StatusText = "Digite um nome para criar a categoria.";
            return;
        }

        try
        {
            IsBusy = true;

            var created = await _videoConfigService.CreateCategoryAsync(name);
            NewCategoryName = string.Empty;

            await RefreshCategoriesAsync(created.Id);
            StatusText = $"Categoria criada: {created.Name}";
        }
        catch (Exception ex)
        {
            StatusText = $"Erro ao criar categoria: {ex.Message}";
        }
        finally
        {
            IsBusy = false;
        }
    }

    [RelayCommand]
    private async Task SaveSelectedCategoryAsync()
    {
        if (SelectedCategoryForManagement?.CategoryId is not int categoryId)
        {
            StatusText = "Selecione uma categoria para editar.";
            return;
        }

        var newName = CategoryEditorName.Trim();
        if (string.IsNullOrWhiteSpace(newName))
        {
            StatusText = "O nome da categoria é obrigatório.";
            return;
        }

        var preset = GetCategoryPresetSelection();

        try
        {
            IsBusy = true;

            await _videoConfigService.UpdateCategoryAsync(categoryId, newName, preset);
            await RefreshCategoriesAsync(categoryId);

            StatusText = preset.Count == 0
                ? $"Categoria atualizada: {newName}"
                : $"Categoria atualizada: {newName} (preset: {FormatPresetLabel(preset)})";
        }
        catch (Exception ex)
        {
            StatusText = $"Erro ao salvar categoria: {ex.Message}";
        }
        finally
        {
            IsBusy = false;
        }
    }

    [RelayCommand]
    private async Task DeleteSelectedCategoryAsync()
    {
        if (SelectedCategoryForManagement?.CategoryId is not int categoryId)
        {
            StatusText = "Selecione uma categoria para excluir.";
            return;
        }

        var categoryName = SelectedCategoryForManagement.Name;

        try
        {
            IsBusy = true;

            await _videoConfigService.DeleteCategoryAsync(categoryId);
            await RefreshCategoriesAsync(null);
            await ReloadVideosOnlyAsync();

            StatusText = $"Categoria removida: {categoryName}. Os vídeos ficaram em 'Sem categoria'.";
        }
        catch (Exception ex)
        {
            StatusText = $"Erro ao excluir categoria: {ex.Message}";
        }
        finally
        {
            IsBusy = false;
        }
    }

    [RelayCommand]
    private void StopAllOutputs()
    {
        StopVideoOutput();
    }

    public async Task AddVideoAsync(string filePath)
    {
        if (string.IsNullOrWhiteSpace(filePath) || !File.Exists(filePath))
            return;

        try
        {
            IsBusy = true;

            var defaultCategoryId = SelectedCategoryFilter is { Kind: VideoCategoryFilterKind.Category }
                ? SelectedCategoryFilter.CategoryId
                : null;

            IReadOnlyList<int> defaultMonitors = Array.Empty<int>();
            if (defaultCategoryId is not null)
            {
                defaultMonitors = _categoryOptions
                    .FirstOrDefault(c => c.CategoryId == defaultCategoryId)?.MonitorPreset
                    ?? Array.Empty<int>();
            }

            if (defaultMonitors.Count == 0 && _monitors.Count > 0)
                defaultMonitors = [0];

            var saved = await _videoConfigService.AddVideoAsync(filePath, defaultMonitors, defaultCategoryId);

            var card = BuildCard(saved);
            _allCards.Add(card);
            ApplyFilter();

            StatusText = $"Vídeo adicionado: {card.VideoName}";
        }
        catch (Exception ex)
        {
            StatusText = $"Erro ao adicionar vídeo: {ex.Message}";
        }
        finally
        {
            IsBusy = false;
        }
    }

    public void RefreshMonitors(IReadOnlyList<Screen> screens)
    {
        _monitors.Clear();

        for (int i = 0; i < screens.Count; i++)
        {
            var screen = screens[i];
            _monitors.Add(new MonitorInfo(
                i,
                $"Tela {i + 1}",
                $"{screen.Bounds.Width}x{screen.Bounds.Height}"));
        }

        MonitorStatusText = _monitors.Count == 0
            ? "Nenhuma tela detectada."
            : $"{_monitors.Count} tela(s) detectada(s).";

        foreach (var video in _allCards)
            video.SyncMonitors(_monitors);

        LoadCategoryEditorFromSelection();
    }

    private async Task LoadAsync()
    {
        try
        {
            IsBusy = true;
            StatusText = "Carregando vídeos e categorias...";

            var categoriesTask = _videoConfigService.GetCategoriesAsync();
            var videosTask = _videoConfigService.GetVideosAsync();
            await Task.WhenAll(categoriesTask, videosTask);

            var categories = categoriesTask.Result;
            RebuildCategoryOptions(categories);
            RebuildCategoryFilters(categories, null);
            RebuildCategoryManagementOptions(null);
            RebuildCards(videosTask.Result);

            StatusText = _allCards.Count == 0
                ? "Clique em Adicionar vídeo para começar."
                : $"{_allCards.Count} vídeo(s) carregado(s).";
        }
        catch (Exception ex)
        {
            StatusText = $"Erro ao carregar vídeos: {ex.Message}";
        }
        finally
        {
            IsBusy = false;
        }
    }

    private async Task ReloadVideosOnlyAsync()
    {
        var videos = await _videoConfigService.GetVideosAsync();
        RebuildCards(videos);
    }

    private async Task RefreshCategoriesAsync(int? categoryToSelect)
    {
        var categories = await _videoConfigService.GetCategoriesAsync();

        RebuildCategoryOptions(categories);
        RebuildCategoryFilters(categories, categoryToSelect);
        RebuildCategoryManagementOptions(categoryToSelect);

        foreach (var card in _allCards)
            card.SetCategoryOptions(_categoryOptions);

        ApplyFilter();
    }

    private void RebuildCategoryOptions(IReadOnlyList<VideoCategory> categories)
    {
        _categoryOptions.Clear();
        _categoryOptions.Add(new CategoryOptionViewModel(
            CategoryId: null,
            Name: "Sem categoria",
            MonitorPreset: Array.Empty<int>()));

        foreach (var category in categories)
        {
            _categoryOptions.Add(new CategoryOptionViewModel(
                CategoryId: category.Id,
                Name: category.Name,
                MonitorPreset: ParseMonitorPreset(category.MonitorPreset)));
        }
    }

    private void RebuildCategoryFilters(
        IReadOnlyList<VideoCategory> categories,
        int? categoryToSelect)
    {
        var previous = SelectedCategoryFilter;

        CategoryFilters.Clear();
        CategoryFilters.Add(new CategoryFilterItemViewModel(VideoCategoryFilterKind.All, null, "Todas as categorias"));
        CategoryFilters.Add(new CategoryFilterItemViewModel(VideoCategoryFilterKind.Uncategorized, null, "Sem categoria"));
        foreach (var category in categories)
            CategoryFilters.Add(new CategoryFilterItemViewModel(VideoCategoryFilterKind.Category, category.Id, category.Name));

        CategoryFilterItemViewModel? nextFilter = null;
        if (categoryToSelect is not null)
        {
            nextFilter = CategoryFilters.FirstOrDefault(f =>
                f.Kind == VideoCategoryFilterKind.Category &&
                f.CategoryId == categoryToSelect);
        }

        nextFilter ??= previous is not null
            ? CategoryFilters.FirstOrDefault(f =>
                f.Kind == previous.Kind && f.CategoryId == previous.CategoryId)
            : null;

        SelectedCategoryFilter = nextFilter ?? CategoryFilters.First();
    }

    private void RebuildCategoryManagementOptions(int? categoryToSelect)
    {
        var previous = SelectedCategoryForManagement;

        CategoryManagementOptions.Clear();
        foreach (var category in _categoryOptions.Where(c => c.CategoryId is not null))
            CategoryManagementOptions.Add(category);

        CategoryOptionViewModel? selected = null;

        if (categoryToSelect is not null)
            selected = CategoryManagementOptions.FirstOrDefault(c => c.CategoryId == categoryToSelect);

        selected ??= previous is not null
            ? CategoryManagementOptions.FirstOrDefault(c => c.CategoryId == previous.CategoryId)
            : null;

        SelectedCategoryForManagement = selected;
        LoadCategoryEditorFromSelection();
    }

    private void LoadCategoryEditorFromSelection()
    {
        if (SelectedCategoryForManagement is null)
        {
            CategoryEditorName = string.Empty;
            CategoryPresetMonitorOptions.Clear();
            return;
        }

        CategoryEditorName = SelectedCategoryForManagement.Name;
        RebuildCategoryPresetMonitorOptions(SelectedCategoryForManagement.MonitorPreset);
    }

    private void RebuildCategoryPresetMonitorOptions(IReadOnlyList<int> selectedPreset)
    {
        CategoryPresetMonitorOptions.Clear();

        var selectedSet = selectedPreset.ToHashSet();
        foreach (var monitor in _monitors)
        {
            CategoryPresetMonitorOptions.Add(new VideoMonitorSelectionViewModel
            {
                Index = monitor.Index,
                Label = $"{monitor.Label} ({monitor.Resolution})",
                IsSelected = selectedSet.Contains(monitor.Index)
            });
        }
    }

    private IReadOnlyList<int> GetCategoryPresetSelection()
    {
        return CategoryPresetMonitorOptions
            .Where(m => m.IsSelected)
            .Select(m => m.Index)
            .Distinct()
            .OrderBy(i => i)
            .ToList();
    }

    private void RebuildCards(IReadOnlyList<VideoConfig> videos)
    {
        _allCards.Clear();

        foreach (var video in videos)
            _allCards.Add(BuildCard(video));

        ApplyFilter();
        SetPlayingCard(_playingVideoId);
    }

    private VideoCardViewModel BuildCard(VideoConfig video)
    {
        var card = new VideoCardViewModel(
            videoId: video.Id,
            videoName: video.VideoName,
            filePath: video.FilePath,
            selectedMonitorIndices: video.MonitorTargets.Select(t => t.MonitorIndex),
            selectedCategoryId: video.CategoryId,
            onSaveAsync: SaveVideoAsync,
            onRemoveAsync: RemoveVideoAsync,
            onPlayAsync: PlayVideoAsync,
            onStop: () => StopVideoOutput(),
            onMoveUpAsync: MoveVideoUpAsync,
            onMoveDownAsync: MoveVideoDownAsync,
            onApplyCategoryPresetAsync: ApplyCategoryPresetAsync);

        card.SetCategoryOptions(_categoryOptions);
        card.SetCategorySelection(video.CategoryId);
        card.SyncMonitors(_monitors);

        return card;
    }

    private async Task<bool> SaveVideoAsync(VideoCardViewModel card)
    {
        try
        {
            await _videoConfigService.UpdateVideoAsync(
                card.VideoId,
                card.GetSelectedMonitorIndices(),
                card.GetSelectedCategoryId());

            ApplyFilter();
            StatusText = $"Configuração salva: {card.VideoName}";
            return true;
        }
        catch (Exception ex)
        {
            StatusText = $"Erro ao salvar vídeo: {ex.Message}";
            return false;
        }
    }

    private async Task RemoveVideoAsync(VideoCardViewModel card)
    {
        try
        {
            if (card.IsPlaying)
                StopVideoOutput(updateStatus: false);

            await _videoConfigService.DeleteVideoAsync(card.VideoId);

            _allCards.Remove(card);
            ApplyFilter();
            StatusText = $"Vídeo removido: {card.VideoName}";
        }
        catch (Exception ex)
        {
            StatusText = $"Erro ao remover vídeo: {ex.Message}";
        }
    }

    private async Task MoveVideoUpAsync(VideoCardViewModel card)
    {
        try
        {
            await _videoConfigService.MoveVideoAsync(card.VideoId, -1);
            await ReloadVideosOnlyAsync();
            StatusText = $"Ordem atualizada: {card.VideoName}";
        }
        catch (Exception ex)
        {
            StatusText = $"Erro ao mover vídeo: {ex.Message}";
        }
    }

    private async Task MoveVideoDownAsync(VideoCardViewModel card)
    {
        try
        {
            await _videoConfigService.MoveVideoAsync(card.VideoId, +1);
            await ReloadVideosOnlyAsync();
            StatusText = $"Ordem atualizada: {card.VideoName}";
        }
        catch (Exception ex)
        {
            StatusText = $"Erro ao mover vídeo: {ex.Message}";
        }
    }

    private Task ApplyCategoryPresetAsync(VideoCardViewModel card)
    {
        var categoryId = card.GetSelectedCategoryId();
        if (categoryId is null)
        {
            StatusText = "Selecione uma categoria para aplicar preset.";
            return Task.CompletedTask;
        }

        var category = _categoryOptions.FirstOrDefault(c => c.CategoryId == categoryId);
        if (category is null || category.MonitorPreset.Count == 0)
        {
            StatusText = "Essa categoria não possui preset de telas.";
            return Task.CompletedTask;
        }

        card.SetSelectedMonitorIndices(category.MonitorPreset);
        StatusText = $"Preset aplicado: {FormatPresetLabel(category.MonitorPreset)}";
        return Task.CompletedTask;
    }

    private async Task PlayVideoAsync(VideoCardViewModel card)
    {
        try
        {
            var monitors = card.GetSelectedMonitorIndices();
            if (monitors.Count == 0)
            {
                StatusText = "Marque pelo menos uma tela antes de exibir o vídeo.";
                return;
            }

            await _videoOutputService.ShowVideoAsync(card.FilePath, monitors);
            SetPlayingCard(card.VideoId);
            StatusText = $"Exibindo {card.VideoName} em {card.SelectedMonitorsSummary}.";
        }
        catch (Exception ex)
        {
            SetPlayingCard(null);
            StatusText = $"Erro ao exibir vídeo: {ex.Message}";
        }
    }

    private void StopVideoOutput(bool updateStatus = true)
    {
        _videoOutputService.StopAll();
        SetPlayingCard(null);

        if (updateStatus)
            StatusText = "Exibição de vídeo parada.";
    }

    private void SetPlayingCard(int? videoId)
    {
        _playingVideoId = videoId;

        foreach (var card in _allCards)
            card.SetPlaying(videoId is not null && card.VideoId == videoId.Value);

        OnPropertyChanged(nameof(IsAnyVideoPlaying));
    }

    private void OnVideoOutputsStopped()
    {
        void ApplyStoppedState()
        {
            SetPlayingCard(null);
            StatusText = "Exibição de vídeo parada.";
        }

        if (Dispatcher.UIThread.CheckAccess())
            ApplyStoppedState();
        else
            Dispatcher.UIThread.Post(ApplyStoppedState);
    }

    private void ApplyFilter()
    {
        IEnumerable<VideoCardViewModel> source = _allCards;

        var filter = SelectedCategoryFilter;
        if (filter is not null)
        {
            source = filter.Kind switch
            {
                VideoCategoryFilterKind.Category =>
                    _allCards.Where(v => v.GetSelectedCategoryId() == filter.CategoryId),
                VideoCategoryFilterKind.Uncategorized =>
                    _allCards.Where(v => v.GetSelectedCategoryId() is null),
                _ => _allCards
            };
        }

        Videos.Clear();
        foreach (var video in source)
            Videos.Add(video);

        OnPropertyChanged(nameof(HasVideos));
    }

    private static IReadOnlyList<int> ParseMonitorPreset(string? csv)
    {
        if (string.IsNullOrWhiteSpace(csv))
            return Array.Empty<int>();

        return csv
            .Split(',', StringSplitOptions.RemoveEmptyEntries)
            .Select(s => int.TryParse(s.Trim(), out var parsed) ? parsed : -1)
            .Where(i => i >= 0)
            .Distinct()
            .OrderBy(i => i)
            .ToList();
    }

    private static string FormatPresetLabel(IReadOnlyList<int> monitorPreset)
    {
        if (monitorPreset.Count == 0)
            return "sem telas";

        return string.Join(", ", monitorPreset.Select(i => $"Tela {i + 1}"));
    }

    private void OnVideosCollectionChanged(object? sender, System.Collections.Specialized.NotifyCollectionChangedEventArgs e)
    {
        OnPropertyChanged(nameof(HasVideos));
    }

    private void OnCategoryManagementCollectionChanged(object? sender, System.Collections.Specialized.NotifyCollectionChangedEventArgs e)
    {
        OnPropertyChanged(nameof(HasManageableCategories));
    }
}
