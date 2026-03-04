// Sample query of LouvorJA database structure
// Run with: dotnet script tools/SampleLja.csx
// Or temporarily add to 7Hinos project and run

#r "nuget: Microsoft.Data.Sqlite, 9.0.0"
using Microsoft.Data.Sqlite;

var dbPath = @"C:\Program Files (x86)\Louvor JA\config\database.db";
using var con = new SqliteConnection($"Data Source={dbPath};Mode=ReadOnly");
con.Open();

// Sample 3 songs from MUSICAS
Console.WriteLine("=== MUSICAS (3 rows) ===");
using (var cmd = con.CreateCommand())
{
    cmd.CommandText = "SELECT ID, NOME, ALBUM, URL, URL_INSTRUMENTAL, IDIOMA FROM MUSICAS LIMIT 3";
    using var r = cmd.ExecuteReader();
    while (r.Read())
    {
        Console.WriteLine($"ID={r["ID"]} | NOME={r["NOME"]} | ALBUM={r["ALBUM"]} | URL={r["URL"]} | URL_INSTR={r["URL_INSTRUMENTAL"]} | IDIOMA={r["IDIOMA"]}");
    }
}

// ALBUM names
Console.WriteLine("\n=== ALBUM (5 rows) ===");
using (var cmd = con.CreateCommand())
{
    cmd.CommandText = "SELECT * FROM ALBUM LIMIT 5";
    using var r = cmd.ExecuteReader();
    // Print column names
    Console.WriteLine(string.Join(" | ", Enumerable.Range(0, r.FieldCount).Select(i => r.GetName(i))));
    while (r.Read())
        Console.WriteLine(string.Join(" | ", Enumerable.Range(0, r.FieldCount).Select(i => r[i]?.ToString() ?? "NULL")));
}

// MUSICAS_LETRA for song ID=1 (slides)
Console.WriteLine("\n=== MUSICAS_LETRA for first song ===");
int firstId;
using (var cmd = con.CreateCommand())
{
    cmd.CommandText = "SELECT ID FROM MUSICAS ORDER BY ID LIMIT 1";
    firstId = Convert.ToInt32(cmd.ExecuteScalar());
}
using (var cmd = con.CreateCommand())
{
    cmd.CommandText = "SELECT ID, MUSICA, ORDEM, LETRA, LETRA_AUX, TEMPO FROM MUSICAS_LETRA WHERE MUSICA = $id ORDER BY ORDEM";
    cmd.Parameters.AddWithValue("$id", firstId);
    using var r = cmd.ExecuteReader();
    while (r.Read())
        Console.WriteLine($"ORDEM={r["ORDEM"]} TEMPO={r["TEMPO"]}\nLETRA=[{r["LETRA"]}]\nLETRA_AUX=[{r["LETRA_AUX"]}]\n");
}

Console.WriteLine("\n=== MUSICAS total count ===");
using (var cmd = con.CreateCommand())
{
    cmd.CommandText = "SELECT COUNT(*) FROM MUSICAS";
    Console.WriteLine("Songs: " + cmd.ExecuteScalar());
}
using (var cmd = con.CreateCommand())
{
    cmd.CommandText = "SELECT COUNT(*) FROM MUSICAS_LETRA";
    Console.WriteLine("Slides: " + cmd.ExecuteScalar());
}
using (var cmd = con.CreateCommand())
{
    cmd.CommandText = "SELECT COUNT(*) FROM ALBUM";
    Console.WriteLine("Albums: " + cmd.ExecuteScalar());
}
