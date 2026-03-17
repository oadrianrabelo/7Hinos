using SevenHinos.Models;

namespace SevenHinos.Services;

public interface IVideoConfigService
{
    Task<IReadOnlyList<VideoCategory>> GetCategoriesAsync(CancellationToken ct = default);
    Task<VideoCategory> CreateCategoryAsync(string name, CancellationToken ct = default);
    Task UpdateCategoryAsync(
        int categoryId,
        string name,
        IEnumerable<int> monitorPreset,
        CancellationToken ct = default);
    Task DeleteCategoryAsync(int categoryId, CancellationToken ct = default);

    Task<IReadOnlyList<VideoConfig>> GetVideosAsync(CancellationToken ct = default);
    Task<VideoConfig> AddVideoAsync(
        string filePath,
        IEnumerable<int> monitorIndices,
        int? categoryId,
        CancellationToken ct = default);

    Task UpdateVideoAsync(
        int videoId,
        string videoName,
        IEnumerable<int> monitorIndices,
        int? categoryId,
        CancellationToken ct = default);
    Task MoveVideoAsync(int videoId, int direction, CancellationToken ct = default);

    Task DeleteVideoAsync(int videoId, CancellationToken ct = default);
}
