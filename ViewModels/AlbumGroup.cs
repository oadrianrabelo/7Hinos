using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Text.RegularExpressions;
using SevenHinos.Models;

namespace SevenHinos.ViewModels;

// ─── Library sections (mirrors LouvorJA's top-nav categories) ────────────────

public enum LibrarySection
{
    Hymnal,      // Hinário Adventista (2022 — current)
    Hymnal1996,  // Hinário Adventista 1996 (classic)
    JA,          // CDs Oficiais/Ano: "YYYY - Title" annual albums
    Adoradores,  // Adoradores, Adoradores 2 … 5
    Cantores,    // Salmos, Filhos de Israel, Diferente, etc.
    CelebradeSP, // Celebra São Paulo 1 / 2 / 3
    Diversas,    // everything else
}

// ─── Section metadata ────────────────────────────────────────────────────────

public sealed record LibrarySectionInfo(
    LibrarySection Section,
    string         Label,    // two-line label for nav button
    string         Icon);

// Keep old enum alias so SongManagerViewModel still compiles
public enum AlbumCategory { Hymnal, JAYear, Other }

// ─── Per-album tree node ─────────────────────────────────────────────────────

/// <summary>A single album bucket shown as a tree-node in the song browser.</summary>
public sealed class AlbumGroup
{
    // ── Classification ───────────────────────────────────────────────────────

    private static readonly Regex _jaPattern =
        new(@"^\d{4} - ", RegexOptions.Compiled);

    private static readonly HashSet<string> _cantoresAlbums =
        new(StringComparer.OrdinalIgnoreCase)
        {
            "Salmos", "Salmos 2", "Filhos de Israel",
            "Jesus Luz do Mundo", "Diferente", "Magnífico Deus",
        };

    public static LibrarySection ClassifyAlbum(string album)
    {
        if (album.Equals("Hinário Adventista 2022", StringComparison.OrdinalIgnoreCase))
            return LibrarySection.Hymnal;

        if (album.Equals("Hinário Adventista", StringComparison.OrdinalIgnoreCase))
            return LibrarySection.Hymnal1996;

        if (_jaPattern.IsMatch(album))
            return LibrarySection.JA;

        if (album.StartsWith("Adoradores", StringComparison.OrdinalIgnoreCase))
            return LibrarySection.Adoradores;

        if (album.StartsWith("Celebra São Paulo", StringComparison.OrdinalIgnoreCase) ||
            album.StartsWith("Celebra Sao Paulo", StringComparison.OrdinalIgnoreCase))
            return LibrarySection.CelebradeSP;

        if (_cantoresAlbums.Contains(album))
            return LibrarySection.Cantores;

        return LibrarySection.Diversas;
    }

    // ── Section metadata (used to build the top-nav buttons) ─────────────────

    public static readonly IReadOnlyList<LibrarySectionInfo> Sections =
    [
        new(LibrarySection.Hymnal,      "Hinário\nAdventista", "♩"),
        new(LibrarySection.Hymnal1996,  "Hinário\nAdv. 1996",  "♩"),
        new(LibrarySection.JA,          "JA /\nMin. Música",   "★"),
        new(LibrarySection.Adoradores,  "Adoradores",          "♪"),
        new(LibrarySection.Cantores,    "Cantores",            "♬"),
        new(LibrarySection.CelebradeSP, "Celebra SP",          "♫"),
        new(LibrarySection.Diversas,    "Diversas",            "☰"),
    ];

    // ── AlbumGroup fields ────────────────────────────────────────────────────

    public string         AlbumName { get; init; } = string.Empty;
    public LibrarySection Section   { get; init; } = LibrarySection.Diversas;

    /// <summary>
    /// False for sections that contain a single well-known album (Hinários),
    /// so the redundant album name header is hidden in the tree.
    /// </summary>
    public bool ShowAlbumHeader =>
        Section is not (LibrarySection.Hymnal or LibrarySection.Hymnal1996);

    /// <summary>Songs in this album, ordered appropriately for the section.</summary>
    public ObservableCollection<Song> Songs { get; } = [];

    // ── Factory ───────────────────────────────────────────────────────────────

    /// <summary>
    /// Builds album groups from <paramref name="songs"/>, filtered to
    /// <paramref name="section"/> and text-searched with <paramref name="query"/>.
    /// </summary>
    public static IEnumerable<AlbumGroup> BuildSection(
        IEnumerable<Song> songs,
        LibrarySection    section,
        string?           query = null)
    {
        IEnumerable<Song> filtered = songs.Where(s => ClassifyAlbum(s.Album) == section);

        if (!string.IsNullOrWhiteSpace(query))
            filtered = filtered.Where(s =>
                s.Title.Contains(query, StringComparison.OrdinalIgnoreCase) ||
                s.Album.Contains(query, StringComparison.OrdinalIgnoreCase));

        var groups = filtered
            .GroupBy(s => string.IsNullOrWhiteSpace(s.Album) ? "(sem álbum)" : s.Album);

        // JA: newest-first; everything else: alphabetical
        IEnumerable<IGrouping<string, Song>> sortedGroups = section == LibrarySection.JA
            ? groups.OrderByDescending(g => g.Key)
            : groups.OrderBy(g => g.Key, StringComparer.CurrentCulture);

        bool isHymnal = section is LibrarySection.Hymnal or LibrarySection.Hymnal1996;

        foreach (var g in sortedGroups)
        {
            var ag = new AlbumGroup { AlbumName = g.Key, Section = section };

            // Hymnal: numeric sort so "3" comes before "10"
            IEnumerable<Song> ordered = isHymnal
                ? g.OrderBy(s => LeadingNumber(s.Title)).ThenBy(s => s.Title)
                : g.OrderBy(s => s.Title, StringComparer.CurrentCulture);

            foreach (var s in ordered) ag.Songs.Add(s);
            yield return ag;
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    private static int LeadingNumber(string title)
    {
        var m = Regex.Match(title, @"^(\d+)");
        return m.Success ? int.Parse(m.Groups[1].Value) : int.MaxValue;
    }

    /// <summary>
    /// Builds all sections in canonical order (Hinários → JA newest-first → others).
    /// Used by the catalog/editor view which displays everything at once.
    /// </summary>
    public static IEnumerable<AlbumGroup> BuildAll(IEnumerable<Song> songs, string? query = null)
    {
        var all = songs.ToList();
        // Hymnal 2022 first, then 1996, then JA newest-first, then the rest alphabetically
        var sectionOrder = new[]
        {
            LibrarySection.Hymnal,
            LibrarySection.Hymnal1996,
            LibrarySection.JA,
            LibrarySection.Adoradores,
            LibrarySection.Cantores,
            LibrarySection.CelebradeSP,
            LibrarySection.Diversas,
        };
        foreach (var section in sectionOrder)
            foreach (var ag in BuildSection(all, section, query))
                yield return ag;
    }
}
