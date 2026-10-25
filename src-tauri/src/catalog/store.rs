//! O catálogo no disco: uma pasta com um JSON por chave do `json_db`
//! (`pt_musics.json`, `music_1.json`...) e um arquivo com a versão instalada.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

const VERSION_FILE: &str = "_version";

pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn read(&self, key: &str) -> Option<Vec<u8>> {
        fs::read(self.path(key)?).ok()
    }

    pub fn write(&self, key: &str, bytes: &[u8]) -> Result<()> {
        let path = self
            .path(key)
            .ok_or_else(|| Error::other(format!("Chave inválida: {key}")))?;
        fs::create_dir_all(&self.dir)?;
        let partial = path.with_extension("part");
        fs::write(&partial, bytes)?;
        fs::rename(partial, path)?;
        Ok(())
    }

    pub fn version(&self) -> Option<u32> {
        fs::read_to_string(self.dir.join(VERSION_FILE))
            .ok()?
            .trim()
            .parse()
            .ok()
    }

    /// Instala um pacote ZIP do catálogo.
    pub fn install_bundle(&self, zip_path: &Path, version: u32) -> Result<usize> {
        let mut archive = zip::ZipArchive::new(fs::File::open(zip_path)?)?;
        self.replace(version, |staged| {
            let mut count = 0;
            for i in 0..archive.len() {
                let mut entry = archive.by_index(i)?;
                if !entry.is_file() {
                    continue;
                }
                // Os pacotes já tiveram as chaves na raiz e dentro de `json_db/`.
                let Some(key) = Path::new(entry.name())
                    .file_name()
                    .and_then(|n| n.to_str())
                    .and_then(|n| n.strip_suffix(".json"))
                    .map(str::to_owned)
                else {
                    continue;
                };
                if staged.path(&key).is_none() {
                    continue;
                }
                let mut bytes = Vec::with_capacity(entry.size() as usize);
                entry.read_to_end(&mut bytes)?;
                staged.write(&key, &bytes)?;
                count += 1;
            }
            Ok(count)
        })
    }

    /// Instala chaves já prontas (por exemplo, convertidas do LouvorJA desktop).
    pub fn install_entries(&self, entries: &[(String, Vec<u8>)], version: u32) -> Result<usize> {
        self.replace(version, |staged| {
            for (key, bytes) in entries {
                staged.write(key, bytes)?;
            }
            Ok(entries.len())
        })
    }

    /// Monta o catálogo novo numa pasta ao lado e só troca quando ele está
    /// completo: um pacote ruim nunca estraga o catálogo atual.
    fn replace(&self, version: u32, fill: impl FnOnce(&Store) -> Result<usize>) -> Result<usize> {
        let staging = self.dir.with_extension("new");
        if staging.exists() {
            fs::remove_dir_all(&staging)?;
        }
        fs::create_dir_all(&staging)?;
        let staged = Store::new(staging.clone());

        let count = match fill(&staged) {
            Ok(count) => count,
            Err(e) => {
                let _ = fs::remove_dir_all(&staging);
                return Err(e);
            }
        };

        let complete = ["pt", "es"].iter().any(|lang| {
            staged
                .path(&format!("{lang}_categories"))
                .is_some_and(|p| p.is_file())
                && staged
                    .path(&format!("{lang}_musics"))
                    .is_some_and(|p| p.is_file())
        });
        if !complete {
            fs::remove_dir_all(&staging)?;
            return Err(Error::other(
                "O pacote baixado não tem a lista de hinos. Tente de novo mais tarde.",
            ));
        }
        fs::write(staging.join(VERSION_FILE), version.to_string())?;

        let previous = self.dir.with_extension("old");
        if previous.exists() {
            fs::remove_dir_all(&previous)?;
        }
        if self.dir.exists() {
            fs::rename(&self.dir, &previous)?;
        }
        fs::rename(&staging, &self.dir)?;
        let _ = fs::remove_dir_all(previous);
        Ok(count)
    }

    /// Só aceita os nomes que o `json_db` usa, então a chave nunca sai da pasta.
    fn path(&self, key: &str) -> Option<PathBuf> {
        let valid = !key.is_empty()
            && !key.starts_with('_')
            && key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        valid.then(|| self.dir.join(format!("{key}.json")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_zip(path: &Path, files: &[(&str, &str)]) {
        let mut zip = zip::ZipWriter::new(fs::File::create(path).unwrap());
        let options = zip::write::SimpleFileOptions::default();
        for (name, body) in files {
            zip.start_file(*name, options).unwrap();
            zip.write_all(body.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
    }

    #[test]
    fn rejects_keys_that_escape_the_folder() {
        let store = Store::new(PathBuf::from("/c"));
        assert!(store.path("music_1").is_some());
        assert!(store.path("../x").is_none());
        assert!(store.path("a/b").is_none());
        assert!(store.path("_version").is_none());
    }

    #[test]
    fn installs_bundle_and_replaces_previous_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().join("catalog"));
        store.write("stale", b"{}").unwrap();

        let zip = dir.path().join("bundle.zip");
        make_zip(
            &zip,
            &[
                ("_bundle_manifest.json", "{}"),
                ("pt_categories.json", "[]"),
                ("json_db/pt_musics.json", "[]"),
                ("music_1.json", "{}"),
                ("../evil.json", "{}"),
                ("readme.txt", "x"),
            ],
        );
        assert_eq!(store.install_bundle(&zip, 185).unwrap(), 4);
        assert_eq!(store.version(), Some(185));
        assert!(store.read("music_1").is_some());
        assert!(
            store.read("evil").is_some(),
            "o nome é reduzido ao arquivo, sem caminho"
        );
        assert!(store.read("stale").is_none());
        assert!(!dir.path().join("evil.json").exists());
    }

    #[test]
    fn keeps_current_catalog_when_bundle_is_incomplete() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().join("catalog"));
        store.write("pt_musics", b"[1]").unwrap();

        let zip = dir.path().join("bundle.zip");
        make_zip(&zip, &[("music_1.json", "{}")]);
        assert!(store.install_bundle(&zip, 2).is_err());
        assert_eq!(store.read("pt_musics").unwrap(), b"[1]");
    }
}
