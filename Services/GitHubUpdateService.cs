using System.Diagnostics;
using System.Linq;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Reflection;
using System.Text.Json;
using Avalonia.Controls;
using Avalonia.Threading;
using SevenHinos.Views;

namespace SevenHinos.Services;

public sealed class GitHubUpdateService : IAppUpdateService
{
    private sealed record ReleaseInfo(Version Version, string? InstallerUrl, string? ReleasePageUrl);

    private sealed class GitHubReleasePayload
    {
        public string? tag_name { get; set; }
        public string? html_url { get; set; }
        public List<GitHubReleaseAssetPayload>? assets { get; set; }
    }

    private sealed class GitHubReleaseAssetPayload
    {
        public string? name { get; set; }
        public string? browser_download_url { get; set; }
    }

    private static readonly HttpClient Http = CreateHttpClient();

    // Keep this centralized to simplify changing repository later.
    private const string RepositoryOwner = "oadrianrabelo";
    private const string RepositoryName = "7Hinos";

    public async Task TryCheckAndPromptAsync(Window owner, CancellationToken ct = default)
    {
        ReleaseInfo? latest;
        try
        {
            latest = await GetLatestReleaseAsync(ct);
        }
        catch
        {
            // Offline / DNS / GitHub hiccup: fail silently by design.
            return;
        }

        if (latest is null)
            return;

        var currentVersion = ResolveCurrentVersion();
        if (latest.Version <= currentVersion)
            return;

        var shouldUpdate = await Dispatcher.UIThread.InvokeAsync(async () =>
        {
            var prompt = new UpdatePromptWindow();
            return await prompt.ShowDialogAsync(
                owner,
                currentVersion.ToString(3),
                latest.Version.ToString(3));
        });

        if (!shouldUpdate)
            return;

        if (string.IsNullOrWhiteSpace(latest.InstallerUrl))
        {
            if (!string.IsNullOrWhiteSpace(latest.ReleasePageUrl))
            {
                try
                {
                    Process.Start(new ProcessStartInfo
                    {
                        FileName = latest.ReleasePageUrl,
                        UseShellExecute = true
                    });
                }
                catch
                {
                    // Silent on purpose.
                }
            }

            return;
        }

        try
        {
            var installerPath = await DownloadInstallerAsync(latest, ct);
            Process.Start(new ProcessStartInfo
            {
                FileName = installerPath,
                UseShellExecute = true
            });
        }
        catch
        {
            // If download/run fails we do not interrupt app flow.
        }
    }

    private static async Task<ReleaseInfo?> GetLatestReleaseAsync(CancellationToken ct)
    {
        using var request = new HttpRequestMessage(
            HttpMethod.Get,
            $"https://api.github.com/repos/{RepositoryOwner}/{RepositoryName}/releases/latest");

        request.Headers.Accept.Add(new MediaTypeWithQualityHeaderValue("application/vnd.github+json"));

        using var response = await Http.SendAsync(request, ct);
        if (!response.IsSuccessStatusCode)
            return null;

        await using var content = await response.Content.ReadAsStreamAsync(ct);
        var payload = await JsonSerializer.DeserializeAsync<GitHubReleasePayload>(content, cancellationToken: ct);

        if (payload is null || string.IsNullOrWhiteSpace(payload.tag_name))
            return null;

        if (!TryParseVersion(payload.tag_name, out var releaseVersion))
            return null;

        var installerUrl = payload.assets?
            .FirstOrDefault(a =>
                !string.IsNullOrWhiteSpace(a.name) &&
                a.name.EndsWith(".msi", StringComparison.OrdinalIgnoreCase))?
            .browser_download_url;

        return new ReleaseInfo(releaseVersion, installerUrl, payload.html_url);
    }

    private static async Task<string> DownloadInstallerAsync(ReleaseInfo release, CancellationToken ct)
    {
        var targetFolder = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "7Hinos",
            "updates");

        Directory.CreateDirectory(targetFolder);

        var fileName = $"7Hinos-Setup-{release.Version:0.0.0}.msi";
        var targetPath = Path.Combine(targetFolder, fileName);

        using var response = await Http.GetAsync(
            release.InstallerUrl,
            HttpCompletionOption.ResponseHeadersRead,
            ct);

        response.EnsureSuccessStatusCode();

        await using var source = await response.Content.ReadAsStreamAsync(ct);
        await using var target = File.Create(targetPath);
        await source.CopyToAsync(target, ct);

        return targetPath;
    }

    private static Version ResolveCurrentVersion()
    {
        var asm = Assembly.GetExecutingAssembly();

        var informational = asm
            .GetCustomAttribute<AssemblyInformationalVersionAttribute>()?
            .InformationalVersion;

        if (TryParseVersion(informational, out var parsedInfoVersion))
            return parsedInfoVersion;

        var assemblyVersion = asm.GetName().Version;
        if (assemblyVersion is not null)
            return new Version(assemblyVersion.Major, assemblyVersion.Minor, Math.Max(assemblyVersion.Build, 0));

        return new Version(0, 0, 0);
    }

    private static bool TryParseVersion(string? rawValue, out Version version)
    {
        version = new Version(0, 0, 0);
        if (string.IsNullOrWhiteSpace(rawValue))
            return false;

        var clean = rawValue.Trim();
        if (clean.StartsWith("v", StringComparison.OrdinalIgnoreCase))
            clean = clean[1..];

        clean = clean.Split('+')[0];
        clean = clean.Split('-')[0];

        var parts = clean
            .Split('.', StringSplitOptions.RemoveEmptyEntries)
            .Select(p => int.TryParse(p, out var parsed) ? parsed : -1)
            .ToList();

        if (parts.Count == 0 || parts.Any(p => p < 0))
            return false;

        while (parts.Count < 3)
            parts.Add(0);

        version = new Version(parts[0], parts[1], parts[2]);
        return true;
    }

    private static HttpClient CreateHttpClient()
    {
        var httpClient = new HttpClient
        {
            Timeout = TimeSpan.FromSeconds(8)
        };

        httpClient.DefaultRequestHeaders.UserAgent.ParseAdd("7Hinos-Updater/1.0");
        return httpClient;
    }
}
