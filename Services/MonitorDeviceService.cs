using Microsoft.EntityFrameworkCore;
using SevenHinos.Data;
using SevenHinos.Models;

namespace SevenHinos.Services;

public sealed class MonitorDeviceService(IDbContextFactory<AppDbContext> dbFactory) : IMonitorDeviceService
{
    public async Task<IReadOnlyList<MonitorDevice>> GetAllMonitorsAsync(CancellationToken ct = default)
    {
        await using var db = await dbFactory.CreateDbContextAsync(ct);
        return await db.MonitorDevices
            .OrderBy(m => m.MonitorIndex)
            .ToListAsync(ct);
    }

    public async Task<MonitorDevice?> GetMonitorByIndexAsync(int monitorIndex, CancellationToken ct = default)
    {
        if (monitorIndex < 0)
            return null;

        await using var db = await dbFactory.CreateDbContextAsync(ct);
        return await db.MonitorDevices
            .FirstOrDefaultAsync(m => m.MonitorIndex == monitorIndex, ct);
    }

    public async Task<string> GetMonitorNameAsync(int monitorIndex, CancellationToken ct = default)
    {
        var monitor = await GetMonitorByIndexAsync(monitorIndex, ct);
        if (monitor != null && !string.IsNullOrWhiteSpace(monitor.CustomName))
            return monitor.CustomName;

        return $"Tela {monitorIndex + 1}";
    }

    public async Task SetMonitorNameAsync(int monitorIndex, string customName, CancellationToken ct = default)
    {
        if (monitorIndex < 0)
            throw new ArgumentException("Monitor index must be non-negative.", nameof(monitorIndex));

        var trimmed = (customName ?? string.Empty).Trim();

        await using var db = await dbFactory.CreateDbContextAsync(ct);
        var monitor = await db.MonitorDevices
            .FirstOrDefaultAsync(m => m.MonitorIndex == monitorIndex, ct);

        if (monitor == null)
        {
            monitor = new MonitorDevice
            {
                MonitorIndex = monitorIndex,
                CustomName = trimmed,
                CreatedAt = DateTime.UtcNow,
                UpdatedAt = DateTime.UtcNow
            };
            db.MonitorDevices.Add(monitor);
        }
        else
        {
            monitor.CustomName = trimmed;
            monitor.UpdatedAt = DateTime.UtcNow;
        }

        await db.SaveChangesAsync(ct);
    }

    public async Task EnsureMonitorExistsAsync(int monitorIndex, CancellationToken ct = default)
    {
        if (monitorIndex < 0)
            return;

        var existing = await GetMonitorByIndexAsync(monitorIndex, ct);
        if (existing != null)
            return;

        await using var db = await dbFactory.CreateDbContextAsync(ct);
        var monitor = new MonitorDevice
        {
            MonitorIndex = monitorIndex,
            CustomName = string.Empty,
            CreatedAt = DateTime.UtcNow,
            UpdatedAt = DateTime.UtcNow
        };
        db.MonitorDevices.Add(monitor);
        await db.SaveChangesAsync(ct);
    }
}
