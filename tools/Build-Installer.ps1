param(
    [string] $Configuration = "Release",
    [string] $Runtime = "win-x64",
    [string] $PublishDir = ".artifacts/publish/win-x64-portable",
    [string] $InstallerDir = ".artifacts/installer",
    [string] $VersionOverride = ""
)

$ErrorActionPreference = "Stop"

function Resolve-FullPath {
    param([string] $BasePath, [string] $PathValue)

    if ([System.IO.Path]::IsPathRooted($PathValue)) {
        return [System.IO.Path]::GetFullPath($PathValue)
    }

    return [System.IO.Path]::GetFullPath((Join-Path $BasePath $PathValue))
}

function Convert-ToMsiVersion {
    param([string] $VersionText)

    $normalizedVersion = ($VersionText ?? "0.1.0").Trim()
    if ($normalizedVersion.StartsWith('v')) {
        $normalizedVersion = $normalizedVersion.Substring(1)
    }

    $numericPart = ($normalizedVersion -split '-')[0]
    $parts = @($numericPart -split '\.' | Where-Object { $_ -match '^\d+$' })

    while ($parts.Count -lt 3) {
        $parts += '0'
    }

    return "{0}.{1}.{2}" -f [int]$parts[0], [int]$parts[1], [int]$parts[2]
}

$projectRoot = Split-Path -Parent $PSScriptRoot
$appProject = Join-Path $projectRoot "SevenHinos.csproj"
$wixProject = Join-Path $projectRoot "Installer\SevenHinos.Installer.wixproj"
$generatedDir = Join-Path $projectRoot "Installer\Generated"
$generatedFile = Join-Path $generatedDir "BuildVariables.wxi"
$publishDirFull = Resolve-FullPath -BasePath $projectRoot -PathValue $PublishDir
$installerDirFull = Resolve-FullPath -BasePath $projectRoot -PathValue $InstallerDir

$publishArgs = @{
    Configuration = $Configuration
    Runtime = $Runtime
    OutputDir = $PublishDir
}

if (-not [string]::IsNullOrWhiteSpace($VersionOverride)) {
    $publishArgs["VersionOverride"] = $VersionOverride
}

& (Join-Path $PSScriptRoot "Publish-Portable.ps1") @publishArgs

$rawVersion = if (-not [string]::IsNullOrWhiteSpace($VersionOverride)) {
    $VersionOverride.Trim()
}
else {
    $versionOutput = @(dotnet msbuild $appProject -nologo -getProperty:Version)
    ($versionOutput | Where-Object { $_ } | Select-Object -Last 1).Trim()
}

$msiVersion = Convert-ToMsiVersion -VersionText $rawVersion

New-Item -ItemType Directory -Path $generatedDir -Force | Out-Null

$escapedPublishDir = [System.Security.SecurityElement]::Escape($publishDirFull)
$wxiContent = @"
<?xml version="1.0" encoding="utf-8"?>
<Include>
  <?define ProductVersion = "$msiVersion" ?>
  <?define PublishDir = "$escapedPublishDir" ?>
</Include>
"@

Set-Content -Path $generatedFile -Value $wxiContent -Encoding utf8

if (Test-Path $installerDirFull) {
    Remove-Item $installerDirFull -Recurse -Force
}

New-Item -ItemType Directory -Path $installerDirFull -Force | Out-Null

dotnet build $wixProject -c $Configuration -p:Platform=x64 -p:InstallerPlatform=x64 -o $installerDirFull

$msi = Get-ChildItem -Path $installerDirFull -Filter *.msi -Recurse |
    Sort-Object LastWriteTimeUtc -Descending |
    Select-Object -First 1

if (-not $msi) {
    throw "Nenhum arquivo MSI foi gerado."
}

$versionedMsiPath = Join-Path $installerDirFull "7Hinos-Setup-$msiVersion.msi"
if ($msi.FullName -ne $versionedMsiPath) {
    Move-Item -Path $msi.FullName -Destination $versionedMsiPath -Force
    $msi = Get-Item $versionedMsiPath
}

Write-Host "Installer generated at: $($msi.FullName)"