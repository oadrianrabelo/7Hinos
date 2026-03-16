param(
    [string] $Configuration = "Release",
    [string] $Runtime = "win-x64",
    [string] $OutputDir = ".artifacts/publish/win-x64-portable"
)

$projectRoot = Split-Path -Parent $PSScriptRoot
$projectFile = Join-Path $projectRoot "SevenHinos.csproj"
$publishDir = Join-Path $projectRoot $OutputDir
$zipPath = "$publishDir.zip"

if (Test-Path $publishDir) {
    Remove-Item $publishDir -Recurse -Force
}

dotnet publish $projectFile `
    -c $Configuration `
    -r $Runtime `
    --self-contained true `
    -p:PublishSingleFile=false `
    -p:PublishTrimmed=false `
    -p:DebugType=None `
    -p:DebugSymbols=false `
    -o $publishDir

if (Test-Path $zipPath) {
    Remove-Item $zipPath -Force
}

Compress-Archive -Path (Join-Path $publishDir "*") -DestinationPath $zipPath -Force

Write-Host "Portable publish generated at: $publishDir"
Write-Host "ZIP package generated at: $zipPath"