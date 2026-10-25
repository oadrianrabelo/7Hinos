//! Cache local de mídia (áudios, capas e fundos dos slides).
//!
//! Cada arquivo remoto tem um lugar fixo no disco que espelha o caminho da URL:
//! `https://files.louvorja.com.br/musics/pt/Album/Faixa.mp3` vira
//! `<dados>/media/musics/pt/Album/Faixa.mp3`. Assim não é preciso índice: o
//! arquivo existe ou não existe.
//!
//! Antes de baixar, o cache também procura nas pastas locais que já têm a
//! mídia do LouvorJA (a instalação do LouvorJA desktop, os downloads da versão
//! antiga do 7Hinos). É isso que faz o app funcionar numa igreja sem internet.

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use futures_util::StreamExt;
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use url::Url;

use crate::error::{Error, Result};

/// Endereço de onde o LouvorJA serve a mídia do catálogo.
pub const FILES_URL: &str = "https://files.louvorja.com.br";

/// Uma mídia como a interface a enxerga: o endereço remoto e, se já baixada, o
/// caminho local (que tem preferência).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Media {
    pub url: String,
    pub path: Option<String>,
}

/// Pastas do computador organizadas como as do LouvorJA: áudios em
/// `<pasta>/<álbum>/<faixa>`, fundos e capas direto na pasta.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocalFolders {
    pub music: Vec<PathBuf>,
    pub images: Vec<PathBuf>,
    pub covers: Vec<PathBuf>,
}

const AUDIO_EXTENSIONS: [&str; 5] = ["mp3", "opus", "ogg", "m4a", "wav"];
const IMAGE_EXTENSIONS: [&str; 6] = ["jpg", "jpeg", "png", "bmp", "webp", "gif"];

#[derive(Clone)]
pub struct MediaCache {
    root: PathBuf,
    client: reqwest::Client,
    local: Arc<RwLock<LocalFolders>>,
}

impl MediaCache {
    pub fn new(root: PathBuf, client: reqwest::Client) -> Self {
        Self {
            root,
            client,
            local: Default::default(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn set_local(&self, folders: LocalFolders) {
        *self.local.write().expect("pastas locais") = folders;
    }

    /// Resolve um campo `url_*` do catálogo. Vazio vira `None`.
    pub fn resolve(&self, raw: &str) -> Option<Media> {
        let url = absolute_url(raw)?;
        let path = self.find(&url).map(|p| p.to_string_lossy().into_owned());
        Some(Media { url, path })
    }

    pub fn is_cached(&self, url: &str) -> bool {
        self.find(url).is_some()
    }

    /// O arquivo no disco para esta URL: baixado pelo 7Hinos ou numa pasta local.
    pub fn find(&self, url: &str) -> Option<PathBuf> {
        if let Some(path) = self.local_path(url).filter(|p| p.is_file()) {
            return Some(path);
        }
        let segments = decoded_segments(url)?;
        let local = self.local.read().expect("pastas locais");
        let (roots, rest, extensions): (&[PathBuf], &[String], &[&str]) =
            match segments.first().map(String::as_str) {
                // musics/<idioma>/<álbum>/<faixa>
                Some("musics") if segments.len() > 2 => {
                    (&local.music, &segments[2..], &AUDIO_EXTENSIONS)
                }
                Some("images") => (&local.images, &segments[1..], &IMAGE_EXTENSIONS),
                Some("covers") => (&local.covers, &segments[1..], &IMAGE_EXTENSIONS),
                _ => return None,
            };
        roots.iter().find_map(|root| {
            find_variant(
                &rest.iter().fold(root.clone(), |p, s| p.join(s)),
                extensions,
            )
        })
    }

    pub fn local_path(&self, url: &str) -> Option<PathBuf> {
        let segments = decoded_segments(url)?;
        let mut path = self.root.clone();
        for segment in &segments {
            path.push(sanitize(segment));
        }
        (!segments.is_empty()).then_some(path)
    }

    /// Baixa o arquivo se ainda não estiver no disco. Escreve num `.part` e só
    /// renomeia no fim, então um download interrompido nunca parece completo.
    pub async fn download(&self, url: &str) -> Result<PathBuf> {
        let target = self
            .local_path(url)
            .ok_or_else(|| Error::other(format!("Endereço de mídia inválido: {url}")))?;
        if let Some(existing) = self.find(url) {
            return Ok(existing);
        }
        if let Some(dir) = target.parent() {
            tokio::fs::create_dir_all(dir).await?;
        }
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(Error::Http(response.status().as_u16()));
        }
        let partial = target.with_extension("part");
        let mut file = tokio::fs::File::create(&partial).await?;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            file.write_all(&chunk?).await?;
        }
        file.flush().await?;
        drop(file);
        tokio::fs::rename(&partial, &target).await?;
        Ok(target)
    }

    /// Espaço ocupado pela mídia baixada, em bytes.
    pub fn disk_usage(&self) -> u64 {
        dir_size(&self.root)
    }

    pub fn clear(&self) -> Result<()> {
        if self.root.exists() {
            std::fs::remove_dir_all(&self.root)?;
        }
        Ok(())
    }
}

/// Segmentos do caminho da URL, decodificados e sem nada que saia da pasta.
fn decoded_segments(url: &str) -> Option<Vec<String>> {
    let url = Url::parse(url).ok()?;
    url.path_segments()?
        .map(|segment| {
            let segment = percent_decode_str(segment).decode_utf8().ok()?;
            let valid = !segment.is_empty()
                && segment != "."
                && segment != ".."
                && !segment.contains(['/', '\\']);
            valid.then(|| segment.into_owned())
        })
        .collect()
}

/// O próprio arquivo ou o mesmo nome com outra extensão: o LouvorJA desktop
/// guarda MP3 e BMP onde o acervo online às vezes tem Opus e JPG.
fn find_variant(path: &Path, extensions: &[&str]) -> Option<PathBuf> {
    if path.is_file() {
        return Some(path.to_path_buf());
    }
    extensions
        .iter()
        .map(|ext| path.with_extension(ext))
        .find(|candidate| candidate.is_file())
}

/// Normaliza os formatos de endereço que já existiram no catálogo
/// (completo, `/file/...`, `/musics/...`) para uma URL completa e codificada.
pub fn absolute_url(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let (origin, path) = match raw.split_once("://") {
        Some((scheme, rest)) => {
            let (host, path) = rest.split_once('/').unwrap_or((rest, ""));
            (format!("{scheme}://{host}"), path)
        }
        None => {
            let path = raw.trim_start_matches('/');
            (
                FILES_URL.to_owned(),
                path.strip_prefix("file/").unwrap_or(path),
            )
        }
    };
    // O catálogo traz nomes de arquivo crus: "Quem é Jesus?.mp3" não tem query.
    let path = path.replace('?', "%3F").replace('#', "%23");
    Url::parse(&format!("{origin}/{path}"))
        .ok()
        .map(String::from)
}

/// Nomes de arquivo válidos em Windows, macOS e Linux.
fn sanitize(segment: &str) -> String {
    let cleaned: String = segment
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    cleaned.trim_end_matches(['.', ' ']).to_owned()
}

fn dir_size(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| match entry.metadata() {
            Ok(meta) if meta.is_dir() => dir_size(&entry.path()),
            Ok(meta) => meta.len(),
            Err(_) => 0,
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cache() -> MediaCache {
        MediaCache::new(PathBuf::from("/data/media"), reqwest::Client::new())
    }

    #[test]
    fn normalizes_every_known_url_shape() {
        let expected = "https://files.louvorja.com.br/musics/pt/1992%20-%20Brilha%20Jesus/Nosso%20Sol%20%C3%89%20Jesus.mp3";
        for raw in [
            "https://files.louvorja.com.br/musics/pt/1992 - Brilha Jesus/Nosso Sol É Jesus.mp3",
            "/musics/pt/1992 - Brilha Jesus/Nosso Sol É Jesus.mp3",
            "/file/musics/pt/1992 - Brilha Jesus/Nosso Sol É Jesus.mp3",
        ] {
            assert_eq!(absolute_url(raw).as_deref(), Some(expected), "{raw}");
        }
        assert_eq!(
            absolute_url("https://files.louvorja.com.br/musics/Quem é?.mp3").as_deref(),
            Some("https://files.louvorja.com.br/musics/Quem%20%C3%A9%3F.mp3")
        );
        assert_eq!(absolute_url("  "), None);
    }

    #[test]
    fn maps_urls_to_mirrored_paths() {
        let url = absolute_url("/covers/2026.jpg").unwrap();
        assert_eq!(
            cache().local_path(&url),
            Some(PathBuf::from("/data/media/covers/2026.jpg"))
        );
        let url = absolute_url("/musics/pt/Hinário/01 - Santo?.opus").unwrap();
        assert_eq!(
            cache().local_path(&url),
            Some(PathBuf::from(
                "/data/media/musics/pt/Hinário/01 - Santo_.opus"
            ))
        );
    }

    #[test]
    fn finds_media_in_local_louvorja_folders() {
        let dir = tempfile::tempdir().unwrap();
        let musicas = dir.path().join("musicas");
        let imagens = dir.path().join("imagens");
        std::fs::create_dir_all(musicas.join("1992 - Brilha Jesus")).unwrap();
        std::fs::create_dir_all(&imagens).unwrap();
        std::fs::write(musicas.join("1992 - Brilha Jesus/Brilha Jesus.mp3"), b"x").unwrap();
        std::fs::write(imagens.join("ceu.bmp"), b"x").unwrap();

        let cache = MediaCache::new(dir.path().join("media"), reqwest::Client::new());
        let opus = absolute_url("/musics/pt/1992 - Brilha Jesus/Brilha Jesus.opus").unwrap();
        let jpg = absolute_url("/images/ceu.jpg").unwrap();
        assert!(!cache.is_cached(&opus));

        cache.set_local(LocalFolders {
            music: vec![musicas.clone()],
            images: vec![imagens.clone()],
            covers: vec![],
        });
        assert_eq!(
            cache.find(&opus),
            Some(musicas.join("1992 - Brilha Jesus/Brilha Jesus.mp3"))
        );
        assert_eq!(cache.find(&jpg), Some(imagens.join("ceu.bmp")));
        assert!(cache
            .find(&absolute_url("/covers/ceu.jpg").unwrap())
            .is_none());
    }

    #[test]
    fn rejects_path_traversal() {
        assert_eq!(cache().local_path("https://x.com/a/..%2Fb"), None);
        assert_eq!(cache().local_path("https://x.com/"), None);
    }
}
