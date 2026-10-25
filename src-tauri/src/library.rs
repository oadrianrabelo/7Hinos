//! A biblioteca de hinos: junta o catálogo em disco, a API do LouvorJA e o
//! cache de mídia atrás de uma interface pequena. É o único lugar que sabe de
//! onde cada coisa vem; os comandos só repassam.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use futures_util::{stream, StreamExt};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::catalog::louvorja::{self, Install};
use crate::catalog::model::{self, Album, Collection, Lang, Song, SongSummary};
use crate::catalog::raw::{RawAlbum, RawAlbumRef, RawMusic, RawSongRow};
use crate::catalog::remote::{Remote, API_URL};
use crate::catalog::store::Store;
use crate::catalog::Catalog;
use crate::error::{Error, Result};
use crate::media::MediaCache;
use crate::sync::{LockExt, RwLockExt};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogStatus {
    pub installed: bool,
    pub version: Option<u32>,
    pub songs: usize,
    pub albums: usize,
    /// Última vez (epoch em segundos) que o LouvorJA respondeu.
    pub last_check: Option<u64>,
    /// Versão publicada vista na última resposta.
    pub available: Option<u32>,
    /// Erro da última tentativa, se ela falhou.
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct SyncInfo {
    last_check: Option<u64>,
    available: Option<u32>,
    last_error: Option<String>,
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

/// O que baixar para usar sem internet.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Scope {
    All,
    Collection { id: u32 },
    Album { id: u32 },
    Songs { ids: Vec<u32> },
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineStatus {
    pub cached: usize,
    pub total: usize,
}

/// Coleções que o LouvorJA publica fora do pacote: (id, chave, nome).
const EXTRA_COLLECTIONS: [(u32, &str, &str); 2] = [
    (9001, "children_albums", "Infantis"),
    (9002, "doxology_albums", "Doxologia"),
];

pub struct Library {
    data_dir: PathBuf,
    store: Store,
    remote: Remote,
    media: MediaCache,
    catalog: RwLock<Option<Arc<Catalog>>>,
    sync_lock: tokio::sync::Mutex<()>,
    info: std::sync::Mutex<SyncInfo>,
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
            data_dir: data_dir.clone(),
            store: Store::new(data_dir.join("catalog")),
            remote: Remote::new(client.clone(), api),
            media: MediaCache::new(data_dir.join("media"), client),
            catalog: RwLock::new(None),
            sync_lock: tokio::sync::Mutex::new(()),
            info: Default::default(),
        }
    }

    pub fn media(&self) -> &MediaCache {
        &self.media
    }

    pub fn data_dir(&self) -> &std::path::Path {
        &self.data_dir
    }

    pub fn catalog_dir(&self) -> &std::path::Path {
        self.store.dir()
    }

    /// Instala o catálogo de uma pasta (a `acervo/` de um pacote de pendrive).
    pub async fn import_catalog_dir(&self, dir: PathBuf, lang: Lang) -> Result<()> {
        let _guard = self.sync_lock.lock().await;
        let store = Store::new(self.store.dir().to_path_buf());
        tokio::task::spawn_blocking(move || -> Result<()> {
            let source = Store::new(dir.clone());
            let mut entries = Vec::new();
            for entry in std::fs::read_dir(&dir)?.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if let Some(key) = name.strip_suffix(".json") {
                    if let Some(bytes) = source.read(key) {
                        entries.push((key.to_owned(), bytes));
                    }
                }
            }
            store.install_entries(&entries, source.version().unwrap_or(0))?;
            Ok(())
        })
        .await
        .map_err(|e| Error::other(e.to_string()))??;
        self.load(lang)
    }

    /// (Re)carrega as listas do idioma. Sem catálogo baixado, fica vazio.
    pub fn load(&self, lang: Lang) -> Result<()> {
        let loaded = match Catalog::load(&self.store, lang) {
            Ok(catalog) => Some(Arc::new(catalog)),
            Err(Error::CatalogMissing) => None,
            Err(e) => return Err(e),
        };
        *self.catalog.writing() = loaded;
        Ok(())
    }

    pub fn catalog(&self) -> Result<Arc<Catalog>> {
        self.catalog.reading().clone().ok_or(Error::CatalogMissing)
    }

    pub fn status(&self) -> CatalogStatus {
        let info = self.info.locked().clone();
        let catalog = self.catalog().ok();
        CatalogStatus {
            installed: catalog.is_some(),
            version: catalog.as_ref().and_then(|c| c.version),
            songs: catalog.as_ref().map_or(0, |c| c.song_count()),
            albums: catalog.as_ref().map_or(0, |c| c.album_count()),
            last_check: info.last_check,
            available: info.available,
            last_error: info.last_error,
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
        let result = self.sync_inner(lang, force, progress).await;
        let mut info = self.info.locked();
        match &result {
            Ok(_) => info.last_error = None,
            Err(e) => info.last_error = Some(e.to_string()),
        }
        result
    }

    async fn sync_inner(
        &self,
        lang: Lang,
        force: bool,
        progress: impl FnMut(u64, Option<u64>),
    ) -> Result<bool> {
        let _guard = self.sync_lock.lock().await;
        let check = self.check_update().await?;
        {
            let mut info = self.info.locked();
            info.last_check = Some(now_secs());
            info.available = Some(check.available);
        }
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
        // Já guarda as chaves avulsas enquanto há internet, para valer offline.
        let _ = self.collections().await;
        let _ = self.hymnal_1996().await;
        Ok(true)
    }

    /// Importa o catálogo do LouvorJA desktop instalado. Devolve o idioma do
    /// banco importado, que passa a ser o idioma dos hinos.
    pub async fn import_louvorja(&self, install: Install, lang: Lang) -> Result<Lang> {
        let _guard = self.sync_lock.lock().await;
        let store = Store::new(self.store.dir().to_path_buf());
        let lang = tokio::task::spawn_blocking(move || -> Result<Lang> {
            let catalog = louvorja::read(&install, lang)?;
            store.install_entries(&catalog.entries, catalog.version)?;
            Ok(catalog.lang)
        })
        .await
        .map_err(|e| Error::other(e.to_string()))??;
        self.load(lang)?;
        Ok(lang)
    }

    /// Coleções do catálogo mais as que o LouvorJA publica em chaves à parte
    /// (infantis e doxologia), quando ainda não estão entre elas.
    pub async fn collections(&self) -> Result<Vec<Collection>> {
        let catalog = self.catalog()?;
        let mut list = catalog.collections(&self.media);
        for (id, suffix, name) in EXTRA_COLLECTIONS {
            let slug = suffix.trim_end_matches("_albums");
            if list.iter().any(|c| c.slug == slug || c.name == name) {
                continue;
            }
            let key = format!("{}_{suffix}", catalog.lang.code());
            let Some(albums) = self.optional_key::<Vec<RawAlbumRef>>(&key).await else {
                continue;
            };
            if !albums.is_empty() {
                list.push(Collection {
                    id,
                    name: name.to_owned(),
                    slug: slug.to_owned(),
                    albums: albums
                        .iter()
                        .map(|a| model::album_summary(a, &self.media))
                        .collect(),
                });
            }
        }
        Ok(list)
    }

    /// O hinário de 1996 (só existe em português).
    pub async fn hymnal_1996(&self) -> Result<Vec<SongSummary>> {
        let catalog = self.catalog()?;
        let key = format!("{}_hymnal_1996", catalog.lang.code());
        let rows = self
            .optional_key::<Vec<RawSongRow>>(&key)
            .await
            .unwrap_or_default();
        Ok(rows
            .iter()
            .map(|r| model::song_summary(r, Some(r.track)))
            .collect())
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

    /// Todos os arquivos de que um escopo precisa para tocar sem internet.
    pub async fn scope_media(&self, scope: &Scope) -> Result<BTreeSet<String>> {
        let mut urls = BTreeSet::new();
        match scope {
            Scope::All => {
                for collection in self.collections().await? {
                    urls.extend(
                        collection
                            .albums
                            .iter()
                            .filter_map(|a| a.cover.as_ref().map(|m| m.url.clone())),
                    );
                }
                for id in self.catalog()?.song_ids() {
                    // Uma música com problema não impede as outras.
                    if let Ok(song) = self.song_media(id).await {
                        urls.extend(song);
                    }
                }
            }
            Scope::Collection { id } => {
                let collections = self.collections().await?;
                let collection = collections
                    .iter()
                    .find(|c| c.id == *id)
                    .ok_or_else(|| Error::other("Coleção não encontrada."))?;
                for album in &collection.albums {
                    urls.extend(self.album_media(album.id).await?);
                }
            }
            Scope::Album { id } => urls.extend(self.album_media(*id).await?),
            Scope::Songs { ids } => {
                for &id in ids {
                    urls.extend(self.song_media(id).await?);
                }
            }
        }
        Ok(urls)
    }

    pub async fn scope_offline(&self, scope: &Scope) -> Result<OfflineStatus> {
        let urls = self.scope_media(scope).await?;
        Ok(OfflineStatus {
            cached: urls.iter().filter(|u| self.media.is_cached(u)).count(),
            total: urls.len(),
        })
    }

    /// Das músicas pedidas, as que tocam sem internet: o áudio cantado está no
    /// computador (ou a música nem tem áudio, só letra).
    pub async fn available_offline(&self, ids: &[u32]) -> Vec<u32> {
        let mut available = Vec::new();
        for &id in ids {
            let Ok(raw) = self.local_key::<RawMusic>(&format!("music_{id}")) else {
                continue;
            };
            let song = model::song(&raw, &self.media);
            if song.audio.as_ref().is_none_or(|a| a.path.is_some()) {
                available.push(id);
            }
        }
        available
    }

    /// Baixa a mídia que falta, quatro arquivos por vez. Um arquivo que falha
    /// não interrompe os outros; o erro volta no fim.
    pub async fn download(
        &self,
        urls: BTreeSet<String>,
        cancel: &AtomicBool,
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
            .map(|url| async move {
                if cancel.load(Ordering::Relaxed) {
                    return Err(Error::Cancelled);
                }
                self.media.download(&url).await
            })
            .buffer_unordered(4);
        let mut first_error = None;
        while let Some(result) = results.next().await {
            if cancel.load(Ordering::Relaxed) {
                return Err(Error::Cancelled);
            }
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

    /// Só do disco, sem internet (para respostas rápidas na interface).
    fn local_key<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        let bytes = self.store.read(key).ok_or(Error::CatalogMissing)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    /// Chave que pode não existir: lê do disco e só vai à internet quando a
    /// última conversa com o LouvorJA deu certo, para nunca deixar a tela
    /// esperando numa igreja sem conexão.
    async fn optional_key<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        if let Some(value) = self
            .store
            .read(key)
            .and_then(|b| serde_json::from_slice(&b).ok())
        {
            return Some(value);
        }
        let online = {
            let info = self.info.locked();
            info.last_check.is_some() && info.last_error.is_none()
        };
        if !online {
            return None;
        }
        let bytes = self.remote.key(key).await.ok()?;
        let value = serde_json::from_slice(&bytes).ok()?;
        let _ = self.store.write(key, &bytes);
        Some(value)
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

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
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
                "/json_db/pt_children_albums",
                br#"[{"id_album":690,"name":"M\u00fasicas Infantis"}]"#.to_vec(),
            ),
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

        // Infantis vem de uma chave à parte; doxologia não existe neste servidor.
        let names: Vec<String> = library
            .collections()
            .await
            .unwrap()
            .into_iter()
            .map(|c| c.name)
            .collect();
        assert_eq!(names, ["CDs", "Infantis"]);
        assert!(dir.path().join("catalog/pt_children_albums.json").is_file());
        assert!(library.hymnal_1996().await.unwrap().is_empty());

        // music_1 não veio no pacote: busca na API e guarda para a próxima.
        let song = library.song(1).await.unwrap();
        assert_eq!(song.slides.len(), 2);
        assert_eq!(song.slides[1].time_ms, Some(5_000));
        assert!(dir.path().join("catalog/music_1.json").is_file());
    }

    #[tokio::test]
    async fn imports_a_local_louvorja_install_and_plays_offline() {
        let dir = tempfile::tempdir().unwrap();
        let install = crate::catalog::louvorja::tests::fake_install(dir.path());
        let library = Library::with_api(dir.path().join("data"), "http://127.0.0.1:9");
        library.media().set_local(install.folders());

        assert_eq!(
            library.import_louvorja(install, Lang::Es).await.unwrap(),
            Lang::Pt
        );
        let status = library.status();
        assert_eq!(
            (status.installed, status.version, status.songs),
            (true, Some(185), 2)
        );

        let song = library.song(10).await.unwrap();
        assert!(song
            .audio
            .as_ref()
            .unwrap()
            .path
            .as_ref()
            .unwrap()
            .ends_with("Brilha Jesus.mp3"));
        assert!(song.slides[1]
            .image
            .as_ref()
            .unwrap()
            .path
            .as_ref()
            .unwrap()
            .ends_with("ceu.bmp"));
        // O hino 11 tem áudio que não está no computador.
        assert_eq!(library.available_offline(&[10, 11, 99]).await, [10]);
        let hymnal = library
            .scope_offline(&Scope::Songs { ids: vec![11] })
            .await
            .unwrap();
        assert!(hymnal.cached < hymnal.total);
        let cancel = AtomicBool::new(true);
        let urls = library
            .scope_media(&Scope::Songs { ids: vec![11] })
            .await
            .unwrap();
        assert!(matches!(
            library.download(urls, &cancel, |_| {}).await,
            Err(Error::Cancelled)
        ));
        let offline = library
            .scope_offline(&Scope::Album { id: 1 })
            .await
            .unwrap();
        assert_eq!(offline.cached, offline.total);
    }

    #[tokio::test]
    async fn reports_network_errors_without_touching_the_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let library = Library::with_api(dir.path().to_path_buf(), "http://127.0.0.1:9");
        assert!(matches!(
            library.sync(Lang::Pt, true, |_, _| {}).await,
            Err(Error::Network(_))
        ));
        let status = library.status();
        assert!(!status.installed);
        assert!(status.last_error.is_some() && status.last_check.is_none());
    }
}
