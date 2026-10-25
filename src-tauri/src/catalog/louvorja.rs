//! Importação do LouvorJA desktop instalado no computador, para igrejas sem
//! internet.
//!
//! O LouvorJA (programa em Delphi) guarda tudo numa pasta:
//!
//! ```text
//! <instalação>/config/pt_database.db   (ou database.db, nas versões antigas)
//! <instalação>/config/musicas/<álbum>/<faixa>.mp3
//! <instalação>/config/imagens/<fundo>
//! <instalação>/config/capas/<capa>
//! ```
//!
//! O banco é lido e convertido para o mesmo formato do acervo online
//! (`json_db`), com os endereços de mídia que o acervo online usaria. As pastas
//! entram como fontes locais do cache de mídia, então tudo toca sem internet e
//! continua funcionando se um dia o acervo online for baixado por cima.
//!
//! O esquema seguido é o que a própria API do LouvorJA gera para o programa
//! (`scripts/export-legacy-sqlite.ts` em louvorja/api-workers). Bancos antigos
//! podem não ter algumas tabelas ou colunas; cada uma é opcional aqui.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use rusqlite::types::Value as Sql;
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use serde_json::{json, Value};

use super::model::Lang;
use crate::error::{Error, Result};
use crate::media::{LocalFolders, FILES_URL};

/// Uma instalação do LouvorJA encontrada no disco.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Install {
    /// Pasta da instalação (a que contém `config`).
    pub root: PathBuf,
    pub database: PathBuf,
}

impl Install {
    /// Aceita a pasta da instalação, a pasta `config` ou o próprio banco.
    pub fn find(path: &Path, lang: Lang) -> Option<Install> {
        if path.is_file() {
            let config = path.parent()?;
            let root = config.parent().unwrap_or(config).to_path_buf();
            return Some(Install {
                root,
                database: path.to_path_buf(),
            });
        }
        for config in [path.join("config"), path.to_path_buf()] {
            let names = match lang {
                Lang::Pt => ["pt_database.db", "database.db", "es_database.db"],
                Lang::Es => ["es_database.db", "database.db", "pt_database.db"],
            };
            if let Some(database) = names.iter().map(|n| config.join(n)).find(|p| p.is_file()) {
                let root = if config == path {
                    path.parent().unwrap_or(path)
                } else {
                    path
                };
                return Some(Install {
                    root: root.to_path_buf(),
                    database,
                });
            }
        }
        None
    }

    fn config(&self) -> &Path {
        self.database.parent().unwrap_or(&self.root)
    }

    pub fn folders(&self) -> LocalFolders {
        let config = self.config();
        LocalFolders {
            music: vec![config.join("musicas")],
            images: vec![config.join("imagens")],
            covers: vec![config.join("capas")],
        }
    }
}

/// Procura o LouvorJA nos lugares onde o instalador oficial o coloca.
pub fn detect(lang: Lang) -> Vec<Install> {
    let mut bases: Vec<PathBuf> = [
        "ProgramFiles(x86)",
        "ProgramFiles",
        "ProgramW6432",
        "LOCALAPPDATA",
    ]
    .iter()
    .filter_map(|var| std::env::var_os(var).map(PathBuf::from))
    .collect();
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        bases.push(PathBuf::from(local).join("Programs"));
    }
    bases.push(PathBuf::from("C:\\"));

    let mut found: Vec<Install> = Vec::new();
    for base in bases {
        for name in ["Louvor JA", "LouvorJA", "Louvor JÁ"] {
            if let Some(install) = Install::find(&base.join(name), lang) {
                if !found.contains(&install) {
                    found.push(install);
                }
            }
        }
    }
    found
}

/// Converte as pastas cadastradas em Ajustes nas fontes do cache de mídia.
/// Uma instalação do LouvorJA vale pelas três pastas; outra pasta qualquer é
/// tratada como pasta de músicas (`<álbum>/<faixa>`).
pub fn local_folders(paths: &[String], lang: Lang) -> LocalFolders {
    let mut folders = LocalFolders::default();
    for path in paths.iter().map(PathBuf::from) {
        match Install::find(&path, lang) {
            Some(install) => {
                let f = install.folders();
                folders.music.extend(f.music);
                folders.images.extend(f.images);
                folders.covers.extend(f.covers);
            }
            None => folders.music.push(path),
        }
    }
    folders
}

/// O catálogo lido do banco: as chaves do `json_db` e a versão do banco.
pub struct LocalCatalog {
    pub lang: Lang,
    pub version: u32,
    pub entries: Vec<(String, Vec<u8>)>,
}

/// Categorias do LouvorJA na ordem em que aparecem no app.
const CATEGORIES: [(&str, &str, &str); 6] = [
    ("JA_ANO", "aym", "CDs Oficiais/Ano"),
    ("HASD", "hymnal", "Hinário Adventista"),
    ("HASD_1996", "hymnal_1996", "Hinário Adventista 1996"),
    ("INF", "children", "Infantis"),
    ("DOX", "doxology", "Doxologia"),
    ("DIV", "misc", "Diversas"),
];

struct Song {
    name: String,
    folder: String,
    image: String,
    audio: String,
    instrumental: String,
    language: String,
}

struct Lyric {
    order: i64,
    text: String,
    aux: String,
    image: String,
    show: bool,
    time: String,
    instrumental_time: String,
}

pub fn read(install: &Install, fallback_lang: Lang) -> Result<LocalCatalog> {
    let db = Db::open(&install.database)?;

    if !db.has("MUSICAS") || !db.has("MUSICAS_LETRA") {
        return Err(Error::other(
            "Esse arquivo não parece ser o banco do LouvorJA (faltam as músicas).",
        ));
    }

    // ── Músicas ─────────────────────────────────────────────────────────────
    let mut songs: BTreeMap<i64, Song> = BTreeMap::new();
    for row in db.rows(
        "MUSICAS",
        &[
            "ID",
            "NOME",
            "ALBUM",
            "IMAGEM",
            "URL",
            "URL_INSTRUMENTAL",
            "IDIOMA",
        ],
    )? {
        let Some(id) = int(&row[0]) else { continue };
        songs.entry(id).or_insert_with(|| Song {
            name: text(&row[1]),
            folder: text(&row[2]),
            image: text(&row[3]),
            audio: text(&row[4]),
            instrumental: text(&row[5]),
            language: text(&row[6]).to_lowercase(),
        });
    }

    let lang = match songs
        .values()
        .find(|s| !s.language.is_empty())
        .map(|s| s.language.as_str())
    {
        Some("es") => Lang::Es,
        Some("pt") => Lang::Pt,
        _ => match install.database.file_name().and_then(|n| n.to_str()) {
            Some(n) if n.starts_with("es_") => Lang::Es,
            Some(n) if n.starts_with("pt_") => Lang::Pt,
            _ => fallback_lang,
        },
    };

    let mut lyrics: HashMap<i64, Vec<Lyric>> = HashMap::new();
    for row in db.rows(
        "MUSICAS_LETRA",
        &[
            "MUSICA",
            "ORDEM",
            "LETRA",
            "LETRA_AUX",
            "IMAGEM",
            "EXIBE_SLIDE",
            "TEMPO",
            "TEMPO_PB",
        ],
    )? {
        let Some(song) = int(&row[0]) else { continue };
        let time = time(&row[6]);
        let instrumental_time = time_or(&row[7], &time);
        lyrics.entry(song).or_default().push(Lyric {
            order: int(&row[1]).unwrap_or(0),
            text: text(&row[2]),
            aux: text(&row[3]),
            image: text(&row[4]),
            show: !matches!(&row[5], Sql::Integer(0)) && text(&row[5]) != "N",
            time,
            instrumental_time,
        });
    }
    for list in lyrics.values_mut() {
        list.sort_by_key(|l| l.order);
    }

    let positions: HashMap<String, i64> = if db.has("IMAGEM_POSICAO") {
        db.rows("IMAGEM_POSICAO", &["IMAGEM", "POSICAO"])?
            .iter()
            .filter_map(|r| Some((text(&r[0]), int(&r[1])?)))
            .collect()
    } else {
        HashMap::new()
    };

    // ── Álbuns ──────────────────────────────────────────────────────────────
    let mut albums: BTreeMap<i64, (String, String)> = BTreeMap::new();
    if db.has("ALBUM") {
        for row in db.rows("ALBUM", &["ID", "NOME", "IMAGEM"])? {
            if let Some(id) = int(&row[0]) {
                albums.entry(id).or_insert((text(&row[1]), text(&row[2])));
            }
        }
    }

    // id do álbum → faixas (id da música, número)
    let mut tracks: BTreeMap<i64, Vec<(i64, i64)>> = BTreeMap::new();
    if db.has("ALBUM_MUSICAS") {
        for row in db.rows("ALBUM_MUSICAS", &["ID_ALBUM", "ID_MUSICA", "FAIXA"])? {
            if let (Some(album), Some(song)) = (int(&row[0]), int(&row[1])) {
                if songs.contains_key(&song) && albums.contains_key(&album) {
                    tracks
                        .entry(album)
                        .or_default()
                        .push((song, int(&row[2]).unwrap_or(0)));
                }
            }
        }
    }
    for list in tracks.values_mut() {
        list.sort_by_key(|&(song, track)| (track, song));
        list.dedup();
    }

    // id do álbum → (tipo, subtítulo, ordem)
    let mut kinds: Vec<(i64, String, String, i64)> = Vec::new();
    if db.has("ALBUM_TIPO") {
        for row in db.rows("ALBUM_TIPO", &["ID_ALBUM", "TIPO", "SUBTITULO", "ORDEM"])? {
            if let Some(album) = int(&row[0]).filter(|a| tracks.contains_key(a)) {
                kinds.push((
                    album,
                    text(&row[1]),
                    text(&row[2]),
                    int(&row[3]).unwrap_or(0),
                ));
            }
        }
    }
    // Álbum sem tipo vai para "Diversas" em vez de sumir.
    if !kinds.is_empty() {
        let typed: HashSet<i64> = kinds.iter().map(|k| k.0).collect();
        for &album in tracks.keys().filter(|a| !typed.contains(a)) {
            kinds.push((album, "DIV".into(), String::new(), 0));
        }
    }
    let kind_names: HashMap<String, String> = if db.has("TIPOS_ALBUM") {
        db.rows("TIPOS_ALBUM", &["ID", "TIPO"])?
            .iter()
            .map(|r| (text(&r[0]), text(&r[1])))
            .collect()
    } else {
        HashMap::new()
    };

    let version = if db.has("VERSAO") {
        db.rows("VERSAO", &["VERSAO_BD"])?
            .first()
            .and_then(|r| int(&r[0]))
            .unwrap_or(0) as u32
    } else {
        0
    };

    // ── Conversão para o formato do acervo online ───────────────────────────
    let code = lang.code();
    let cover_url = |name: &str| media_url("covers", name);
    let image_url = |name: &str| media_url("images", name);
    let audio_url = |song: &Song, file: &str| {
        if file.is_empty() {
            String::new()
        } else {
            media_url(&format!("musics/{code}/{}", song.folder), file)
        }
    };

    let album_ref = |id: i64, subtitle: &str| {
        let (name, image) = &albums[&id];
        json!({ "id_album": id, "name": name, "subtitle": subtitle, "url_image": cover_url(image) })
    };

    let mut categories = Vec::new();
    if kinds.is_empty() {
        let all: Vec<Value> = tracks.keys().map(|&id| album_ref(id, "")).collect();
        categories
            .push(json!({ "id_category": 1, "name": "Álbuns", "slug": "albums", "albums": all }));
    } else {
        let known: HashSet<&str> = CATEGORIES.iter().map(|c| c.0).collect();
        for (index, (kind, slug, default_name)) in CATEGORIES.iter().enumerate() {
            let mut members: Vec<&(i64, String, String, i64)> = kinds
                .iter()
                .filter(|k| k.1 == *kind || (*kind == "DIV" && !known.contains(k.1.as_str())))
                .collect();
            if *kind == "JA_ANO" {
                // CDs do ano: o mais recente primeiro.
                members.sort_by(|a, b| b.2.cmp(&a.2).then(b.0.cmp(&a.0)));
            } else {
                members.sort_by_key(|k| (k.3, k.0));
            }
            let mut seen = HashSet::new();
            let list: Vec<Value> = members
                .into_iter()
                .filter(|k| seen.insert(k.0))
                .map(|k| album_ref(k.0, &k.2))
                .collect();
            if !list.is_empty() {
                let name = kind_names
                    .get(*kind)
                    .cloned()
                    .unwrap_or_else(|| default_name.to_string());
                categories.push(json!({
                    "id_category": index + 1, "name": name, "slug": slug, "albums": list
                }));
            }
        }
    }

    let album_names_of = |song: i64| -> String {
        let mut names: Vec<&str> = tracks
            .iter()
            .filter(|(_, list)| list.iter().any(|&(s, _)| s == song))
            .map(|(id, _)| albums[id].0.as_str())
            .collect();
        names.dedup();
        names.join(", ")
    };

    let mut entries: Vec<(String, Value)> = Vec::new();
    let musics: Vec<Value> = songs
        .iter()
        .map(|(&id, song)| {
            let lyric: Vec<&str> = lyrics
                .get(&id)
                .map(|l| l.iter().map(|x| x.text.as_str()).collect())
                .unwrap_or_default();
            json!({
                "id_music": id,
                "name": song.name,
                "has_instrumental_music": i32::from(!song.instrumental.is_empty()),
                "lyric": lyric.join("\n"),
                "albums_names": album_names_of(id),
            })
        })
        .collect();

    let hymnal_albums: Vec<i64> = kinds
        .iter()
        .filter(|k| k.1 == "HASD")
        .map(|k| k.0)
        .collect();
    let hymnal: Vec<Value> = hymnal_albums
        .iter()
        .flat_map(|album| tracks[album].iter())
        .map(|&(song, track)| {
            json!({
                "id_music": song,
                "name": songs[&song].name,
                "track": track,
                "has_instrumental_music": i32::from(!songs[&song].instrumental.is_empty()),
            })
        })
        .collect();

    for (&album, list) in &tracks {
        let (name, image) = &albums[&album];
        let musics: Vec<Value> = list
            .iter()
            .map(|&(song, track)| {
                json!({
                    "id_music": song,
                    "name": songs[&song].name,
                    "track": track,
                    "has_instrumental_music": i32::from(!songs[&song].instrumental.is_empty()),
                })
            })
            .collect();
        entries.push((
            format!("album_{album}"),
            json!({ "id_album": album, "name": name, "url_image": cover_url(image), "musics": musics }),
        ));
    }

    for (&id, song) in &songs {
        let slides: Vec<Value> = lyrics
            .get(&id)
            .map(|list| {
                list.iter()
                    .map(|l| {
                        json!({
                            "lyric": l.text,
                            "aux_lyric": l.aux,
                            "url_image": image_url(&l.image),
                            "image_position": positions.get(&l.image).copied().unwrap_or(0),
                            "time": l.time,
                            "instrumental_time": l.instrumental_time,
                            "show_slide": i32::from(l.show),
                            "order": l.order,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let in_albums: Vec<Value> = tracks
            .iter()
            .filter(|(_, list)| list.iter().any(|&(s, _)| s == id))
            .map(|(&album, _)| json!({ "id_album": album, "name": albums[&album].0 }))
            .collect();
        entries.push((
            format!("music_{id}"),
            json!({
                "id_music": id,
                "name": song.name,
                "url_image": image_url(&song.image),
                "image_position": positions.get(&song.image).copied().unwrap_or(0),
                "url_music": audio_url(song, &song.audio),
                "url_instrumental_music": audio_url(song, &song.instrumental),
                "albums": in_albums,
                "lyric": slides,
            }),
        ));
    }

    entries.push((format!("{code}_categories"), Value::Array(categories)));
    entries.push((format!("{code}_musics"), Value::Array(musics)));
    entries.push((format!("{code}_hymnal"), Value::Array(hymnal)));

    let entries = entries
        .into_iter()
        .map(|(key, value)| Ok((key, serde_json::to_vec(&value)?)))
        .collect::<Result<Vec<_>>>()?;
    Ok(LocalCatalog {
        lang,
        version,
        entries,
    })
}

/// O endereço que o acervo online usaria para este arquivo.
fn media_url(dir: &str, name: &str) -> String {
    let name = name.trim().replace('\\', "/");
    if name.is_empty() {
        return String::new();
    }
    format!("{FILES_URL}/{dir}/{name}")
}

/// Leitura tolerante de bancos SQLite de terceiros (somente leitura).
pub(crate) struct Db {
    conn: Connection,
}

impl Db {
    pub(crate) fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(sql_error)?;
        Ok(Self { conn })
    }

    pub(crate) fn has(&self, table: &str) -> bool {
        !self.columns(table).is_empty()
    }

    fn columns(&self, table: &str) -> HashSet<String> {
        let Ok(mut stmt) = self
            .conn
            .prepare(&format!("PRAGMA table_info(\"{table}\")"))
        else {
            return HashSet::new();
        };
        stmt.query_map([], |row| row.get::<_, String>(1))
            .map(|rows| rows.flatten().map(|c| c.to_uppercase()).collect())
            .unwrap_or_default()
    }

    /// Lê as colunas pedidas; as que não existem neste banco vêm nulas.
    pub(crate) fn rows(&self, table: &str, wanted: &[&str]) -> Result<Vec<Vec<Sql>>> {
        let existing = self.columns(table);
        let select: Vec<String> = wanted
            .iter()
            .map(|c| {
                if existing.contains(*c) {
                    format!("\"{c}\"")
                } else {
                    "NULL".into()
                }
            })
            .collect();
        let sql = format!("SELECT {} FROM \"{table}\"", select.join(", "));
        let mut stmt = self.conn.prepare(&sql).map_err(sql_error)?;
        let rows = stmt
            .query_map([], |row| {
                (0..wanted.len()).map(|i| row.get::<_, Sql>(i)).collect()
            })
            .map_err(sql_error)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(sql_error)
    }
}

fn sql_error(e: rusqlite::Error) -> Error {
    Error::other(format!("Não foi possível ler o banco de dados: {e}"))
}

pub(crate) fn text(value: &Sql) -> String {
    match value {
        Sql::Text(s) => s.trim().to_owned(),
        Sql::Integer(n) => n.to_string(),
        Sql::Real(f) => f.to_string(),
        _ => String::new(),
    }
}

pub(crate) fn int(value: &Sql) -> Option<i64> {
    match value {
        Sql::Integer(n) => Some(*n),
        Sql::Real(f) => Some(*f as i64),
        Sql::Text(s) => s.trim().parse().ok(),
        _ => None,
    }
}

/// Os tempos dos slides já foram texto ("00:01:05"), fração de dia (TTime do
/// Delphi) e milissegundos. Tudo vira "HH:MM:SS.mmm".
fn time(value: &Sql) -> String {
    let ms = match value {
        Sql::Text(s) if s.contains(':') => return s.trim().to_owned(),
        Sql::Text(s) => match s.trim().replace(',', ".").parse::<f64>() {
            Ok(f) => number_to_ms(f),
            Err(_) => return String::new(),
        },
        Sql::Integer(n) => *n as f64,
        Sql::Real(f) => number_to_ms(*f),
        _ => return String::new(),
    };
    let ms = ms.max(0.0).round() as u64;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        ms / 3_600_000,
        ms / 60_000 % 60,
        ms / 1000 % 60,
        ms % 1000
    )
}

fn number_to_ms(f: f64) -> f64 {
    if f < 1.0 {
        f * 86_400_000.0
    } else {
        f
    }
}

fn time_or(value: &Sql, fallback: &str) -> String {
    let t = time(value);
    if t.is_empty() || t.trim_start_matches(['0', ':', '.']).is_empty() {
        fallback.to_owned()
    } else {
        t
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::catalog::raw::{RawAlbum, RawCategory, RawMusic, RawSongRow};

    /// Cria uma instalação falsa com o esquema que o LouvorJA usa.
    pub fn fake_install(dir: &Path) -> Install {
        let config = dir.join("Louvor JA/config");
        std::fs::create_dir_all(config.join("musicas/1992 - Brilha Jesus")).unwrap();
        std::fs::create_dir_all(config.join("imagens")).unwrap();
        std::fs::create_dir_all(config.join("capas")).unwrap();
        std::fs::write(
            config.join("musicas/1992 - Brilha Jesus/Brilha Jesus.mp3"),
            b"x",
        )
        .unwrap();
        std::fs::write(config.join("imagens/ceu.bmp"), b"x").unwrap();
        std::fs::write(config.join("capas/1992.bmp"), b"x").unwrap();

        let conn = Connection::open(config.join("pt_database.db")).unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE ALBUM (ID INT, NOME TEXT, IMAGEM TEXT, PERMITE_DESATIVAR TEXT);
            INSERT INTO ALBUM VALUES (1, 'Brilha Jesus', '1992.bmp', 'S'), (2, 'Hinário Adventista', '', 'N'), (3, 'Vazio', '', 'S');
            CREATE TABLE ALBUM_MUSICAS (ID_ALBUM INT, ID_MUSICA INT, FAIXA INT);
            INSERT INTO ALBUM_MUSICAS VALUES (1, 10, 1), (2, 11, 1), (2, 10, 2);
            CREATE TABLE ALBUM_TIPO (ID_ALBUM INT, TIPO TEXT, SUBTITULO TEXT, ORDEM INT);
            INSERT INTO ALBUM_TIPO VALUES (1, 'JA_ANO', '1992', 1), (2, 'HASD', '', 1);
            CREATE TABLE TIPOS_ALBUM (ID TEXT, TIPO TEXT);
            INSERT INTO TIPOS_ALBUM VALUES ('JA_ANO', 'CDs Oficiais/Ano'), ('HASD', 'Hinário Adventista');
            CREATE TABLE IMAGEM_POSICAO (IMAGEM TEXT, POSICAO INT);
            INSERT INTO IMAGEM_POSICAO VALUES ('ceu.bmp', 2);
            CREATE TABLE MUSICAS (ID INT, ALBUM TEXT, NOME TEXT, IMAGEM TEXT, URL TEXT, URL_INSTRUMENTAL TEXT, IDIOMA TEXT);
            INSERT INTO MUSICAS VALUES
              (10, '1992 - Brilha Jesus', 'Brilha Jesus', 'ceu.bmp', 'Brilha Jesus.mp3', '', 'PT'),
              (10, '1992 - Brilha Jesus', 'Brilha Jesus', 'ceu.bmp', 'Brilha Jesus.mp3', '', 'PT'),
              (11, 'Hinario', 'Santo, Santo, Santo!', '', '001.mp3', '001 PB.mp3', 'PT');
            CREATE TABLE MUSICAS_LETRA (ID INT, EXIBE_SLIDE INT, ORDEM INT, IMAGEM TEXT, LETRA TEXT, MUSICA INT, TEMPO TEXT, TEMPO_PB TEXT);
            INSERT INTO MUSICAS_LETRA VALUES
              (1, 1, 2, '', 'Ilumina meu caminho', 10, '00:00:12', '00:00:00'),
              (2, 1, 1, 'ceu.bmp', 'Brilha, Jesus', 10, '00:00:05', '00:00:04'),
              (3, 0, 3, '', 'oculto', 10, '00:00:20', '00:00:00'),
              (4, 1, 1, '', 'Santo! Santo! Santo!', 11, '00:00:08', '00:00:00');
            CREATE TABLE VERSAO (ID INT, VERSAO_BD INT);
            INSERT INTO VERSAO VALUES (1, 185);
            "#,
        )
        .unwrap();
        Install::find(&dir.join("Louvor JA"), Lang::Pt).unwrap()
    }

    fn entry<T: serde::de::DeserializeOwned>(catalog: &LocalCatalog, key: &str) -> T {
        let (_, bytes) = catalog.entries.iter().find(|(k, _)| k == key).unwrap();
        serde_json::from_slice(bytes).unwrap()
    }

    #[test]
    fn finds_install_from_root_config_or_database() {
        let dir = tempfile::tempdir().unwrap();
        let install = fake_install(dir.path());
        let root = dir.path().join("Louvor JA");
        assert_eq!(install.root, root);
        assert_eq!(
            Install::find(&root.join("config"), Lang::Pt),
            Some(install.clone())
        );
        assert_eq!(
            Install::find(&install.database, Lang::Pt),
            Some(install.clone())
        );
        assert_eq!(Install::find(dir.path(), Lang::Pt), None);
        assert_eq!(install.folders().music, [root.join("config/musicas")]);
    }

    #[test]
    fn converts_the_database_to_the_online_format() {
        let dir = tempfile::tempdir().unwrap();
        let catalog = read(&fake_install(dir.path()), Lang::Es).unwrap();
        assert_eq!((catalog.lang, catalog.version), (Lang::Pt, 185));

        let categories: Vec<RawCategory> = entry(&catalog, "pt_categories");
        let names: Vec<_> = categories.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["CDs Oficiais/Ano", "Hinário Adventista"]);
        assert_eq!(categories[0].albums[0].subtitle, "1992");
        assert_eq!(
            categories[0].albums[0].url_image,
            "https://files.louvorja.com.br/covers/1992.bmp"
        );

        let musics: Vec<RawSongRow> = entry(&catalog, "pt_musics");
        assert_eq!(musics.len(), 2);
        assert_eq!(musics[0].albums_names, "Brilha Jesus, Hinário Adventista");
        assert!(musics[0].lyric.contains("Brilha, Jesus"));

        let hymnal: Vec<RawSongRow> = entry(&catalog, "pt_hymnal");
        assert_eq!(
            hymnal
                .iter()
                .map(|h| (h.id_music, h.track))
                .collect::<Vec<_>>(),
            [(11, 1), (10, 2)]
        );

        let album: RawAlbum = entry(&catalog, "album_1");
        assert_eq!(album.musics.len(), 1);

        let music: RawMusic = entry(&catalog, "music_10");
        assert_eq!(
            music.url_music,
            "https://files.louvorja.com.br/musics/pt/1992 - Brilha Jesus/Brilha Jesus.mp3"
        );
        assert_eq!(music.url_instrumental_music, "");
        assert_eq!(music.image_position, 2);
        let texts: Vec<_> = music
            .lyric
            .iter()
            .map(|l| (l.lyric.as_str(), l.show_slide))
            .collect();
        assert_eq!(
            texts,
            [
                ("Brilha, Jesus", true),
                ("Ilumina meu caminho", true),
                ("oculto", false)
            ]
        );
        assert_eq!(music.lyric[0].instrumental_time, "00:00:04");
        assert_eq!(music.lyric[1].instrumental_time, "00:00:12");
    }

    #[test]
    fn tolerates_old_databases_with_fewer_tables() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config");
        std::fs::create_dir_all(&config).unwrap();
        let conn = Connection::open(config.join("database.db")).unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE MUSICAS (ID INT, NOME TEXT, URL TEXT);
            INSERT INTO MUSICAS VALUES (1, 'Castelo Forte', 'castelo.mp3');
            CREATE TABLE MUSICAS_LETRA (MUSICA INT, ORDEM INT, LETRA TEXT, TEMPO INT);
            INSERT INTO MUSICAS_LETRA VALUES (1, 1, 'Castelo forte', 65500);
            "#,
        )
        .unwrap();
        let catalog = read(&Install::find(dir.path(), Lang::Pt).unwrap(), Lang::Pt).unwrap();
        let music: RawMusic = entry(&catalog, "music_1");
        assert_eq!(music.lyric[0].time, "00:01:05.500");
        let categories: Vec<RawCategory> = entry(&catalog, "pt_categories");
        assert!(categories.is_empty() || categories[0].albums.is_empty());
    }

    #[test]
    fn rejects_files_that_are_not_louvorja_databases() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("database.db");
        Connection::open(&path)
            .unwrap()
            .execute_batch("CREATE TABLE X (A INT);")
            .unwrap();
        let install = Install {
            root: dir.path().into(),
            database: path,
        };
        assert!(read(&install, Lang::Pt).is_err());
    }

    #[test]
    fn converts_every_time_format() {
        assert_eq!(time(&Sql::Text("00:01:05".into())), "00:01:05");
        assert_eq!(time(&Sql::Integer(65_500)), "00:01:05.500");
        assert_eq!(time(&Sql::Real(65.0 / 86_400.0)), "00:01:05.000");
        assert_eq!(time(&Sql::Null), "");
    }
}
