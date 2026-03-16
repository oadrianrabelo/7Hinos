using Avalonia.Controls;

namespace SevenHinos.Services;

public interface IAppUpdateService
{
    Task TryCheckAndPromptAsync(Window owner, CancellationToken ct = default);
}
