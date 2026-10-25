//! O catálogo como o app o entende, já traduzido do formato do LouvorJA.

use serde::{Deserialize, Serialize};

use super::raw::{parse_time_ms, RawAlbum, RawAlbumRef, RawMusic, RawSongRow};
use crate::media::{Media, MediaCache};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[default]
    Pt,
    Es,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::Pt => "pt",
            Lang::Es => "es",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: u32,
    pub name: String,
    pub slug: String,
    pub albums: Vec<AlbumSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumSummary {
    pub id: u32,
    pub name: String,
    pub subtitle: Option<String>,
    pub color: Option<String>,
    pub cover: Option<Media>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: u32,
    pub name: String,
    pub color: Option<String>,
    pub cover: Option<Media>,
    pub songs: Vec<SongSummary>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SongSummary {
    pub id: u32,
    pub title: String,
    /// Onde a música aparece (nomes dos álbuns), para diferenciar homônimas.
    pub subtitle: String,
    /// Número do hino no hinário ou da faixa no álbum.
    pub number: Option<u32>,
    pub has_instrumental: bool,
    pub duration_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: u32,
    pub title: String,
    pub subtitle: String,
    pub cover: Option<Media>,
    pub audio: Option<Media>,
    pub instrumental: Option<Media>,
    pub duration_ms: Option<u32>,
    pub slides: Vec<Slide>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SlideKind {
    Title,
    Lyric,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Slide {
    pub kind: SlideKind,
    pub text: String,
    /// Texto de apoio (tradução, referência) mostrado acima da letra.
    pub aux: Option<String>,
    pub image: Option<Media>,
    /// Ancoragem do fundo numa grade 3×3, de 1 (topo-esquerda) a 9.
    pub image_position: u8,
    /// Repete a letra do slide anterior (normalmente o refrão).
    pub repeat: bool,
    pub time_ms: Option<u32>,
    pub instrumental_time_ms: Option<u32>,
}

fn non_empty(s: &str) -> Option<String> {
    let s = s.trim();
    (!s.is_empty()).then(|| s.to_owned())
}

pub fn album_summary(raw: &RawAlbumRef, media: &MediaCache) -> AlbumSummary {
    AlbumSummary {
        id: raw.id_album,
        name: raw.name.trim().to_owned(),
        subtitle: non_empty(&raw.subtitle),
        color: non_empty(&raw.color),
        cover: media.resolve(&raw.url_image),
    }
}

pub fn song_summary(raw: &RawSongRow, number: Option<u32>) -> SongSummary {
    SongSummary {
        id: raw.id_music,
        title: raw.name.trim().to_owned(),
        subtitle: raw.albums_names.trim().to_owned(),
        number: number.filter(|n| *n > 0),
        has_instrumental: raw.has_instrumental_music,
        duration_ms: parse_time_ms(&raw.duration).filter(|ms| *ms > 0),
    }
}

pub fn album(raw: &RawAlbum, media: &MediaCache) -> Album {
    let mut songs: Vec<SongSummary> = raw
        .musics
        .iter()
        .map(|row| SongSummary {
            subtitle: raw.name.trim().to_owned(),
            ..song_summary(row, Some(row.track))
        })
        .collect();
    songs.sort_by_key(|s| (s.number.unwrap_or(u32::MAX), s.id));
    Album {
        id: raw.id_album,
        name: raw.name.trim().to_owned(),
        color: non_empty(&raw.color),
        cover: media.resolve(&raw.url_image),
        songs,
    }
}

/// Monta os slides do jeito que o LouvorJA apresenta: primeiro a capa com o
/// título, depois cada estrofe visível em ordem. Um slide sem imagem própria
/// herda o fundo do anterior.
pub fn song(raw: &RawMusic, media: &MediaCache) -> Song {
    let subtitle = raw
        .albums
        .first()
        .map(|a| a.name.trim().to_owned())
        .unwrap_or_default();
    let cover = media.resolve(&raw.url_image);
    let mut image = cover.clone();
    let mut position = position_or_center(raw.image_position);

    let mut slides = vec![Slide {
        kind: SlideKind::Title,
        text: raw.name.trim().to_owned(),
        aux: non_empty(&subtitle),
        image: image.clone(),
        image_position: position,
        repeat: false,
        time_ms: Some(0),
        instrumental_time_ms: Some(0),
    }];

    let mut lyrics: Vec<_> = raw.lyric.iter().filter(|l| l.show_slide).collect();
    lyrics.sort_by_key(|l| l.order);

    let mut previous_text = String::new();
    for lyric in lyrics {
        if let Some(own) = media.resolve(&lyric.url_image) {
            image = Some(own);
            position = position_or_center(lyric.image_position);
        }
        let text = clean_lyric(&lyric.lyric);
        let time_ms = parse_time_ms(&lyric.time);
        slides.push(Slide {
            kind: SlideKind::Lyric,
            repeat: !text.is_empty() && text == previous_text,
            aux: non_empty(&clean_lyric(&lyric.aux_lyric)),
            image: image.clone(),
            image_position: position,
            time_ms,
            instrumental_time_ms: parse_time_ms(&lyric.instrumental_time)
                .filter(|ms| *ms > 0)
                .or(time_ms),
            text: text.clone(),
        });
        previous_text = text;
    }

    Song {
        id: raw.id_music,
        title: raw.name.trim().to_owned(),
        subtitle,
        cover,
        audio: media.resolve(&raw.url_music),
        instrumental: media.resolve(&raw.url_instrumental_music),
        duration_ms: parse_time_ms(&raw.duration).filter(|ms| *ms > 0),
        slides,
    }
}

fn position_or_center(position: u32) -> u8 {
    match position {
        1..=9 => position as u8,
        _ => 5,
    }
}

/// Quebras de linha do Windows e espaços sobrando fazem o slide desalinhar.
fn clean_lyric(text: &str) -> String {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn media() -> MediaCache {
        MediaCache::new(PathBuf::from("/nao/existe"), reqwest::Client::new())
    }

    fn sample() -> RawMusic {
        serde_json::from_str(
            r#"{
              "id_music": 1, "name": " Nosso Sol é Jesus ", "duration": "00:02:17",
              "url_image": "https://files.louvorja.com.br/images/a.jpg", "image_position": 0,
              "url_music": "/musics/pt/1992 - Brilha Jesus/Nosso Sol É Jesus.mp3",
              "url_instrumental_music": null,
              "albums": [{ "id_album": 1, "name": "Brilha Jesus" }],
              "lyric": [
                { "lyric": "Refrão\r\n  linha 2 ", "order": 2, "time": "00:00:20", "instrumental_time": "00:00:00", "show_slide": 1 },
                { "lyric": "O nosso sol", "order": 1, "time": "00:00:05", "url_image": "/images/b.jpg", "image_position": 2, "show_slide": 1 },
                { "lyric": "escondido", "order": 3, "show_slide": 0 },
                { "lyric": "Refrão\nlinha 2", "aux_lyric": "Chorus", "order": 4, "time": "00:00:40", "instrumental_time": "00:00:38", "show_slide": 1 }
              ]
            }"#,
        )
        .unwrap()
    }

    #[test]
    fn builds_title_then_visible_lyrics_in_order() {
        let song = song(&sample(), &media());
        let texts: Vec<_> = song.slides.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(
            texts,
            [
                "Nosso Sol é Jesus",
                "O nosso sol",
                "Refrão\nlinha 2",
                "Refrão\nlinha 2"
            ]
        );
        assert_eq!(song.slides[0].kind, SlideKind::Title);
        assert_eq!(song.slides[0].aux.as_deref(), Some("Brilha Jesus"));
        assert_eq!(song.duration_ms, Some(137_000));
        assert!(song.instrumental.is_none());
    }

    #[test]
    fn carries_background_forward_and_marks_repeats() {
        let song = song(&sample(), &media());
        let images: Vec<_> = song
            .slides
            .iter()
            .map(|s| {
                s.image
                    .as_ref()
                    .unwrap()
                    .url
                    .rsplit('/')
                    .next()
                    .unwrap()
                    .to_owned()
            })
            .collect();
        assert_eq!(images, ["a.jpg", "b.jpg", "b.jpg", "b.jpg"]);
        assert_eq!(song.slides[0].image_position, 5);
        assert_eq!(song.slides[2].image_position, 2);
        assert!(!song.slides[2].repeat);
        assert!(song.slides[3].repeat);
        assert_eq!(song.slides[3].aux.as_deref(), Some("Chorus"));
    }

    #[test]
    fn instrumental_time_falls_back_to_voice_time() {
        let song = song(&sample(), &media());
        assert_eq!(song.slides[2].time_ms, Some(20_000));
        assert_eq!(song.slides[2].instrumental_time_ms, Some(20_000));
        assert_eq!(song.slides[3].instrumental_time_ms, Some(38_000));
    }
}
