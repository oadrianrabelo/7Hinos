using SevenHinos.Models;

namespace SevenHinos.Services;

public interface IMonitorDeviceService
{
    Task<IReadOnlyList<MonitorDevice>> GetAllMonitorsAsync(CancellationToken ct = default);
    Task<MonitorDevice?> GetMonitorByIndexAsync(int monitorIndex, CancellationToken ct = default);
    Task<string> GetMonitorNameAsync(int monitorIndex, CancellationToken ct = default);
    Task SetMonitorNameAsync(int monitorIndex, string customName, CancellationToken ct = default);
    Task EnsureMonitorExistsAsync(int monitorIndex, CancellationToken ct = default);
}
