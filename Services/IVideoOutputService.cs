namespace SevenHinos.Services;

public interface IVideoOutputService : IDisposable
{
    bool IsActive { get; }

    event Action? OutputsStopped;

    Task ShowVideoAsync(
        string filePath,
        IReadOnlyList<int> monitorIndices,
        CancellationToken ct = default);

    void ShowIdentificationWindow(int monitorIndex);

    void StopAll();
}
