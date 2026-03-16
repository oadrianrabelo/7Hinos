<#
.SYNOPSIS
    Importa músicas do banco LouvorJA para o banco 7Hinos.

.DESCRIPTION
    Lê MUSICAS + MUSICAS_LETRA do database.db do LouvorJA e insere
    na tabela Songs + SongSlides do 7Hinos. Já existentes são ignorados.
    Requer o módulo PSSQLite: Install-Module PSSQLite -Scope CurrentUser

.EXAMPLE
    .\Import-LouvorJA.ps1
    .\Import-LouvorJA.ps1 -LjaDb "D:\outro\database.db"
#>
param(
    [string] $LjaDb        = "C:\Program Files (x86)\Louvor JA\config\database.db",
    [string] $SevenHinosDb = "$env:APPDATA\7Hinos\7hinos.db",
    [string] $MusicasRoot  = "C:\Program Files (x86)\Louvor JA\config\musicas"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ── Dependências ─────────────────────────────────────────────────────────────

if (-not (Get-Module -ListAvailable -Name PSSQLite)) {
    Write-Host "Instalando PSSQLite..." -ForegroundColor Yellow
    Install-Module PSSQLite -Scope CurrentUser -Force
}
Import-Module PSSQLite -ErrorAction Stop

# ── Verificações iniciais ─────────────────────────────────────────────────────

if (-not (Test-Path $LjaDb))        { Write-Error "LouvorJA DB não encontrado: $LjaDb"; exit 1 }
if (-not (Test-Path $SevenHinosDb)) { Write-Error "7Hinos DB não encontrado: $SevenHinosDb`nRode o app 7Hinos ao menos uma vez."; exit 1 }

Write-Host "LouvorJA DB  : $LjaDb" -ForegroundColor DarkGray
Write-Host "7Hinos DB    : $SevenHinosDb" -ForegroundColor DarkGray

# ── Lê slides do LouvorJA (MUSICAS_LETRA) ────────────────────────────────────

Write-Host "`nLendo slides (MUSICAS_LETRA)..." -ForegroundColor Cyan
$slideRows = Invoke-SqliteQuery -DataSource $LjaDb `
    -Query "SELECT MUSICA, ORDEM, LETRA, LETRA_AUX, EXIBE_SLIDE, TEMPO FROM MUSICAS_LETRA ORDER BY MUSICA, ORDEM"

$slides = @{}
foreach ($row in $slideRows) {
    $mid = [int]$row.MUSICA
    if (-not $slides.ContainsKey($mid)) { $slides[$mid] = [System.Collections.Generic.List[hashtable]]::new() }
    # TEMPO comes from PSSQLite as a DateTime string like "03/03/2026 00:00:08"
    # Extract the TimeOfDay portion and convert to total milliseconds
    $tempoMs = if ($null -eq $row.TEMPO -or $row.TEMPO -eq '') { 0L } else {
        $tempoStr = "$($row.TEMPO)"
        try {
            if ($tempoStr -match '\d{2}/\d{2}/\d{4}') {
                # PSSQLite returned a DateTime — use TimeOfDay
                [long]([DateTime]::Parse($tempoStr).TimeOfDay.TotalMilliseconds)
            } else {
                [long]([TimeSpan]::Parse($tempoStr).TotalMilliseconds)
            }
        } catch { 0L }
    }
    $slides[$mid].Add(@{
        Order      = if ($null -eq $row.ORDEM)       { 0 }  else { [int]$row.ORDEM }
        Letra      = if ($null -eq $row.LETRA)       { '' } else { "$($row.LETRA)" }
        LetraAux   = if ($null -eq $row.LETRA_AUX)   { '' } else { "$($row.LETRA_AUX)" }
        ShowSlide  = if ($null -eq $row.EXIBE_SLIDE)  { 1 }  else { [int]$row.EXIBE_SLIDE }
        TempoMs    = $tempoMs
    })
}
Write-Host "  $($slides.Keys.Count) músicas com slides." -ForegroundColor DarkGray

# ── Lê músicas existentes no 7Hinos ──────────────────────────────────────────

$existingRows = Invoke-SqliteQuery -DataSource $SevenHinosDb -Query "SELECT Title, Album FROM Songs"
$existing = [System.Collections.Generic.HashSet[string]]::new()
foreach ($row in $existingRows) { [void]$existing.Add("$($row.Title)|$($row.Album)") }
Write-Host "Músicas já no banco 7Hinos: $($existing.Count)" -ForegroundColor DarkGray

# ── Lê músicas do LouvorJA ────────────────────────────────────────────────────

Write-Host "Lendo MUSICAS do LouvorJA..." -ForegroundColor Cyan
$songs = @(Invoke-SqliteQuery -DataSource $LjaDb `
    -Query "SELECT ID, NOME, ALBUM, URL, URL_INSTRUMENTAL FROM MUSICAS ORDER BY ALBUM, NOME")
Write-Host "  $($songs.Count) músicas encontradas." -ForegroundColor DarkGray

# ── Abre conexão destino para inserção em batch ──────────────────────────────

$dstConn = New-SQLiteConnection -DataSource $SevenHinosDb
if ($dstConn.State -ne [System.Data.ConnectionState]::Open) { $dstConn.Open() }

$tx          = $dstConn.BeginTransaction()
$songCmd     = $dstConn.CreateCommand()
$songCmd.Transaction = $tx
$songCmd.CommandText = @"
INSERT INTO Songs (Title, Album, Lyrics, IsFavorite, YoutubeUrl, AudioFilePath, InstrumentalFilePath, CreatedAt)
VALUES (@title, @album, @lyrics, 0, NULL, @audio, @instrumental, @created);
SELECT last_insert_rowid();
"@
[void]$songCmd.Parameters.AddWithValue("@title",        "")
[void]$songCmd.Parameters.AddWithValue("@album",        "")
[void]$songCmd.Parameters.AddWithValue("@lyrics",       "")
[void]$songCmd.Parameters.AddWithValue("@audio",        "")
[void]$songCmd.Parameters.AddWithValue("@instrumental", "")
[void]$songCmd.Parameters.AddWithValue("@created",      "")

$slideCmd    = $dstConn.CreateCommand()
$slideCmd.Transaction = $tx
$slideCmd.CommandText = @"
INSERT INTO SongSlides (SongId, "Order", Content, AuxContent, Time, ShowSlide)
VALUES (@songId, @order, @content, @aux, @time, @showslide)
"@
[void]$slideCmd.Parameters.AddWithValue("@songId",    0)
[void]$slideCmd.Parameters.AddWithValue("@order",     0)
[void]$slideCmd.Parameters.AddWithValue("@content",   "")
[void]$slideCmd.Parameters.AddWithValue("@aux",       "")
[void]$slideCmd.Parameters.AddWithValue("@time",      "00:00:00")
[void]$slideCmd.Parameters.AddWithValue("@showslide", 1)

$imported = 0
$skipped  = 0
$total    = $songs.Count
$now      = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.0000000Z")

for ($i = 0; $i -lt $total; $i++) {
    $song      = $songs[$i]
    $albumName = if ($null -eq $song.ALBUM) { "" } else { "$($song.ALBUM)" }
    $title     = if ($null -eq $song.NOME)  { "" } else { "$($song.NOME)" }
    $key       = "$title|$albumName"

    if ($existing.Contains($key)) { $skipped++; continue }

    $songSlides = if ($null -ne $song.ID -and $slides.ContainsKey([int]$song.ID)) { $slides[[int]$song.ID] } else { @() }
    $lyrics     = ($songSlides | ForEach-Object { $_.Letra }) -join "`n`n"

    # Audio paths (optional)
    $audioPath = [DBNull]::Value
    $instrPath = [DBNull]::Value
    if ($song.URL) {
        $p = Join-Path $MusicasRoot ($song.URL.ToString().Replace('/', '\'))
        if (Test-Path $p) { $audioPath = $p }
    }
    if ($song.URL_INSTRUMENTAL) {
        $p = Join-Path $MusicasRoot ($song.URL_INSTRUMENTAL.ToString().Replace('/', '\'))
        if (Test-Path $p) { $instrPath = $p }
    }

    $songCmd.Parameters["@title"].Value        = $title
    $songCmd.Parameters["@album"].Value        = $albumName
    $songCmd.Parameters["@lyrics"].Value       = $lyrics
    $songCmd.Parameters["@audio"].Value        = $audioPath
    $songCmd.Parameters["@instrumental"].Value = $instrPath
    $songCmd.Parameters["@created"].Value      = $now

    $newId = [long]$songCmd.ExecuteScalar()

    foreach ($sl in $songSlides) {
        $slideCmd.Parameters["@songId"].Value    = $newId
        $slideCmd.Parameters["@order"].Value      = $sl.Order
        $slideCmd.Parameters["@content"].Value    = $sl.Letra
        $slideCmd.Parameters["@aux"].Value        = if ($sl.LetraAux) { $sl.LetraAux } else { [DBNull]::Value }
        $slideCmd.Parameters["@time"].Value       = $sl.TempoMs   # stored as ms (long integer)
        $slideCmd.Parameters["@showslide"].Value  = $sl.ShowSlide
        [void]$slideCmd.ExecuteNonQuery()
    }

    $imported++
    [void]$existing.Add($key)

    if (($i + 1) % 100 -eq 0) {
        Write-Progress -Activity "Importando músicas" `
            -Status "$($i+1) / $total  |  $title" `
            -PercentComplete ([int](($i + 1) / $total * 100))
    }
}

Write-Progress -Activity "Importando músicas" -Completed
$tx.Commit()
$dstConn.Close()

Write-Host "✓ Importação concluída!" -ForegroundColor Green
Write-Host "  Importadas : $imported" -ForegroundColor White
Write-Host "  Ignoradas  : $skipped (já existiam)" -ForegroundColor DarkGray
Write-Host "  Total LJA  : $total" -ForegroundColor DarkGray
