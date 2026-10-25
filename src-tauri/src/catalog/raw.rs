//! Formato público do catálogo do LouvorJA (`/json_db/*`).
//!
//! O catálogo vem de exportações de um banco antigo, então números às vezes
//! chegam como texto e campos às vezes chegam nulos. Tudo aqui é tolerante:
//! um campo estranho vira um valor padrão em vez de derrubar a leitura.

use serde::{Deserialize, Deserializer};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct RawCategory {
    #[serde(default, deserialize_with = "num")]
    pub id_category: u32,
    #[serde(default, deserialize_with = "text")]
    pub name: String,
    #[serde(default, deserialize_with = "text")]
    pub slug: String,
    #[serde(default)]
    pub albums: Vec<RawAlbumRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawAlbumRef {
    #[serde(default, deserialize_with = "num")]
    pub id_album: u32,
    #[serde(default, deserialize_with = "text")]
    pub name: String,
    #[serde(default, deserialize_with = "text")]
    pub color: String,
    #[serde(default, deserialize_with = "text")]
    pub url_image: String,
    #[serde(default, deserialize_with = "text")]
    pub subtitle: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawAlbum {
    #[serde(default, deserialize_with = "num")]
    pub id_album: u32,
    #[serde(default, deserialize_with = "text")]
    pub name: String,
    #[serde(default, deserialize_with = "text")]
    pub color: String,
    #[serde(default, deserialize_with = "text")]
    pub url_image: String,
    #[serde(default)]
    pub musics: Vec<RawSongRow>,
}

/// Uma linha das listas (`{lang}_musics`, `{lang}_hymnal`, faixas de álbum).
#[derive(Debug, Clone, Deserialize)]
pub struct RawSongRow {
    #[serde(default, deserialize_with = "num")]
    pub id_music: u32,
    #[serde(default, deserialize_with = "text")]
    pub name: String,
    #[serde(default, deserialize_with = "num")]
    pub track: u32,
    #[serde(default, deserialize_with = "flag")]
    pub has_instrumental_music: bool,
    #[serde(default, deserialize_with = "text")]
    pub duration: String,
    #[serde(default, deserialize_with = "text")]
    pub lyric: String,
    #[serde(default, deserialize_with = "text")]
    pub albums_names: String,
}

/// Uma música completa (`music_{id}`), com a letra slide a slide.
#[derive(Debug, Clone, Deserialize)]
pub struct RawMusic {
    #[serde(default, deserialize_with = "num")]
    pub id_music: u32,
    #[serde(default, deserialize_with = "text")]
    pub name: String,
    #[serde(default, deserialize_with = "text")]
    pub duration: String,
    #[serde(default, deserialize_with = "text")]
    pub url_image: String,
    #[serde(default, deserialize_with = "num")]
    pub image_position: u32,
    #[serde(default, deserialize_with = "text")]
    pub url_music: String,
    #[serde(default, deserialize_with = "text")]
    pub url_instrumental_music: String,
    #[serde(default, deserialize_with = "list")]
    pub lyric: Vec<RawLyric>,
    #[serde(default)]
    pub albums: Vec<RawAlbumRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawLyric {
    #[serde(default, deserialize_with = "text")]
    pub lyric: String,
    #[serde(default, deserialize_with = "text")]
    pub aux_lyric: String,
    #[serde(default, deserialize_with = "text")]
    pub url_image: String,
    #[serde(default, deserialize_with = "num")]
    pub image_position: u32,
    #[serde(default, deserialize_with = "text")]
    pub time: String,
    #[serde(default, deserialize_with = "text")]
    pub instrumental_time: String,
    #[serde(default = "yes", deserialize_with = "flag")]
    pub show_slide: bool,
    #[serde(default, deserialize_with = "num")]
    pub order: u32,
}

/// `/json_db/config`: a pergunta "há catálogo novo?".
#[derive(Debug, Clone, Deserialize)]
pub struct RawConfig {
    #[serde(default, deserialize_with = "num")]
    pub version_number: u32,
}

fn yes() -> bool {
    true
}

fn text<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Ok(match Value::deserialize(d)? {
        Value::String(s) => s,
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    })
}

fn num<'de, D: Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
    Ok(match Value::deserialize(d)? {
        Value::Number(n) => n
            .as_u64()
            .or_else(|| n.as_f64().map(|f| f.max(0.0) as u64))
            .unwrap_or(0) as u32,
        Value::String(s) => s.trim().parse().unwrap_or(0),
        Value::Bool(b) => b as u32,
        _ => 0,
    })
}

fn flag<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    Ok(match Value::deserialize(d)? {
        Value::Bool(b) => b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => matches!(s.trim(), "1" | "true" | "S" | "s"),
        _ => false,
    })
}

/// A letra às vezes chega como lista e às vezes como objeto `{ "0": {...} }`.
fn list<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<RawLyric>, D::Error> {
    let items: Vec<Value> = match Value::deserialize(d)? {
        Value::Array(items) => items,
        Value::Object(map) => map.into_iter().map(|(_, v)| v).collect(),
        _ => Vec::new(),
    };
    Ok(items
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect())
}

/// Converte "HH:MM:SS", "MM:SS", "HH:MM:SS.mmm" ou segundos em milissegundos.
pub fn parse_time_ms(value: &str) -> Option<u32> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let mut total = 0f64;
    for part in value.split(':') {
        let n: f64 = part.trim().replace(',', ".").parse().ok()?;
        if n < 0.0 {
            return None;
        }
        total = total * 60.0 + n;
    }
    Some((total * 1000.0).round() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_times() {
        assert_eq!(parse_time_ms("00:02:17"), Some(137_000));
        assert_eq!(parse_time_ms("01:05"), Some(65_000));
        assert_eq!(parse_time_ms("00:00:01.5"), Some(1_500));
        assert_eq!(parse_time_ms("12"), Some(12_000));
        assert_eq!(parse_time_ms(""), None);
        assert_eq!(parse_time_ms("abc"), None);
    }

    #[test]
    fn tolerates_messy_fields() {
        let json = r#"{
            "id_music": "42", "name": null, "image_position": "5",
            "lyric": { "0": { "lyric": "A", "show_slide": "1", "order": 2 },
                       "1": { "lyric": "B", "show_slide": 0, "order": 1 },
                       "2": "lixo" }
        }"#;
        let music: RawMusic = serde_json::from_str(json).unwrap();
        assert_eq!(music.id_music, 42);
        assert_eq!(music.name, "");
        assert_eq!(music.image_position, 5);
        assert_eq!(music.lyric.len(), 2);
        assert!(music.lyric.iter().any(|l| l.lyric == "B" && !l.show_slide));
    }

    #[test]
    fn show_slide_defaults_to_true() {
        let lyric: RawLyric = serde_json::from_str(r#"{ "lyric": "x" }"#).unwrap();
        assert!(lyric.show_slide);
    }
}
