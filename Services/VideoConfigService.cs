using System.IO;
using Microsoft.EntityFrameworkCore;
using SevenHinos.Data;
using SevenHinos.Models;

namespace SevenHinos.Services;

public sealed class VideoConfigService(IDbContextFactory<AppDbContext> dbFactory) : IVideoConfigService
{
    public async Task<IReadOnlyList<VideoCategory>> GetCategoriesAsync(CancellationToken ct = default)
    {
        await using var db = await dbFactory.CreateDbContextAsync(ct);
        return await db.VideoCategories
            .AsNoTracking()
            .OrderBy(c => c.Name)
            .ToListAsync(ct);
    }

    public async Task<VideoCategory> CreateCategoryAsync(string name, CancellationToken ct = default)
    {
        var trimmed = (name ?? string.Empty).Trim();
        if (string.IsNullOrWhiteSpace(trimmed))
            throw new InvalidOperationException("O nome da categoria é obrigatório.");

        await using var db = await dbFactory.CreateDbContextAsync(ct);

        var lower = trimmed.ToLowerInvariant();
        var exists = await db.VideoCategories
            .AnyAsync(c => c.Name.ToLower() == lower, ct);

        if (exists)
            throw new InvalidOperationException("Essa categoria já existe.");

        var category = new VideoCategory
        {
            Name = trimmed,
            MonitorPreset = string.Empty
        };

        db.VideoCategories.Add(category);
        await db.SaveChangesAsync(ct);
        return category;
    }

    public async Task UpdateCategoryAsync(
        int categoryId,
        string name,
        IEnumerable<int> monitorPreset,
        CancellationToken ct = default)
    {
        var trimmed = (name ?? string.Empty).Trim();
        if (string.IsNullOrWhiteSpace(trimmed))
            throw new InvalidOperationException("O nome da categoria é obrigatório.");

        await using var db = await dbFactory.CreateDbContextAsync(ct);

        var category = await db.VideoCategories.FirstOrDefaultAsync(c => c.Id == categoryId, ct)
            ?? throw new InvalidOperationException("Categoria não encontrada.");

        var lower = trimmed.ToLowerInvariant();
        var exists = await db.VideoCategories
            .AnyAsync(c => c.Id != categoryId && c.Name.ToLower() == lower, ct);

        if (exists)
            throw new InvalidOperationException("Já existe uma categoria com esse nome.");

        category.Name = trimmed;
        category.MonitorPreset = EncodeMonitorPreset(monitorPreset);

        await db.SaveChangesAsync(ct);
    }

    public async Task DeleteCategoryAsync(int categoryId, CancellationToken ct = default)
    {
        await using var db = await dbFactory.CreateDbContextAsync(ct);

        var category = await db.VideoCategories.FirstOrDefaultAsync(c => c.Id == categoryId, ct)
            ?? throw new InvalidOperationException("Categoria não encontrada.");

        var uncategorized = await db.VideoConfigs
            .Where(v => v.CategoryId == null)
            .OrderBy(v => v.DisplayOrder)
            .ThenBy(v => v.Id)
            .ToListAsync(ct);

        var categoryVideos = await db.VideoConfigs
            .Where(v => v.CategoryId == categoryId)
            .OrderBy(v => v.DisplayOrder)
            .ThenBy(v => v.Id)
            .ToListAsync(ct);

        var nextOrder = uncategorized.Count;
        foreach (var video in categoryVideos)
        {
            video.CategoryId = null;
            video.DisplayOrder = nextOrder++;
        }

        db.VideoCategories.Remove(category);

        await db.SaveChangesAsync(ct);
        await NormalizeOrderAsync(db, null, ct);
        await db.SaveChangesAsync(ct);
    }

    public async Task<IReadOnlyList<VideoConfig>> GetVideosAsync(CancellationToken ct = default)
    {
        await using var db = await dbFactory.CreateDbContextAsync(ct);
        var videos = await db.VideoConfigs
            .AsNoTracking()
            .Include(v => v.Category)
            .Include(v => v.MonitorTargets)
            .ToListAsync(ct);

        return videos
            .OrderBy(v => v.CategoryId is null ? 1 : 0)
            .ThenBy(v => v.Category?.Name)
            .ThenBy(v => v.DisplayOrder)
            .ThenBy(v => v.VideoName)
            .ToList();
    }

    public async Task<VideoConfig> AddVideoAsync(
        string filePath,
        IEnumerable<int> monitorIndices,
        int? categoryId,
        CancellationToken ct = default)
    {
        if (string.IsNullOrWhiteSpace(filePath))
            throw new InvalidOperationException("Selecione um arquivo de vídeo válido.");

        var absolutePath = Path.GetFullPath(filePath);

        await using var db = await dbFactory.CreateDbContextAsync(ct);

        var lower = absolutePath.ToLowerInvariant();
        var duplicate = await db.VideoConfigs
            .AnyAsync(v => v.FilePath.ToLower() == lower, ct);

        if (duplicate)
            throw new InvalidOperationException("Esse vídeo já foi adicionado.");

        VideoCategory? category = null;
        if (categoryId is not null)
            category = await db.VideoCategories.FirstOrDefaultAsync(c => c.Id == categoryId.Value, ct);

        categoryId = category?.Id;

        var presetMonitors = category is null
            ? []
            : ParseMonitorPreset(category.MonitorPreset);

        var selectedMonitors = monitorIndices
            .Where(i => i >= 0)
            .Distinct()
            .OrderBy(i => i)
            .ToList();

        if (selectedMonitors.Count == 0 && presetMonitors.Count > 0)
            selectedMonitors = presetMonitors.ToList();

        var displayOrder = await GetNextDisplayOrderAsync(db, categoryId, ct);

        var video = new VideoConfig
        {
            FilePath = absolutePath,
            VideoName = Path.GetFileName(absolutePath),
            DisplayOrder = displayOrder,
            CategoryId = categoryId,
            MonitorTargets = selectedMonitors
                .Select(i => new VideoMonitorTarget { MonitorIndex = i })
                .ToList()
        };

        db.VideoConfigs.Add(video);
        await db.SaveChangesAsync(ct);

        return await GetVideoByIdAsync(video.Id, ct);
    }

    public async Task UpdateVideoAsync(
        int videoId,
        string videoName,
        IEnumerable<int> monitorIndices,
        int? categoryId,
        CancellationToken ct = default)
    {
        var trimmed = (videoName ?? string.Empty).Trim();
        if (string.IsNullOrWhiteSpace(trimmed))
            throw new InvalidOperationException("O nome do vídeo é obrigatório.");

        await using var db = await dbFactory.CreateDbContextAsync(ct);

        var video = await db.VideoConfigs
            .Include(v => v.MonitorTargets)
            .FirstOrDefaultAsync(v => v.Id == videoId, ct)
            ?? throw new InvalidOperationException("Vídeo não encontrado.");

        var oldCategoryId = video.CategoryId;

        VideoCategory? category = null;
        if (categoryId is not null)
            category = await db.VideoCategories.FirstOrDefaultAsync(c => c.Id == categoryId.Value, ct);

        categoryId = category?.Id;

        video.VideoName = trimmed;
        video.CategoryId = categoryId;

        if (oldCategoryId != categoryId)
            video.DisplayOrder = await GetNextDisplayOrderAsync(db, categoryId, ct);

        db.VideoMonitorTargets.RemoveRange(video.MonitorTargets);
        video.MonitorTargets.Clear();

        foreach (var index in monitorIndices.Where(i => i >= 0).Distinct().OrderBy(i => i))
            video.MonitorTargets.Add(new VideoMonitorTarget { MonitorIndex = index });

        await db.SaveChangesAsync(ct);

        if (oldCategoryId != categoryId)
        {
            await NormalizeOrderAsync(db, oldCategoryId, ct);
            await db.SaveChangesAsync(ct);
        }
    }

    public async Task MoveVideoAsync(int videoId, int direction, CancellationToken ct = default)
    {
        if (direction == 0)
            return;

        await using var db = await dbFactory.CreateDbContextAsync(ct);

        var targetVideo = await db.VideoConfigs.FirstOrDefaultAsync(v => v.Id == videoId, ct)
            ?? throw new InvalidOperationException("Vídeo não encontrado.");

        var sameCategory = await db.VideoConfigs
            .Where(v => v.CategoryId == targetVideo.CategoryId)
            .OrderBy(v => v.DisplayOrder)
            .ThenBy(v => v.Id)
            .ToListAsync(ct);

        var currentIndex = sameCategory.FindIndex(v => v.Id == videoId);
        if (currentIndex < 0)
            return;

        var desiredIndex = currentIndex + Math.Sign(direction);
        if (desiredIndex < 0 || desiredIndex >= sameCategory.Count)
            return;

        var item = sameCategory[currentIndex];
        sameCategory.RemoveAt(currentIndex);
        sameCategory.Insert(desiredIndex, item);

        for (int i = 0; i < sameCategory.Count; i++)
            sameCategory[i].DisplayOrder = i;

        await db.SaveChangesAsync(ct);
    }

    public async Task DeleteVideoAsync(int videoId, CancellationToken ct = default)
    {
        await using var db = await dbFactory.CreateDbContextAsync(ct);

        var video = await db.VideoConfigs.FirstOrDefaultAsync(v => v.Id == videoId, ct);
        if (video is null)
            return;

        var categoryId = video.CategoryId;

        db.VideoConfigs.Remove(video);
        await db.SaveChangesAsync(ct);

        await NormalizeOrderAsync(db, categoryId, ct);
        await db.SaveChangesAsync(ct);
    }

    private async Task<VideoConfig> GetVideoByIdAsync(int id, CancellationToken ct)
    {
        await using var db = await dbFactory.CreateDbContextAsync(ct);
        return await db.VideoConfigs
            .AsNoTracking()
            .Include(v => v.Category)
            .Include(v => v.MonitorTargets)
            .FirstAsync(v => v.Id == id, ct);
    }

    private static async Task<int> GetNextDisplayOrderAsync(
        AppDbContext db,
        int? categoryId,
        CancellationToken ct)
    {
        var max = await db.VideoConfigs
            .Where(v => v.CategoryId == categoryId)
            .Select(v => (int?)v.DisplayOrder)
            .MaxAsync(ct);

        return (max ?? -1) + 1;
    }

    private static async Task NormalizeOrderAsync(
        AppDbContext db,
        int? categoryId,
        CancellationToken ct)
    {
        var ordered = await db.VideoConfigs
            .Where(v => v.CategoryId == categoryId)
            .OrderBy(v => v.DisplayOrder)
            .ThenBy(v => v.Id)
            .ToListAsync(ct);

        for (int i = 0; i < ordered.Count; i++)
            ordered[i].DisplayOrder = i;
    }

    private static IReadOnlyList<int> ParseMonitorPreset(string? csv)
    {
        if (string.IsNullOrWhiteSpace(csv))
            return [];

        return csv
            .Split(',', StringSplitOptions.RemoveEmptyEntries)
            .Select(s => int.TryParse(s.Trim(), out var parsed) ? parsed : -1)
            .Where(i => i >= 0)
            .Distinct()
            .OrderBy(i => i)
            .ToList();
    }

    private static string EncodeMonitorPreset(IEnumerable<int> monitorPreset)
    {
        return string.Join(",", monitorPreset
            .Where(i => i >= 0)
            .Distinct()
            .OrderBy(i => i));
    }
}
