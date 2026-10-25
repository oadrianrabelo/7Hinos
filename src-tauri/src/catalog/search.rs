//! Busca por título, número do hino ou trecho da letra.
//!
//! Pensada para quem digita sem acento, com erro de caixa ou só lembra de um
//! pedaço do refrão. O índice inteiro (≈2 mil músicas) cabe na memória, então
//! uma varredura linear é rápida e não precisa de mais nada.

use super::model::SongSummary;

pub struct IndexedSong {
    pub summary: SongSummary,
    title: String,
    lyric: String,
}

impl IndexedSong {
    pub fn new(summary: SongSummary, lyric: &str) -> Self {
        Self {
            title: normalize(&summary.title),
            lyric: normalize(lyric),
            summary,
        }
    }
}

/// Minúsculas, sem acentos e só letras/números separados por um espaço.
pub fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pending_space = false;
    for c in text.chars().flat_map(char::to_lowercase) {
        let c = fold_accent(c);
        if c.is_alphanumeric() {
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            pending_space = false;
            out.push(c);
        } else {
            pending_space = true;
        }
    }
    out
}

fn fold_accent(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ã' | 'ä' | 'å' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'ç' => 'c',
        'ñ' => 'n',
        'ý' | 'ÿ' => 'y',
        c => c,
    }
}

pub fn search(index: &[IndexedSong], query: &str, limit: usize) -> Vec<SongSummary> {
    let query = normalize(query);
    if query.is_empty() {
        return Vec::new();
    }
    let number: Option<u32> = query.parse().ok();
    let words: Vec<&str> = query.split(' ').collect();

    let mut hits: Vec<(u8, &IndexedSong)> = index
        .iter()
        .filter_map(|song| score(song, &query, &words, number).map(|s| (s, song)))
        .collect();
    hits.sort_by(|(a, x), (b, y)| a.cmp(b).then_with(|| x.title.cmp(&y.title)));
    hits.into_iter()
        .take(limit)
        .map(|(_, song)| song.summary.clone())
        .collect()
}

/// Menor é melhor. `None` significa que a música não combina.
fn score(song: &IndexedSong, query: &str, words: &[&str], number: Option<u32>) -> Option<u8> {
    if number.is_some() && song.summary.number == number {
        return Some(0);
    }
    if song.title == query {
        return Some(1);
    }
    if song.title.starts_with(query) {
        return Some(2);
    }
    if contains_at_word(&song.title, query) {
        return Some(3);
    }
    if song.title.contains(query) {
        return Some(4);
    }
    if contains_at_word(&song.lyric, query) {
        return Some(5);
    }
    let all_words = words
        .iter()
        .all(|w| song.title.contains(w) || song.lyric.contains(w));
    all_words.then_some(6)
}

fn contains_at_word(haystack: &str, needle: &str) -> bool {
    haystack
        .match_indices(needle)
        .any(|(i, _)| i == 0 || haystack.as_bytes()[i - 1] == b' ')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn song(id: u32, title: &str, number: Option<u32>, lyric: &str) -> IndexedSong {
        IndexedSong::new(
            SongSummary {
                id,
                title: title.into(),
                subtitle: String::new(),
                number,
                has_instrumental: false,
                duration_ms: None,
            },
            lyric,
        )
    }

    fn index() -> Vec<IndexedSong> {
        vec![
            song(1, "Santo, Santo, Santo!", Some(1), "Deus Pai eterno"),
            song(2, "Castelo Forte", Some(18), "castelo forte é nosso Deus"),
            song(3, "Nosso Sol é Jesus", None, "O nosso sol veio iluminar"),
            song(4, "Sol da Justiça", Some(118), "brilha"),
            song(5, "Ó Jesus, Te Amo", None, "nosso sol"),
        ]
    }

    fn ids(query: &str) -> Vec<u32> {
        search(&index(), query, 10).iter().map(|s| s.id).collect()
    }

    #[test]
    fn normalizes_accents_case_and_punctuation() {
        assert_eq!(normalize("  Ó Jesus,  Te AMO! "), "o jesus te amo");
        assert_eq!(normalize("Ação—Graças"), "acao gracas");
    }

    #[test]
    fn hymn_number_comes_first() {
        assert_eq!(ids("18"), [2]);
        assert_eq!(ids("1")[0], 1);
    }

    #[test]
    fn ranks_title_matches_before_lyrics() {
        assert_eq!(ids("sol"), [4, 3, 5]);
        assert_eq!(ids("nosso sol"), [3, 5]);
    }

    #[test]
    fn finds_lyrics_without_accents_and_scattered_words() {
        assert_eq!(ids("iluminar"), [3]);
        assert_eq!(ids("forte deus"), [2]);
        assert!(ids("inexistente").is_empty());
        assert!(ids("  ").is_empty());
    }
}
