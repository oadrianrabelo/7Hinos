param(
    [string] $Configuration = "Release",
    [string] $Runtime = "win-x64",
    [string] $OutputDir = ".artifacts/publish/win-x64-portable",
    [string] $VersionOverride = ""
)

$projectRoot = Split-Path -Parent $PSScriptRoot
$projectFile = Join-Path $projectRoot "SevenHinos.csproj"
$publishDir = Join-Path $projectRoot $OutputDir
$zipPath = "$publishDir.zip"

if (Test-Path $publishDir) {
    Remove-Item $publishDir -Recurse -Force
}

$publishArgs = @(
    "publish",
    $projectFile,
    "-c", $Configuration,
    "-r", $Runtime,
    "--self-contained", "true",
    "-p:PublishSingleFile=false",
    "-p:PublishTrimmed=false",
    "-p:DebugType=None",
    "-p:DebugSymbols=false",
    "-o", $publishDir
)

if (-not [string]::IsNullOrWhiteSpace($VersionOverride)) {
    $publishArgs += "-p:MinVerVersionOverride=$VersionOverride"
    $publishArgs += "-p:Version=$VersionOverride"
    $publishArgs += "-p:InformationalVersion=$VersionOverride"
}

dotnet @publishArgs

if (Test-Path $zipPath) {
    Remove-Item $zipPath -Force
}

Compress-Archive -Path (Join-Path $publishDir "*") -DestinationPath $zipPath -Force

Write-Host "Portable publish generated at: $publishDir"
Write-Host "ZIP package generated at: $zipPath"
