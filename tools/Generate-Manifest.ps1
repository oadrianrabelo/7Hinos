<#
.SYNOPSIS
    Gera o arquivo manifest.json para o sistema de validação/download do 7Hinos.

.DESCRIPTION
    Varre uma pasta de áudios, calcula o SHA-256 de cada arquivo e gera um
    manifest.json pronto para ser publicado (GitHub Releases, Backblaze B2, etc).

.PARAMETER AudioFolder
    Pasta raiz onde estão os arquivos de áudio (pode ter subpastas por álbum).

.PARAMETER BaseUrl
    URL base onde os arquivos serão hospedados (Oracle Object Storage).
    Ex: https://objectstorage.sa-saopaulo-1.oraclecloud.com/n/SEU-NAMESPACE/b/7hinos-asset/o
    O caminho relativo de cada arquivo será concatenado a essa URL.
    Arquivos em subpastas (álbuns) ficam: BaseUrl/Album/Musica.mp3

.PARAMETER OutputFile
    Caminho do manifest.json a ser gerado. Padrão: .\manifest.json
    Após gerar, suba também esse arquivo para o bucket na raiz.

.PARAMETER Version
    Versão do manifesto. Padrão: 1.0.0

.PARAMETER Extensions
    Extensões de arquivo a incluir. Padrão: mp3, ogg, flac, wav

.EXAMPLE
    .\Generate-Manifest.ps1 `
        -AudioFolder "C:\Program Files (x86)\Louvor JA\config\musicas" `
        -BaseUrl "https://objectstorage.sa-saopaulo-1.oraclecloud.com/n/greafinzbo0u/b/7hinos-asset/o" `
        -Version "1.0.0" `
        -OutputFile ".\manifest.json"
    # Depois suba o manifest.json gerado para o bucket também (raiz).
#>
param(
    [Parameter(Mandatory)]
    [string] $AudioFolder,

    [Parameter(Mandatory)]
    [string] $BaseUrl,

    [string] $OutputFile = ".\manifest.json",
    [string] $Version    = "1.0.0",
    [string[]] $Extensions = @("mp3", "ogg", "flac", "wav")
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ─── Helpers ────────────────────────────────────────────────────────────────

function Get-Sha256([string]$Path) {
    $sha = [System.Security.Cryptography.SHA256]::Create()
    $stream = [System.IO.File]::OpenRead($Path)
    try {
        $bytes = $sha.ComputeHash($stream)
        return ([BitConverter]::ToString($bytes) -replace '-','').ToLower()
    } finally {
        $stream.Dispose()
        $sha.Dispose()
    }
}

function Get-Category([string]$RelativePath) {
    # Use the first sub-folder as category (album name)
    $parts = $RelativePath -split '[/\\]'
    if ($parts.Count -gt 1) { return $parts[0] }
    return $null
}

# ─── Scan files ─────────────────────────────────────────────────────────────

$AudioFolder = (Resolve-Path $AudioFolder).Path
$BaseUrl     = $BaseUrl.TrimEnd('/')

$pattern = $Extensions | ForEach-Object { "*.$_" }
$files   = Get-ChildItem -Path $AudioFolder -Recurse -File |
           Where-Object { $Extensions -contains $_.Extension.TrimStart('.').ToLower() } |
           Sort-Object FullName

if ($files.Count -eq 0) {
    Write-Error "Nenhum arquivo de áudio encontrado em: $AudioFolder"
    exit 1
}

Write-Host "Encontrados $($files.Count) arquivos. Calculando SHA-256..." -ForegroundColor Cyan

$totalSize = ($files | Measure-Object -Property Length -Sum).Sum
Write-Host ("Tamanho total: {0:N2} GB" -f ($totalSize / 1GB)) -ForegroundColor DarkGray

# ─── Build assets ────────────────────────────────────────────────────────────

$assets = [System.Collections.Generic.List[object]]::new()
$i = 0

foreach ($file in $files) {
    $i++
    $relativePath = $file.FullName.Substring($AudioFolder.Length).TrimStart('\', '/').Replace('\', '/')
    $category     = Get-Category $relativePath

    Write-Progress `
        -Activity "Calculando SHA-256" `
        -Status "$i / $($files.Count): $($file.Name)" `
        -PercentComplete ([int]($i / $files.Count * 100))

    $hash = Get-Sha256 $file.FullName
    # URL-encode each segment (handles spaces and special chars)
    $encodedPath = (($relativePath -split '/') |
                   ForEach-Object { [Uri]::EscapeDataString($_) }) -join '/'

    $assets.Add([pscustomobject]@{
        relativePath = $relativePath
        fileName     = $file.Name
        category     = $category
        url          = "$BaseUrl/$encodedPath"
        sha256       = $hash
        size         = $file.Length
    })

    Write-Host ("  [{0,4}/{1}] {2}" -f $i, $files.Count, $file.Name) -ForegroundColor Gray
}

Write-Progress -Activity "Calculando SHA-256" -Completed

# ─── Build manifest ──────────────────────────────────────────────────────────

$manifest = [pscustomobject]@{
    version      = $Version
    generatedAt  = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
    releaseNotes = "Atualização automática gerada em $(Get-Date -Format 'dd/MM/yyyy')"
    assets       = $assets
}

$json = $manifest | ConvertTo-Json -Depth 5 -EnumsAsStrings
[System.IO.File]::WriteAllText((Resolve-Path (Split-Path $OutputFile -Parent)).Path + '\' + (Split-Path $OutputFile -Leaf), $json, [System.Text.Encoding]::UTF8)

Write-Host ""
Write-Host "✓ Manifesto gerado: $OutputFile" -ForegroundColor Green
Write-Host "  Versão  : $Version"             -ForegroundColor DarkGray
Write-Host "  Arquivos: $($assets.Count)"     -ForegroundColor DarkGray
Write-Host ("  Tamanho : {0:N2} GB" -f ($totalSize / 1GB)) -ForegroundColor DarkGray
