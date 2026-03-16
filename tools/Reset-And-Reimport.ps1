<#
.SYNOPSIS
    Apaga todas as músicas e slides do banco 7Hinos e reimporta tudo do LouvorJA.

.DESCRIPTION
    1. Faz backup do banco atual (.bak com timestamp)
    2. Limpa as tabelas Songs + SongSlides (mantém FileAssets intacto)
    3. Executa Import-LouvorJA.ps1 para reimportar tudo do zero

.EXAMPLE
    .\Reset-And-Reimport.ps1
    .\Reset-And-Reimport.ps1 -SkipBackup
#>
param(
    [string] $SevenHinosDb = "$env:APPDATA\7Hinos\7hinos.db",
    [string] $LjaDb        = "C:\Program Files (x86)\Louvor JA\config\database.db",
    [switch] $SkipBackup
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not (Get-Module -ListAvailable -Name PSSQLite)) {
    Write-Host "Instalando PSSQLite..." -ForegroundColor Yellow
    Install-Module PSSQLite -Scope CurrentUser -Force
}
Import-Module PSSQLite -ErrorAction Stop

if (-not (Test-Path $SevenHinosDb)) {
    Write-Error "Banco 7Hinos não encontrado: $SevenHinosDb`nRode o app ao menos uma vez."
    exit 1
}

# ── Contagens antes ───────────────────────────────────────────────────────────

$beforeSongs  = (Invoke-SqliteQuery -DataSource $SevenHinosDb -Query "SELECT COUNT(*) AS cnt FROM Songs").cnt
$beforeSlides = (Invoke-SqliteQuery -DataSource $SevenHinosDb -Query "SELECT COUNT(*) AS cnt FROM SongSlides").cnt
Write-Host "Estado atual: $beforeSongs músicas, $beforeSlides slides" -ForegroundColor DarkGray

# ── Backup ────────────────────────────────────────────────────────────────────

if (-not $SkipBackup) {
    $stamp  = Get-Date -Format "yyyyMMdd_HHmmss"
    $bakDir = Split-Path $SevenHinosDb
    $bakFile = Join-Path $bakDir "7hinos_backup_$stamp.db"
    Copy-Item $SevenHinosDb $bakFile -Force
    Write-Host "✓ Backup salvo em: $bakFile" -ForegroundColor Green
}

# ── Limpa tabelas de música (mantém FileAssets) ───────────────────────────────

Write-Host "`nRecriando schema Songs + SongSlides..." -ForegroundColor Yellow

# Disable FK constraints so we can drop tables freely
Invoke-SqliteQuery -DataSource $SevenHinosDb -Query "PRAGMA foreign_keys = OFF"

Invoke-SqliteQuery -DataSource $SevenHinosDb -Query "DROP TABLE IF EXISTS `"SongSlides`""
Invoke-SqliteQuery -DataSource $SevenHinosDb -Query "DROP TABLE IF EXISTS `"Songs`""
Invoke-SqliteQuery -DataSource $SevenHinosDb -Query "DELETE FROM sqlite_sequence WHERE name IN ('Songs','SongSlides')" 2>$null

Invoke-SqliteQuery -DataSource $SevenHinosDb -Query @"
CREATE TABLE "Songs" (
    "Id"                   INTEGER NOT NULL CONSTRAINT "PK_Songs" PRIMARY KEY AUTOINCREMENT,
    "Title"                TEXT    NOT NULL,
    "Album"                TEXT    NOT NULL DEFAULT '',
    "Lyrics"               TEXT    NOT NULL DEFAULT '',
    "YoutubeUrl"           TEXT    NULL,
    "AudioFilePath"        TEXT    NULL,
    "InstrumentalFilePath" TEXT    NULL,
    "CreatedAt"            TEXT    NOT NULL,
    "IsFavorite"           INTEGER NOT NULL
)
"@

Invoke-SqliteQuery -DataSource $SevenHinosDb -Query @"
CREATE TABLE "SongSlides" (
    "Id"         INTEGER NOT NULL CONSTRAINT "PK_SongSlides" PRIMARY KEY AUTOINCREMENT,
    "SongId"     INTEGER NOT NULL,
    "Order"      INTEGER NOT NULL,
    "Content"    TEXT    NOT NULL,
    "AuxContent" TEXT    NULL,
    "Time"       INTEGER NOT NULL DEFAULT 0,
    "ShowSlide"  INTEGER NOT NULL,
    CONSTRAINT "FK_SongSlides_Songs_SongId" FOREIGN KEY ("SongId") REFERENCES "Songs" ("Id") ON DELETE CASCADE
)
"@
Invoke-SqliteQuery -DataSource $SevenHinosDb -Query "CREATE UNIQUE INDEX IF NOT EXISTS `"IX_SongSlides_SongId_Order`" ON `"SongSlides`" (`"SongId`", `"Order`")"
Invoke-SqliteQuery -DataSource $SevenHinosDb -Query "PRAGMA foreign_keys = ON"

Write-Host "  Schema atualizado (Time agora é INTEGER milissegundos)." -ForegroundColor DarkGray

# ── Reimporta ─────────────────────────────────────────────────────────────────

Write-Host "`nIniciando reimportação do LouvorJA..." -ForegroundColor Cyan

$importScript = Join-Path $PSScriptRoot "Import-LouvorJA.ps1"
if (-not (Test-Path $importScript)) {
    Write-Error "Import-LouvorJA.ps1 não encontrado em: $importScript"
    exit 1
}

& $importScript -LjaDb $LjaDb -SevenHinosDb $SevenHinosDb

# ── Resultado final ───────────────────────────────────────────────────────────

$afterSongs  = (Invoke-SqliteQuery -DataSource $SevenHinosDb -Query "SELECT COUNT(*) AS cnt FROM Songs").cnt
$afterSlides = (Invoke-SqliteQuery -DataSource $SevenHinosDb -Query "SELECT COUNT(*) AS cnt FROM SongSlides").cnt

Write-Host ""
Write-Host "═══════════════════════════════════════" -ForegroundColor DarkGray
Write-Host " Antes : $beforeSongs músicas  |  $beforeSlides slides" -ForegroundColor DarkGray
Write-Host " Depois: $afterSongs músicas  |  $afterSlides slides" -ForegroundColor Green
Write-Host "═══════════════════════════════════════" -ForegroundColor DarkGray
