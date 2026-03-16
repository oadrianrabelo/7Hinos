$sqlite = "c:\Users\adria\projetos\7Hinos\bin\Debug\net9.0\Microsoft.Data.Sqlite.dll"
Add-Type -Path $sqlite

$con = New-Object Microsoft.Data.Sqlite.SqliteConnection("Data Source='$env:APPDATA\7Hinos\7hinos.db';Mode=ReadOnly")
$con.Open()

$cmd = $con.CreateCommand()
$cmd.CommandText = "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
$r = $cmd.ExecuteReader()
$tables = @()
while ($r.Read()) { $tables += $r.GetString(0) }
$r.Close()
Write-Host "Tables: $($tables -join ', ')"

foreach ($t in $tables) {
    $cmd.CommandText = "PRAGMA table_info([$t])"
    $r = $cmd.ExecuteReader()
    $cols = @()
    while ($r.Read()) { $cols += "$($r['name'])[$($r['type'])]" }
    $r.Close()
    $cmd.CommandText = "SELECT COUNT(*) FROM [$t]"
    $n = $cmd.ExecuteScalar()
    Write-Host ""
    Write-Host "${t} ($n rows):"
    Write-Host "  $($cols -join ' | ')"
}
$con.Close()
