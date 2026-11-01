//! Medidas de texto com as fontes de verdade.
//!
//! O libass (que desenha o texto no mpv) não quebra linha do jeito do
//! navegador nem deixa escolher a altura da linha. Então a projeção leve faz
//! o que o CSS fazia: mede cada palavra com a própria fonte, quebra as
//! linhas equilibradas e posiciona linha por linha.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::sync::LockExt;

/// Uma fonte carregada para medir.
pub struct Face {
    data: Vec<u8>,
    /// Nome que o libass usa para achar a fonte (`\fn`).
    pub family: String,
    units_per_em: f32,
    /// usWinAscent + usWinDescent: o libass dimensiona `\fs` por essa altura.
    win_height: f32,
}

impl Face {
    pub fn parse(data: Vec<u8>, family: Option<String>) -> Option<Face> {
        let face = ttf_parser::Face::parse(&data, 0).ok()?;
        let units_per_em = f32::from(face.units_per_em());
        let win_height = face
            .tables()
            .os2
            // O ttf-parser devolve a descida do Windows como número negativo.
            .map(|os2| f32::from(os2.windows_ascender()) + f32::from(os2.windows_descender()).abs())
            .filter(|h| *h > 0.0)
            .unwrap_or_else(|| f32::from(face.ascender()) - f32::from(face.descender()));
        let family = family.or_else(|| family_name(&face))?;
        Some(Face {
            data,
            family,
            units_per_em,
            win_height,
        })
    }

    /// O `\fs` do libass que dá uma letra de `px` pixels (o font-size do CSS).
    pub fn ass_size(&self, px: f32) -> f32 {
        px * self.win_height / self.units_per_em
    }

    /// Largura do texto em pixels, com `px` de tamanho (em do CSS) e
    /// `spacing` em em entre as letras (letter-spacing).
    pub fn width(&self, text: &str, px: f32, spacing: f32) -> f32 {
        let Ok(face) = ttf_parser::Face::parse(&self.data, 0) else {
            return text.chars().count() as f32 * px * 0.5;
        };
        let fallback = f32::from(face.units_per_em()) * 0.5;
        let mut units = 0.0;
        let mut count = 0;
        for c in text.chars() {
            count += 1;
            units += face
                .glyph_index(c)
                .and_then(|g| face.glyph_hor_advance(g))
                .map_or(fallback, f32::from);
        }
        units / self.units_per_em * px + spacing * px * count as f32
    }
}

fn family_name(face: &ttf_parser::Face) -> Option<String> {
    let names = face.names();
    let pick = |id: u16| {
        names
            .into_iter()
            .filter(|n| n.name_id == id && n.is_unicode())
            .find_map(|n| n.to_string())
    };
    pick(ttf_parser::name_id::TYPOGRAPHIC_FAMILY).or_else(|| pick(ttf_parser::name_id::FAMILY))
}

/// Fontes já carregadas, por (id, peso); `None` = não existe.
type FaceCache = HashMap<(String, u16), Option<Arc<Face>>>;

/// As fontes da projeção: as embutidas (TTF gerados de `scripts/fonts-ttf.py`),
/// as do sistema e as que a pessoa adicionou.
pub struct Fonts {
    /// Pasta que o mpv recebe em `--osd-fonts-dir`.
    dir: PathBuf,
    cache: Mutex<FaceCache>,
    system: Mutex<Option<fontdb::Database>>,
}

/// Uma fonte escolhida: o nome para o libass e as medidas.
#[derive(Clone)]
pub struct Font {
    pub family: String,
    pub weight: u16,
    face: Option<Arc<Face>>,
}

impl Font {
    pub fn ass_size(&self, px: f32) -> f32 {
        self.face.as_ref().map_or(px * 1.2, |f| f.ass_size(px))
    }

    pub fn width(&self, text: &str, px: f32, spacing: f32) -> f32 {
        match &self.face {
            Some(f) => f.width(text, px, spacing),
            None => text.chars().count() as f32 * px * (0.5 + spacing),
        }
    }
}

const WEIGHTS: [u16; 3] = [400, 600, 700];

impl Fonts {
    pub fn new(dir: PathBuf) -> Self {
        Fonts {
            dir,
            cache: Mutex::default(),
            system: Mutex::default(),
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// A fonte guardada nos ajustes ("condensed", "system:Arial",
    /// "file:minha.ttf"), no peso pedido.
    pub fn get(&self, id: &str, weight: u16) -> Font {
        let id = if id.is_empty() { "condensed" } else { id };
        let key = (id.to_owned(), weight);
        if let Some(face) = self.cache.locked().get(&key) {
            return self.font_from(id, weight, face.clone());
        }
        let face = self.load(id, weight).map(Arc::new);
        self.cache.locked().insert(key, face.clone());
        self.font_from(id, weight, face)
    }

    fn font_from(&self, id: &str, weight: u16, face: Option<Arc<Face>>) -> Font {
        let family = match &face {
            Some(f) => f.family.clone(),
            None => id.strip_prefix("system:").unwrap_or(id).to_owned(),
        };
        Font {
            family,
            weight,
            face,
        }
    }

    fn load(&self, id: &str, weight: u16) -> Option<Face> {
        if let Some(name) = id.strip_prefix("system:") {
            return self.load_system(name, weight);
        }
        if let Some(file) = id.strip_prefix("file:") {
            let data = std::fs::read(self.dir.join(file)).ok()?;
            return Face::parse(data, None);
        }
        // Embutida: o peso mais próximo que existir.
        let mut weights = WEIGHTS.to_vec();
        weights.sort_by_key(|w| w.abs_diff(weight));
        weights
            .into_iter()
            .find_map(|w| {
                let data = std::fs::read(self.dir.join(format!("{id}-{w}.ttf"))).ok()?;
                Face::parse(data, Some(format!("7H {id}")))
            })
            // Fonte que não existe mais (removida numa versão nova): a padrão.
            .or_else(|| {
                (id != "condensed")
                    .then(|| self.load("condensed", weight))
                    .flatten()
            })
    }

    fn load_system(&self, name: &str, weight: u16) -> Option<Face> {
        let mut system = self.system.locked();
        let db = system.get_or_insert_with(|| {
            let mut db = fontdb::Database::new();
            db.load_system_fonts();
            db
        });
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(name)],
            weight: fontdb::Weight(weight),
            ..Default::default()
        };
        let id = db.query(&query)?;
        let mut parsed = None;
        db.with_face_data(id, |data, index| {
            if index == 0 {
                parsed = Face::parse(data.to_vec(), Some(name.to_owned()));
            }
        });
        parsed
    }
}

/// Quebra o texto em linhas de no máximo `max` pixels, equilibradas como o
/// `text-wrap: balance` do CSS (linhas de tamanho parecido). Quebras de linha
/// do texto são respeitadas.
pub fn wrap(text: &str, max: f32, measure: &dyn Fn(&str) -> f32) -> Vec<String> {
    let mut out = Vec::new();
    for paragraph in text.split('\n') {
        let words: Vec<&str> = paragraph.split_whitespace().collect();
        if words.is_empty() {
            continue;
        }
        let greedy = greedy(&words, max, measure);
        if greedy.len() <= 1 {
            out.extend(greedy);
            continue;
        }
        // A menor largura que ainda dá o mesmo número de linhas.
        let (mut lo, mut hi) = (0.0f32, max);
        for _ in 0..12 {
            let mid = (lo + hi) / 2.0;
            if greedy_count(&words, mid, measure) <= greedy.len() {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        out.extend(greedy_lines(&words, hi, measure));
    }
    out
}

fn greedy(words: &[&str], max: f32, measure: &dyn Fn(&str) -> f32) -> Vec<String> {
    greedy_lines(words, max, measure)
}

fn greedy_count(words: &[&str], max: f32, measure: &dyn Fn(&str) -> f32) -> usize {
    greedy_lines(words, max, measure).len()
}

fn greedy_lines(words: &[&str], max: f32, measure: &dyn Fn(&str) -> f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in words {
        if line.is_empty() {
            line.push_str(word);
            continue;
        }
        let candidate = format!("{line} {word}");
        if measure(&candidate) <= max {
            line = candidate;
        } else {
            lines.push(std::mem::take(&mut line));
            line.push_str(word);
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn by_chars(s: &str) -> f32 {
        s.chars().count() as f32
    }

    #[test]
    fn keeps_short_lines_and_explicit_breaks() {
        assert_eq!(wrap("Santo\nSanto", 100.0, &by_chars), ["Santo", "Santo"]);
        assert_eq!(wrap("  \n", 100.0, &by_chars), Vec::<String>::new());
    }

    #[test]
    fn balances_like_css() {
        // Gananciosa daria "aaaa bbbb cccc" + "dd"; equilibrada divide melhor.
        let lines = wrap("aaaa bbbb cccc dd", 14.0, &by_chars);
        assert_eq!(lines.len(), 2);
        let (a, b) = (by_chars(&lines[0]), by_chars(&lines[1]));
        assert!((a - b).abs() <= 5.0, "{lines:?}");
    }

    #[test]
    fn a_word_longer_than_the_line_still_shows() {
        assert_eq!(wrap("abcdefghij", 4.0, &by_chars), ["abcdefghij"]);
    }

    #[test]
    fn measures_builtin_fonts() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fonts");
        let fonts = Fonts::new(dir);
        let font = fonts.get("condensed", 600);
        assert_eq!(font.family, "7H condensed");
        let narrow = font.width("iiii", 100.0, 0.0);
        let wide = font.width("MMMM", 100.0, 0.0);
        assert!(wide > narrow * 2.0, "{narrow} {wide}");
        // Calibrado com o libass: \fs = px × winHeight / unitsPerEm.
        assert!(
            (font.ass_size(100.0) - 134.9).abs() < 1.0,
            "{}",
            font.ass_size(100.0)
        );
        // Fonte desconhecida cai na padrão.
        assert_eq!(fonts.get("nao-existe", 400).family, "7H condensed");
    }
}
