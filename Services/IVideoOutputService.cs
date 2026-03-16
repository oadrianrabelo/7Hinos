namespace SevenHinos.Services;

public interface IVideoOutputService : IDisposable
{
    bool IsActive { get; }

    Task ShowVideoAsync(
        string filePath,
        IReadOnlyList<int> monitorIndices,
        CancellationToken ct = default);

    void StopAll();
}
