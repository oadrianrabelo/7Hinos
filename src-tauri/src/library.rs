//! A biblioteca de hinos: junta o catálogo em disco, a API do LouvorJA e o
//! cache de mídia atrás de uma interface pequena. É o único lugar que sabe de
//! onde cada coisa vem; os comandos só repassam.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use futures_util::{stream, StreamExt};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::catalog::model::{self, Album, Lang, Song};
use crate::catalog::raw::{RawAlbum, RawMusic};
use crate::catalog::remote::{Remote, API_URL};
use crate::catalog::store::Store;
use crate::catalog::Catalog;
use crate::error::{Error, Result};
use crate::media::MediaCache;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogStatus {
    pub installed: bool,
    pub version: Option<u32>,
    pub songs: usize,
    pub albums: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheck {
    pub installed: Option<u32>,
    pub available: u32,
}

impl UpdateCheck {
    pub fn has_update(&self) -> bool {
        self.installed.is_none_or(|v| self.available > v)
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineStatus {
    pub cached: usize,
    pub total: usize,
}

pub struct Library {
    store: Store,
    remote: Remote,
    media: MediaCache,
    catalog: RwLock<Option<Arc<Catalog>>>,
    sync_lock: tokio::sync::Mutex<()>,
}

impl Library {
    /// `SETEHINOS_API_URL` troca a API (útil para testar com um servidor local).
    pub fn new(data_dir: PathBuf) -> Self {
        let api = std::env::var("SETEHINOS_API_URL").unwrap_or_else(|_| API_URL.to_owned());
        Self::with_api(data_dir, &api)
    }

    pub fn with_api(data_dir: PathBuf, api: &str) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(concat!("7Hinos/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("cliente HTTP");
        Self {
            store: Store::new(data_dir.join("catalog")),
            remote: Remote::new(client.clone(), api),
            media: MediaCache::new(data_dir.join("media"), client),
            catalog: RwLock::new(None),
            sync_lock: tokio::sync::Mutex::new(()),
        }
    }

    pub fn media(&self) -> &MediaCache {
        &self.media
    }

    /// (Re)carrega as listas do idioma. Sem catálogo baixado, fica vazio.
    pub fn load(&self, lang: Lang) -> Result<()> {
        let loaded = match Catalog::load(&self.store, lang) {
            Ok(catalog) => Some(Arc::new(catalog)),
            Err(Error::CatalogMissing) => None,
            Err(e) => return Err(e),
        };
        *self.catalog.write().expect("catálogo") = loaded;
        Ok(())
    }

    pub fn catalog(&self) -> Result<Arc<Catalog>> {
        self.catalog
            .read()
            .expect("catálogo")
            .clone()
            .ok_or(Error::CatalogMissing)
    }

    pub fn status(&self) -> CatalogStatus {
        match self.catalog() {
            Ok(c) => CatalogStatus {
                installed: true,
                version: c.version,
                songs: c.song_count(),
                albums: c.album_count(),
            },
            Err(_) => CatalogStatus {
                installed: false,
                version: None,
                songs: 0,
                albums: 0,
            },
        }
    }

    pub async fn check_update(&self) -> Result<UpdateCheck> {
        Ok(UpdateCheck {
            installed: self.store.version(),
            available: self.remote.version().await?,
        })
    }

    /// Baixa e instala o catálogo publicado, se for mais novo que o instalado
    /// (ou sempre, com `force`). Devolve `true` quando algo mudou.
    pub async fn sync(
        &self,
        lang: Lang,
        force: bool,
        progress: impl FnMut(u64, Option<u64>),
    ) -> Result<bool> {
        let _guard = self.sync_lock.lock().await;
        let check = self.check_update().await?;
        if !force && !check.has_update() {
            return Ok(false);
        }
        let zip = self.store.dir().with_extension("zip");
        if let Some(parent) = zip.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        self.remote
            .download_bundle(check.available, &zip, progress)
            .await?;

        let store = Store::new(self.store.dir().to_path_buf());
        let zip_path = zip.clone();
        let installed =
            tokio::task::spawn_blocking(move || store.install_bundle(&zip_path, check.available))
                .await
                .map_err(|e| Error::other(e.to_string()))?;
        let _ = tokio::fs::remove_file(&zip).await;
        installed?;

        self.load(lang)?;
        Ok(true)
    }

    pub async fn album(&self, id: u32) -> Result<Album> {
        let raw: RawAlbum = self.key(&format!("album_{id}")).await?;
        Ok(model::album(&raw, &self.media))
    }

    pub async fn song(&self, id: u32) -> Result<Song> {
        let raw: RawMusic = self.key(&format!("music_{id}")).await?;
        Ok(model::song(&raw, &self.media))
    }

    /// Todos os endereços de mídia de que uma música precisa para tocar offline.
    pub async fn song_media(&self, id: u32) -> Result<BTreeSet<String>> {
        let song = self.song(id).await?;
        let mut urls = BTreeSet::new();
        for media in [&song.cover, &song.audio, &song.instrumental]
            .into_iter()
            .flatten()
        {
            urls.insert(media.url.clone());
        }
        for slide in &song.slides {
            if let Some(image) = &slide.image {
                urls.insert(image.url.clone());
            }
        }
        Ok(urls)
    }

    pub async fn album_media(&self, id: u32) -> Result<BTreeSet<String>> {
        let album = self.album(id).await?;
        let mut urls: BTreeSet<String> = album.cover.iter().map(|m| m.url.clone()).collect();
        for song in &album.songs {
            urls.extend(self.song_media(song.id).await?);
        }
        Ok(urls)
    }

    pub async fn album_offline(&self, id: u32) -> Result<OfflineStatus> {
        let urls = self.album_media(id).await?;
        Ok(OfflineStatus {
            cached: urls.iter().filter(|u| self.media.is_cached(u)).count(),
            total: urls.len(),
        })
    }

    /// Baixa a mídia que falta, quatro arquivos por vez. Um arquivo que falha
    /// não interrompe os outros; o erro volta no fim.
    pub async fn download(
        &self,
        urls: BTreeSet<String>,
        mut progress: impl FnMut(OfflineStatus),
    ) -> Result<()> {
        let total = urls.len();
        let mut cached = urls.iter().filter(|u| self.media.is_cached(u)).count();
        progress(OfflineStatus { cached, total });

        let missing: Vec<String> = urls
            .into_iter()
            .filter(|u| !self.media.is_cached(u))
            .collect();
        let mut results = stream::iter(missing)
            .map(|url| async move { self.media.download(&url).await })
            .buffer_unordered(4);
        let mut first_error = None;
        while let Some(result) = results.next().await {
            match result {
                Ok(_) => {
                    cached += 1;
                    progress(OfflineStatus { cached, total });
                }
                Err(e) => {
                    first_error.get_or_insert(e);
                }
            }
        }
        first_error.map_or(Ok(()), Err)
    }

    /// Lê uma chave do disco; se faltar, busca na API e guarda para a próxima.
    async fn key<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        if let Some(bytes) = self.store.read(key) {
            if let Ok(value) = serde_json::from_slice(&bytes) {
                return Ok(value);
            }
        }
        let bytes = self.remote.key(key).await?;
        let value = serde_json::from_slice(&bytes)?;
        let _ = self.store.write(key, &bytes);
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// Um "LouvorJA" mínimo: responde cada caminho com um corpo fixo.
    fn serve(routes: Vec<(&'static str, Vec<u8>)>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut stream = stream;
                let mut buf = [0u8; 4096];
                let n = stream.read(&mut buf).unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]);
                let path = request.split_whitespace().nth(1).unwrap_or("/");
                let path = path.split('?').next().unwrap_or(path);
                let (status, body) = routes
                    .iter()
                    .find(|(p, _)| *p == path)
                    .map(|(_, b)| ("200 OK", b.clone()))
                    .unwrap_or(("404 Not Found", b"{}".to_vec()));
                let head = format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&body);
            }
        });
        base
    }

    fn bundle() -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        for (name, body) in [
            (
                "pt_categories.json",
                r#"[{"id_category":1,"name":"CDs","slug":"aym","albums":[{"id_album":5,"name":"Brilha Jesus"}]}]"#,
            ),
            (
                "pt_musics.json",
                r#"[{"id_music":1,"name":"Nosso Sol é Jesus","lyric":"o nosso sol"}]"#,
            ),
            (
                "album_5.json",
                r#"{"id_album":5,"name":"Brilha Jesus","musics":[{"id_music":1,"name":"Nosso Sol é Jesus","track":1}]}"#,
            ),
        ] {
            zip.start_file(name, options).unwrap();
            zip.write_all(body.as_bytes()).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

    #[tokio::test]
    async fn syncs_bundle_and_fetches_missing_songs_on_demand() {
        let base = serve(vec![
            ("/json_db/config", br#"{"version_number":185}"#.to_vec()),
            ("/db/catalog-bundle", bundle()),
            (
                "/json_db/music_1",
                br#"{"id_music":1,"name":"Nosso Sol \u00e9 Jesus","lyric":[{"lyric":"O nosso sol","order":1,"time":"00:00:05"}]}"#.to_vec(),
            ),
        ]);
        let dir = tempfile::tempdir().unwrap();
        let library = Library::with_api(dir.path().to_path_buf(), &base);
        library.load(Lang::Pt).unwrap();
        assert!(!library.status().installed);

        let mut reported = 0;
        assert!(library
            .sync(Lang::Pt, false, |done, _| reported = done)
            .await
            .unwrap());
        assert!(reported > 0);
        let status = library.status();
        assert_eq!(
            (
                status.installed,
                status.version,
                status.songs,
                status.albums
            ),
            (true, Some(185), 1, 1)
        );

        // Mesma versão: nada para baixar.
        assert!(!library.sync(Lang::Pt, false, |_, _| {}).await.unwrap());

        let album = library.album(5).await.unwrap();
        assert_eq!(album.songs[0].number, Some(1));

        // music_1 não veio no pacote: busca na API e guarda para a próxima.
        let song = library.song(1).await.unwrap();
        assert_eq!(song.slides.len(), 2);
        assert_eq!(song.slides[1].time_ms, Some(5_000));
        assert!(dir.path().join("catalog/music_1.json").is_file());
    }

    #[tokio::test]
    async fn reports_network_errors_without_touching_the_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let library = Library::with_api(dir.path().to_path_buf(), "http://127.0.0.1:9");
        assert!(matches!(
            library.sync(Lang::Pt, true, |_, _| {}).await,
            Err(Error::Network(_))
        ));
        assert!(!library.status().installed);
    }
}
