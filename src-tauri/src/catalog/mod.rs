//! O catálogo de hinos: as listas que ficam na memória (coleções, hinário e o
//! índice de busca). Álbuns e músicas completos são lidos do disco sob demanda.

pub mod louvorja;
pub mod model;
pub mod raw;
pub mod remote;
pub mod search;
pub mod store;

use std::collections::HashMap;

use serde::de::DeserializeOwned;

use crate::error::{Error, Result};
use crate::media::MediaCache;
use model::{Collection, Lang, SongSummary};
use raw::{RawCategory, RawSongRow};
use search::IndexedSong;
use store::Store;

pub struct Catalog {
    pub version: Option<u32>,
    categories: Vec<RawCategory>,
    hymnal: Vec<SongSummary>,
    index: Vec<IndexedSong>,
    by_id: HashMap<u32, usize>,
}

impl Catalog {
    /// Carrega as listas do idioma. `CatalogMissing` se ainda não houve download.
    pub fn load(store: &Store, lang: Lang) -> Result<Self> {
        let code = lang.code();
        let categories: Vec<RawCategory> = read(store, &format!("{code}_categories"))?;
        let musics: Vec<RawSongRow> = read(store, &format!("{code}_musics"))?;
        let hymnal_rows: Vec<RawSongRow> =
            read(store, &format!("{code}_hymnal")).unwrap_or_default();

        let hymn_numbers: HashMap<u32, u32> = hymnal_rows
            .iter()
            .filter(|row| row.track > 0)
            .map(|row| (row.id_music, row.track))
            .collect();

        let mut index: Vec<IndexedSong> = musics
            .iter()
            .map(|row| {
                let number = hymn_numbers.get(&row.id_music).copied();
                IndexedSong::new(model::song_summary(row, number), &row.lyric)
            })
            .collect();
        index.sort_by(|a, b| a.summary.title.cmp(&b.summary.title));
        let by_id = index
            .iter()
            .enumerate()
            .map(|(i, song)| (song.summary.id, i))
            .collect();

        let hymnal = hymnal_rows
            .iter()
            .map(|row| model::song_summary(row, Some(row.track)))
            .collect();

        Ok(Self {
            version: store.version(),
            categories,
            hymnal,
            index,
            by_id,
        })
    }

    pub fn collections(&self, media: &MediaCache) -> Vec<Collection> {
        self.categories
            .iter()
            .filter(|c| !c.albums.is_empty())
            .map(|c| Collection {
                id: c.id_category,
                name: c.name.trim().to_owned(),
                slug: c.slug.clone(),
                albums: c
                    .albums
                    .iter()
                    .map(|a| model::album_summary(a, media))
                    .collect(),
            })
            .collect()
    }

    pub fn hymnal(&self) -> &[SongSummary] {
        &self.hymnal
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<SongSummary> {
        search::search(&self.index, query, limit)
    }

    /// Resumos das músicas pedidas, na mesma ordem (favoritos, recentes).
    pub fn summaries(&self, ids: &[u32]) -> Vec<SongSummary> {
        ids.iter()
            .filter_map(|id| self.by_id.get(id))
            .map(|&i| self.index[i].summary.clone())
            .collect()
    }

    /// Procura uma música pelo título (sem acento/caixa), preferindo a que
    /// aparece no álbum indicado. Usado para migrar favoritos antigos.
    pub fn find_title(&self, title: &str, album: &str) -> Option<u32> {
        let title = search::normalize(title);
        let album = search::normalize(album);
        let mut matches = self
            .index
            .iter()
            .filter(|song| search::normalize(&song.summary.title) == title);
        let first = matches.next()?;
        if album.is_empty() {
            return Some(first.summary.id);
        }
        std::iter::once(first)
            .chain(matches)
            .find(|song| search::normalize(&song.summary.subtitle).contains(&album))
            .or(Some(first))
            .map(|song| song.summary.id)
    }

    pub fn song_count(&self) -> usize {
        self.index.len()
    }

    pub fn album_count(&self) -> usize {
        let mut ids: Vec<u32> = self
            .categories
            .iter()
            .flat_map(|c| c.albums.iter().map(|a| a.id_album))
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids.len()
    }
}

fn read<T: DeserializeOwned>(store: &Store, key: &str) -> Result<T> {
    let bytes = store.read(key).ok_or(Error::CatalogMissing)?;
    Ok(serde_json::from_slice(&bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_with(files: &[(&str, &str)]) -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().join("catalog"));
        std::fs::create_dir_all(store.dir()).unwrap();
        for (key, json) in files {
            store.write(key, json.as_bytes()).unwrap();
        }
        (dir, store)
    }

    #[test]
    fn missing_catalog_is_reported_as_such() {
        let (_dir, store) = store_with(&[]);
        assert!(matches!(
            Catalog::load(&store, Lang::Pt),
            Err(Error::CatalogMissing)
        ));
    }

    #[test]
    fn loads_lists_and_links_hymn_numbers() {
        let (_dir, store) = store_with(&[
            (
                "pt_categories",
                r#"[{"id_category":6,"name":"CDs","slug":"aym","albums":[
                    {"id_album":727,"name":"Meu Lugar","url_image":"/covers/2026.jpg","subtitle":"2026"}]},
                   {"id_category":7,"name":"Vazia","slug":"x","albums":[]}]"#,
            ),
            (
                "pt_musics",
                r#"[{"id_music":2,"name":"Castelo Forte","lyric":"castelo"},
                    {"id_music":1,"name":"Santo, Santo, Santo!","lyric":"santo"}]"#,
            ),
            (
                "pt_hymnal",
                r#"[{"id_music":1,"name":"Santo, Santo, Santo!","track":1}]"#,
            ),
        ]);
        let catalog = Catalog::load(&store, Lang::Pt).unwrap();
        let media = MediaCache::new(store.dir().join("media"), reqwest::Client::new());

        let collections = catalog.collections(&media);
        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0].albums[0].subtitle.as_deref(), Some("2026"));
        assert_eq!(catalog.album_count(), 1);
        assert_eq!(catalog.song_count(), 2);
        assert_eq!(catalog.hymnal()[0].number, Some(1));
        assert_eq!(catalog.search("1", 5)[0].id, 1);
        let favorites: Vec<_> = catalog
            .summaries(&[2, 99, 1])
            .iter()
            .map(|s| s.id)
            .collect();
        assert_eq!(favorites, [2, 1]);
    }
}
