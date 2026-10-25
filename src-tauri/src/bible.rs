//! A Bíblia, guardada num SQLite próprio (`bible.db`).
//!
//! Duas origens, o mesmo formato no fim:
//! - o banco do LouvorJA desktop (views `VERSAO_BIBLICA`, `LIVRO`, `BIBLIA`),
//!   para quem não tem internet;
//! - o pacote online `/db/bible-bundle` (um JSON por capítulo) com as listas
//!   `{lang}_bible_version` e `{lang}_bible_book` do catálogo.
//!
//! A conexão é aberta a cada consulta: assim o arquivo pode ser trocado (por
//! um pacote de pendrive, por exemplo) sem fechar o app.

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::catalog::louvorja::{int, text, Db};
use crate::catalog::model::Lang;
use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BibleVersion {
    pub id: i64,
    pub abbreviation: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BibleBook {
    /// 1 (Gênesis) a 66 (Apocalipse).
    pub number: i64,
    pub name: String,
    pub abbreviation: String,
    pub chapters: i64,
    /// 1 = Antigo, 2 = Novo Testamento.
    pub testament: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Verse {
    pub number: i64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub book: i64,
    pub chapter: i64,
    pub verse: i64,
    pub text: String,
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS version (
    id INTEGER PRIMARY KEY,
    lang TEXT NOT NULL,
    abbreviation TEXT NOT NULL,
    name TEXT NOT NULL,
    UNIQUE (lang, abbreviation)
);
CREATE TABLE IF NOT EXISTS book (
    lang TEXT NOT NULL,
    number INTEGER NOT NULL,
    name TEXT NOT NULL,
    abbreviation TEXT NOT NULL,
    chapters INTEGER NOT NULL,
    testament INTEGER NOT NULL,
    PRIMARY KEY (lang, number)
);
CREATE TABLE IF NOT EXISTS verse (
    version INTEGER NOT NULL,
    book INTEGER NOT NULL,
    chapter INTEGER NOT NULL,
    verse INTEGER NOT NULL,
    text TEXT NOT NULL,
    PRIMARY KEY (version, book, chapter, verse)
) WITHOUT ROWID;
"#;

pub struct Bible {
    path: PathBuf,
}

impl Bible {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn read(&self) -> Result<Connection> {
        if !self.path.is_file() {
            return Err(Error::other("A Bíblia ainda não está neste computador."));
        }
        Connection::open_with_flags(&self.path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(sql)
    }

    fn write(&self) -> Result<Connection> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let conn = Connection::open(&self.path).map_err(sql)?;
        conn.execute_batch(SCHEMA).map_err(sql)?;
        Ok(conn)
    }

    pub fn versions(&self, lang: Lang) -> Result<Vec<BibleVersion>> {
        let conn = match self.read() {
            Ok(conn) => conn,
            Err(_) => return Ok(Vec::new()),
        };
        let mut stmt = conn
            .prepare(
                "SELECT v.id, v.abbreviation, v.name FROM version v \
                 WHERE v.lang = ?1 AND EXISTS (SELECT 1 FROM verse WHERE verse.version = v.id) \
                 ORDER BY v.name",
            )
            .map_err(sql)?;
        let rows = stmt
            .query_map([lang.code()], |r| {
                Ok(BibleVersion {
                    id: r.get(0)?,
                    abbreviation: r.get(1)?,
                    name: r.get(2)?,
                })
            })
            .map_err(sql)?;
        rows.collect::<std::result::Result<_, _>>().map_err(sql)
    }

    pub fn books(&self, lang: Lang) -> Result<Vec<BibleBook>> {
        let conn = self.read()?;
        let mut stmt = conn
            .prepare(
                "SELECT number, name, abbreviation, chapters, testament FROM book \
                 WHERE lang = ?1 ORDER BY number",
            )
            .map_err(sql)?;
        let rows = stmt
            .query_map([lang.code()], |r| {
                Ok(BibleBook {
                    number: r.get(0)?,
                    name: r.get(1)?,
                    abbreviation: r.get(2)?,
                    chapters: r.get(3)?,
                    testament: r.get(4)?,
                })
            })
            .map_err(sql)?;
        rows.collect::<std::result::Result<_, _>>().map_err(sql)
    }

    pub fn chapter(&self, version: i64, book: i64, chapter: i64) -> Result<Vec<Verse>> {
        let conn = self.read()?;
        let mut stmt = conn
            .prepare(
                "SELECT verse, text FROM verse WHERE version = ?1 AND book = ?2 AND chapter = ?3 \
                 ORDER BY verse",
            )
            .map_err(sql)?;
        let rows = stmt
            .query_map(params![version, book, chapter], |r| {
                Ok(Verse {
                    number: r.get(0)?,
                    text: r.get(1)?,
                })
            })
            .map_err(sql)?;
        rows.collect::<std::result::Result<_, _>>().map_err(sql)
    }

    /// Procura um trecho do texto (sem diferenciar maiúsculas).
    pub fn search(&self, version: i64, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let query = query.trim();
        if query.chars().count() < 3 {
            return Ok(Vec::new());
        }
        let conn = self.read()?;
        let pattern = format!("%{}%", query.replace(['%', '_'], ""));
        let mut stmt = conn
            .prepare(
                "SELECT book, chapter, verse, text FROM verse WHERE version = ?1 AND text LIKE ?2 \
                 ORDER BY book, chapter, verse LIMIT ?3",
            )
            .map_err(sql)?;
        let rows = stmt
            .query_map(params![version, pattern, limit as i64], |r| {
                Ok(SearchHit {
                    book: r.get(0)?,
                    chapter: r.get(1)?,
                    verse: r.get(2)?,
                    text: r.get(3)?,
                })
            })
            .map_err(sql)?;
        rows.collect::<std::result::Result<_, _>>().map_err(sql)
    }

    /// Importa as versões do banco do LouvorJA desktop. Devolve quantos
    /// versículos entraram (zero se o banco não tiver a Bíblia).
    pub fn import_desktop(&self, database: &Path, lang: Lang) -> Result<usize> {
        let source = Db::open(database)?;
        if !source.has("BIBLIA") || !source.has("LIVRO") {
            return Ok(0);
        }
        let verses = source.rows(
            "BIBLIA",
            &["VERSAO", "LIVRO", "CAPITULO", "VERSICULO", "PASSAGEM"],
        )?;
        if verses.is_empty() {
            return Ok(0);
        }
        let names: HashMap<String, String> = if source.has("VERSAO_BIBLICA") {
            source
                .rows("VERSAO_BIBLICA", &["SIGLA", "VERSAO"])?
                .iter()
                .map(|r| (text(&r[0]), text(&r[1])))
                .collect()
        } else {
            HashMap::new()
        };
        let books: Vec<(i64, String, String, i64, i64)> = source
            .rows(
                "LIVRO",
                &["ID", "LIVRO", "SIGLA", "CAPITULOS", "TESTAMENTO"],
            )?
            .iter()
            .filter_map(|r| {
                let testament = if text(&r[4]).eq_ignore_ascii_case("NT") {
                    2
                } else {
                    1
                };
                Some((
                    int(&r[0])?,
                    text(&r[1]),
                    text(&r[2]),
                    int(&r[3]).unwrap_or(0),
                    testament,
                ))
            })
            .collect();

        let mut conn = self.write()?;
        let tx = conn.transaction().map_err(sql)?;
        for (number, name, abbreviation, chapters, testament) in &books {
            tx.execute(
                "INSERT OR REPLACE INTO book (lang, number, name, abbreviation, chapters, testament) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![lang.code(), number, name, abbreviation, chapters, testament],
            )
            .map_err(sql)?;
        }
        let mut ids: HashMap<String, i64> = HashMap::new();
        let mut count = 0;
        {
            let mut insert = tx
                .prepare(
                    "INSERT OR REPLACE INTO verse (version, book, chapter, verse, text) \
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                )
                .map_err(sql)?;
            for row in &verses {
                let abbreviation = text(&row[0]);
                let version = match ids.get(&abbreviation) {
                    Some(&id) => id,
                    None => {
                        let name = names
                            .get(&abbreviation)
                            .cloned()
                            .unwrap_or_else(|| abbreviation.clone());
                        let id = upsert_version(&tx, lang, &abbreviation, &name)?;
                        tx.execute("DELETE FROM verse WHERE version = ?1", [id])
                            .map_err(sql)?;
                        ids.insert(abbreviation, id);
                        id
                    }
                };
                let (Some(book), Some(chapter), Some(verse)) =
                    (int(&row[1]), int(&row[2]), int(&row[3]))
                else {
                    continue;
                };
                insert
                    .execute(params![version, book, chapter, verse, text(&row[4])])
                    .map_err(sql)?;
                count += 1;
            }
        }
        tx.commit().map_err(sql)?;
        Ok(count)
    }

    /// Importa o pacote online. `versions` e `books` são os JSON de
    /// `{lang}_bible_version` e `{lang}_bible_book` de cada idioma.
    pub fn import_bundle(
        &self,
        zip_path: &Path,
        catalogs: &[(Lang, Value, Value)],
        mut progress: impl FnMut(usize, usize),
    ) -> Result<usize> {
        #[derive(Deserialize)]
        struct RawVersion {
            id_bible_version: i64,
            name: String,
            abbreviation: String,
        }
        #[derive(Deserialize)]
        struct RawBook {
            id_bible_book: i64,
            book_number: i64,
            name: String,
            #[serde(default)]
            abbreviation: String,
            chapters: i64,
            testament: i64,
        }

        let mut conn = self.write()?;
        let tx = conn.transaction().map_err(sql)?;
        // id do LouvorJA → (nosso id, idioma); id do livro → número 1..66.
        let mut versions: HashMap<i64, i64> = HashMap::new();
        let mut book_numbers: HashMap<i64, i64> = HashMap::new();
        for (lang, raw_versions, raw_books) in catalogs {
            let list: Vec<RawVersion> = serde_json::from_value(raw_versions.clone())?;
            for v in list {
                let id = upsert_version(&tx, *lang, &v.abbreviation, &v.name)?;
                tx.execute("DELETE FROM verse WHERE version = ?1", [id])
                    .map_err(sql)?;
                versions.insert(v.id_bible_version, id);
            }
            let books: Vec<RawBook> = serde_json::from_value(raw_books.clone())?;
            for b in books {
                book_numbers.insert(b.id_bible_book, b.book_number);
                tx.execute(
                    "INSERT OR REPLACE INTO book (lang, number, name, abbreviation, chapters, testament) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![lang.code(), b.book_number, b.name, b.abbreviation, b.chapters, b.testament],
                )
                .map_err(sql)?;
            }
        }

        let mut archive = zip::ZipArchive::new(std::fs::File::open(zip_path)?)?;
        let total = archive.len();
        let mut count = 0;
        {
            let mut insert = tx
                .prepare(
                    "INSERT OR REPLACE INTO verse (version, book, chapter, verse, text) \
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                )
                .map_err(sql)?;
            for i in 0..total {
                let mut entry = archive.by_index(i)?;
                let name = Path::new(entry.name())
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_owned();
                // bible_{versão}_{livro}_{capítulo}.json
                let Some(parts) = name
                    .strip_prefix("bible_")
                    .and_then(|n| n.strip_suffix(".json"))
                else {
                    continue;
                };
                let numbers: Vec<i64> = parts.split('_').filter_map(|p| p.parse().ok()).collect();
                let [raw_version, raw_book, chapter] = numbers[..] else {
                    continue;
                };
                let (Some(&version), Some(&book)) =
                    (versions.get(&raw_version), book_numbers.get(&raw_book))
                else {
                    continue;
                };
                let mut body = String::new();
                entry.read_to_string(&mut body)?;
                let verses: HashMap<String, Value> =
                    serde_json::from_str(&body).unwrap_or_default();
                for (number, text) in verses {
                    let (Ok(number), Some(text)) = (number.parse::<i64>(), text.as_str()) else {
                        continue;
                    };
                    insert
                        .execute(params![version, book, chapter, number, text])
                        .map_err(sql)?;
                    count += 1;
                }
                if i % 200 == 0 {
                    progress(i, total);
                }
            }
        }
        tx.commit().map_err(sql)?;
        progress(total, total);
        Ok(count)
    }
}

fn upsert_version(conn: &Connection, lang: Lang, abbreviation: &str, name: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO version (lang, abbreviation, name) VALUES (?1, ?2, ?3) \
         ON CONFLICT (lang, abbreviation) DO UPDATE SET name = excluded.name",
        params![lang.code(), abbreviation, name],
    )
    .map_err(sql)?;
    conn.query_row(
        "SELECT id FROM version WHERE lang = ?1 AND abbreviation = ?2",
        params![lang.code(), abbreviation],
        |r| r.get(0),
    )
    .map_err(sql)
}

fn sql(e: rusqlite::Error) -> Error {
    Error::other(format!("Erro no banco da Bíblia: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn desktop_db(dir: &Path) -> PathBuf {
        let path = dir.join("pt_database.db");
        if path.exists() {
            return path;
        }
        Connection::open(&path)
            .unwrap()
            .execute_batch(
                r#"
                CREATE TABLE VERSAO_BIBLICA (SIGLA TEXT, VERSAO TEXT);
                INSERT INTO VERSAO_BIBLICA VALUES ('ARA', 'Almeida Revista e Atualizada'), ('NTLH', 'Nova Tradução na Linguagem de Hoje');
                CREATE TABLE LIVRO (ID INT, SIGLA TEXT, TESTAMENTO TEXT, LIVRO TEXT, CAPITULOS INT);
                INSERT INTO LIVRO VALUES (1, 'Gn', 'AT', 'Gênesis', 50), (43, 'Jo', 'NT', 'João', 21);
                CREATE TABLE BIBLIA (ID INT, VERSAO TEXT, LIVRO INT, CAPITULO INT, VERSICULO INT, PASSAGEM TEXT);
                INSERT INTO BIBLIA VALUES
                  (1, 'ARA', 1, 1, 1, 'No princípio, criou Deus os céus e a terra.'),
                  (2, 'ARA', 43, 3, 16, 'Porque Deus amou o mundo de tal maneira...'),
                  (3, 'ARA', 43, 3, 17, 'Porquanto Deus enviou o seu Filho ao mundo...'),
                  (4, 'NTLH', 43, 3, 16, 'Porque Deus amou o mundo tanto...');
                "#,
            )
            .unwrap();
        path
    }

    #[test]
    fn imports_from_the_desktop_database_and_reads_it_back() {
        let dir = tempfile::tempdir().unwrap();
        let bible = Bible::new(dir.path().join("bible.db"));
        assert!(bible.versions(Lang::Pt).unwrap().is_empty());

        assert_eq!(
            bible
                .import_desktop(&desktop_db(dir.path()), Lang::Pt)
                .unwrap(),
            4
        );
        let versions = bible.versions(Lang::Pt).unwrap();
        assert_eq!(
            versions
                .iter()
                .map(|v| v.abbreviation.as_str())
                .collect::<Vec<_>>(),
            ["ARA", "NTLH"]
        );
        let books = bible.books(Lang::Pt).unwrap();
        assert_eq!((books[1].name.as_str(), books[1].testament), ("João", 2));

        let ara = versions[0].id;
        let john3 = bible.chapter(ara, 43, 3).unwrap();
        assert_eq!(john3.iter().map(|v| v.number).collect::<Vec<_>>(), [16, 17]);
        assert_eq!(bible.search(ara, "princípio", 10).unwrap()[0].book, 1);
        assert!(
            bible.search(ara, "no", 10).unwrap().is_empty(),
            "busca curta demais"
        );

        // Importar de novo não duplica.
        bible
            .import_desktop(&desktop_db(dir.path()), Lang::Pt)
            .unwrap();
        assert_eq!(bible.chapter(ara, 43, 3).unwrap().len(), 2);
    }

    #[test]
    fn imports_the_online_bundle() {
        let dir = tempfile::tempdir().unwrap();
        let zip_path = dir.path().join("bible.zip");
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&zip_path).unwrap());
        let options = zip::write::SimpleFileOptions::default();
        for (name, body) in [
            ("_bundle_manifest.json", "{}"),
            (
                "bible_13_43_3.json",
                r#"{"16":"Porque Deus amou o mundo","17":"Porquanto Deus enviou"}"#,
            ),
            ("bible_13_1_1.json", r#"{"1":"No princípio"}"#),
            ("bible_99_1_1.json", r#"{"1":"versão desconhecida"}"#),
        ] {
            zip.start_file(name, options).unwrap();
            zip.write_all(body.as_bytes()).unwrap();
        }
        zip.finish().unwrap();

        let versions = serde_json::json!([{ "id_bible_version": 13, "name": "Almeida Corrigida e Fiel", "abbreviation": "ACF" }]);
        let books = serde_json::json!([
            { "id_bible_book": 1, "book_number": 1, "name": "Gênesis", "abbreviation": "Gn", "chapters": 50, "testament": 1 },
            { "id_bible_book": 43, "book_number": 43, "name": "João", "abbreviation": "Jo", "chapters": 21, "testament": 2 }
        ]);
        let bible = Bible::new(dir.path().join("bible.db"));
        let count = bible
            .import_bundle(&zip_path, &[(Lang::Pt, versions, books)], |_, _| {})
            .unwrap();
        assert_eq!(count, 3);
        let acf = bible.versions(Lang::Pt).unwrap()[0].id;
        assert_eq!(
            bible.chapter(acf, 43, 3).unwrap()[0].text,
            "Porque Deus amou o mundo"
        );
    }

    #[test]
    fn databases_without_the_bible_import_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("database.db");
        Connection::open(&path)
            .unwrap()
            .execute_batch("CREATE TABLE MUSICAS (ID INT);")
            .unwrap();
        let bible = Bible::new(dir.path().join("bible.db"));
        assert_eq!(bible.import_desktop(&path, Lang::Pt).unwrap(), 0);
        assert!(!dir.path().join("bible.db").exists());
    }
}
