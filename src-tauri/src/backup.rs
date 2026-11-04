//! Cópia de segurança do que a igreja montou no 7Hinos: ajustes, avisos,
//! culto de hoje, músicas próprias, escala e apresentações.
//!
//! Fica num `.zip` pequeno (o acervo e a Bíblia não entram: vêm da internet
//! ou do pendrive). Uma por semana, sozinha, guardando as últimas.
//!
//! Restaurar com o app aberto mexeria em arquivos em uso; então o `.zip` é
//! aberto numa pasta de espera e o app reinicia. Na abertura seguinte, antes
//! de ler qualquer coisa, os arquivos tomam o lugar dos atuais.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{Error, Result};

/// Pasta das cópias quando a pessoa não escolhe outra.
pub const DIR: &str = "copias-de-seguranca";
const PREFIX: &str = "7Hinos-copia-";
const STAGING: &str = "restaurar.pendente";
/// Uma semana.
pub const INTERVAL_SECS: u64 = 7 * 24 * 60 * 60;

/// O que entra na cópia: nome dentro do `.zip` e onde está no computador.
#[derive(Debug, Clone)]
pub struct Item {
    pub name: &'static str,
    pub path: PathBuf,
}

/// Os itens do 7Hinos, a partir das pastas de ajustes e de dados.
pub fn items(config_dir: &Path, data_dir: &Path) -> Vec<Item> {
    vec![
        Item {
            name: "settings.json",
            path: config_dir.join("settings.json"),
        },
        Item {
            name: "user-songs.json",
            path: data_dir.join("user-songs.json"),
        },
        Item {
            name: crate::roster::FILE,
            path: data_dir.join(crate::roster::FILE),
        },
        Item {
            name: crate::user_songs::MEDIA_DIR,
            path: data_dir.join(crate::user_songs::MEDIA_DIR),
        },
        Item {
            name: crate::DECKS_DIR,
            path: data_dir.join(crate::DECKS_DIR),
        },
    ]
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub path: String,
    pub name: String,
    /// Epoch em segundos.
    pub created_at: u64,
    pub bytes: u64,
}

/// Grava uma cópia nova em `dest` e apaga as mais antigas além de `keep`.
pub fn create(items: &[Item], dest: &Path, keep: usize, label: &str) -> Result<Info> {
    fs::create_dir_all(dest)?;
    let now = chrono::Local::now();
    let mut name = format!("{PREFIX}{}{label}.zip", now.format("%Y-%m-%d-%H%M%S"));
    let mut n = 2;
    while dest.join(&name).exists() {
        name = format!("{PREFIX}{}{label}-{n}.zip", now.format("%Y-%m-%d-%H%M%S"));
        n += 1;
    }
    let path = dest.join(&name);
    let partial = path.with_extension("part");
    write_zip(items, &partial)?;
    fs::rename(&partial, &path)?;
    prune(dest, keep.max(1), &path);
    info(&path).ok_or_else(|| Error::other("A cópia não foi gravada."))
}

fn write_zip(items: &[Item], to: &Path) -> Result<()> {
    let mut zip = zip::ZipWriter::new(fs::File::create(to)?);
    // Áudio e imagem já vêm comprimidos: comprimir de novo só gasta o
    // processador (que na igreja costuma ser fraco).
    let deflate = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let stored =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for item in items {
        for (file, inside) in files_of(item) {
            let options = if inside.ends_with(".json") {
                deflate
            } else {
                stored
            };
            zip.start_file(inside, options)?;
            let mut buf = Vec::new();
            fs::File::open(&file)?.read_to_end(&mut buf)?;
            zip.write_all(&buf)?;
        }
    }
    zip.finish()?;
    Ok(())
}

/// Arquivos de um item com o nome que terão dentro do `.zip`.
fn files_of(item: &Item) -> Vec<(PathBuf, String)> {
    if item.path.is_file() {
        return vec![(item.path.clone(), item.name.to_string())];
    }
    let mut out = Vec::new();
    let mut stack = vec![item.path.clone()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(rel) = path.strip_prefix(&item.path) {
                let rel = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/");
                out.push((path.clone(), format!("{}/{rel}", item.name)));
            }
        }
    }
    out.sort_by(|a, b| a.1.cmp(&b.1));
    out
}

fn info(path: &Path) -> Option<Info> {
    let name = path.file_name()?.to_string_lossy().into_owned();
    if !name.starts_with(PREFIX) || !name.ends_with(".zip") {
        return None;
    }
    let meta = fs::metadata(path).ok()?;
    let created_at = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Some(Info {
        path: path.to_string_lossy().into_owned(),
        name,
        created_at,
        bytes: meta.len(),
    })
}

/// As cópias de uma pasta, da mais nova para a mais antiga.
pub fn list(dir: &Path) -> Vec<Info> {
    let mut all: Vec<Info> = fs::read_dir(dir)
        .map(|entries| entries.flatten().filter_map(|e| info(&e.path())).collect())
        .unwrap_or_default();
    // O nome tem a data e a hora: ordena mesmo se a data do arquivo mudar.
    // (sem o ".zip", para "…-2" vir depois do nome sem número).
    let key = |i: &Info| i.name.trim_end_matches(".zip").to_owned();
    all.sort_by_key(|i| std::cmp::Reverse(key(i)));
    all
}

/// Apaga as cópias mais antigas; a que acabou de ser feita sempre fica.
fn prune(dir: &Path, keep: usize, newest: &Path) {
    let others = list(dir)
        .into_iter()
        .filter(|i| Path::new(&i.path) != newest);
    for old in others.skip(keep - 1) {
        if let Err(e) = fs::remove_file(&old.path) {
            log::warn!("cópia antiga não apagada ({}): {e}", old.name);
        }
    }
}

/// Abre a cópia numa pasta de espera; vale na próxima abertura do app.
pub fn stage_restore(zip_path: &Path, config_dir: &Path, allowed: &[Item]) -> Result<usize> {
    let staging = config_dir.join(STAGING);
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging)?;
    let mut archive = zip::ZipArchive::new(fs::File::open(zip_path)?)
        .map_err(|_| Error::other("Esse arquivo não é uma cópia de segurança do 7Hinos."))?;
    let mut count = 0;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let Some(rel) = entry.enclosed_name() else {
            continue;
        };
        let top = rel
            .components()
            .next()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .unwrap_or_default();
        if entry.is_dir() || !allowed.iter().any(|item| item.name == top) {
            continue;
        }
        let target = staging.join(&rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = fs::File::create(&target)?;
        std::io::copy(&mut entry, &mut out)?;
        count += 1;
    }
    if count == 0 {
        let _ = fs::remove_dir_all(&staging);
        return Err(Error::other(
            "Esse arquivo não é uma cópia de segurança do 7Hinos.",
        ));
    }
    Ok(count)
}

/// Aplica uma restauração pendente. Devolve se aplicou.
///
/// Arquivos soltos (ajustes, escala, músicas próprias) são trocados; nas
/// pastas (mídia das músicas, apresentações) o que veio é copiado por cima e
/// o resto fica, para nunca perder um arquivo que ainda esteja em uso.
pub fn apply_pending(config_dir: &Path, items: &[Item]) -> bool {
    let staging = config_dir.join(STAGING);
    if !staging.is_dir() {
        return false;
    }
    for item in items {
        let from = staging.join(item.name);
        let result = if from.is_file() {
            item.path
                .parent()
                .map(fs::create_dir_all)
                .transpose()
                .and_then(|_| fs::copy(&from, &item.path).map(|_| ()))
        } else if from.is_dir() {
            copy_dir(&from, &item.path)
        } else {
            Ok(())
        };
        if let Err(e) = result {
            log::warn!("restauração de {} falhou: {e}", item.name);
        }
    }
    let _ = fs::remove_dir_all(&staging);
    log::info!("cópia de segurança restaurada");
    true
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)?.flatten() {
        let path = entry.path();
        let target = to.join(entry.file_name());
        if path.is_dir() {
            copy_dir(&path, &target)?;
        } else {
            fs::copy(&path, &target)?;
        }
    }
    Ok(())
}

/// Se já passou uma semana desde a última cópia.
pub fn due(last: Option<u64>, now: u64) -> bool {
    last.is_none_or(|last| now.saturating_sub(last) >= INTERVAL_SECS)
}

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (tempfile::TempDir, PathBuf, PathBuf, Vec<Item>) {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join("config");
        let data = root.path().join("data");
        fs::create_dir_all(&config).unwrap();
        fs::create_dir_all(data.join("apresentacoes/a")).unwrap();
        fs::write(config.join("settings.json"), r#"{"notices":["culto"]}"#).unwrap();
        fs::write(data.join("escala.json"), r#"{"people":[]}"#).unwrap();
        fs::write(data.join("apresentacoes/a/deck.json"), "{}").unwrap();
        fs::write(data.join("apresentacoes/a/1.jpg"), "jpg").unwrap();
        let items = items(&config, &data);
        (root, config, data, items)
    }

    #[test]
    fn backs_up_and_restores_on_next_start() {
        let (root, config, data, items) = setup();
        let dest = root.path().join("copias");
        let made = create(&items, &dest, 5, "").unwrap();
        assert!(made.bytes > 0);
        assert_eq!(list(&dest), vec![made.clone()]);

        // A pessoa muda as coisas depois da cópia…
        fs::write(config.join("settings.json"), "{}").unwrap();
        fs::write(data.join("escala.json"), "{}").unwrap();
        fs::write(data.join("apresentacoes/a/2.jpg"), "nova").unwrap();
        fs::remove_file(data.join("apresentacoes/a/1.jpg")).unwrap();

        // …e restaura: nada muda até reabrir.
        assert_eq!(
            stage_restore(Path::new(&made.path), &config, &items).unwrap(),
            4
        );
        assert_eq!(fs::read_to_string(data.join("escala.json")).unwrap(), "{}");

        assert!(apply_pending(&config, &items));
        assert_eq!(
            fs::read_to_string(config.join("settings.json")).unwrap(),
            r#"{"notices":["culto"]}"#
        );
        assert_eq!(
            fs::read_to_string(data.join("escala.json")).unwrap(),
            r#"{"people":[]}"#
        );
        assert!(data.join("apresentacoes/a/1.jpg").is_file());
        // O que foi criado depois não some.
        assert!(data.join("apresentacoes/a/2.jpg").is_file());
        assert!(!apply_pending(&config, &items));
    }

    #[test]
    fn keeps_only_the_newest() {
        let (root, _config, _data, items) = setup();
        let dest = root.path().join("copias");
        for _ in 0..4 {
            create(&items, &dest, 2, "").unwrap();
        }
        assert_eq!(list(&dest).len(), 2);
    }

    #[test]
    fn refuses_other_zips() {
        let (root, config, _data, items) = setup();
        let other = root.path().join("outro.zip");
        let mut zip = zip::ZipWriter::new(fs::File::create(&other).unwrap());
        zip.start_file("../fora.txt", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"x").unwrap();
        zip.start_file("qualquer.txt", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"x").unwrap();
        zip.finish().unwrap();
        assert!(stage_restore(&other, &config, &items).is_err());
        assert!(!root.path().join("fora.txt").exists());
        assert!(!apply_pending(&config, &items));
    }

    #[test]
    fn weekly() {
        assert!(due(None, 100));
        assert!(!due(Some(100), 100 + INTERVAL_SECS - 1));
        assert!(due(Some(100), 100 + INTERVAL_SECS));
    }
}
