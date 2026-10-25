//! "Apagar tudo e começar de novo".
//!
//! Com o app aberto, arquivos estão em uso (no Windows não dá para apagar um
//! arquivo aberto). Então o pedido só deixa uma marca e reinicia; na abertura
//! seguinte, antes de qualquer coisa ser lida, as pastas são esvaziadas.

use std::fs;
use std::io;
use std::path::Path;

const MARK: &str = "apagar-tudo.pendente";

pub fn request(config_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(config_dir)?;
    fs::write(config_dir.join(MARK), b"")
}

/// Esvazia as pastas se houver um pedido pendente. Devolve se apagou.
pub fn apply_pending(config_dir: &Path, dirs: &[&Path]) -> bool {
    let mark = config_dir.join(MARK);
    if !mark.is_file() {
        return false;
    }
    for dir in dirs {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path == mark {
                continue;
            }
            let removed = if path.is_dir() {
                fs::remove_dir_all(&path)
            } else {
                fs::remove_file(&path)
            };
            if let Err(e) = removed {
                log::warn!("não deu para apagar {}: {e}", path.display());
            }
        }
    }
    let _ = fs::remove_file(&mark);
    log::info!("dados apagados a pedido; começando do zero");
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wipes_only_when_requested() {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join("config");
        let data = root.path().join("data");
        fs::create_dir_all(data.join("media/a")).unwrap();
        fs::create_dir_all(&config).unwrap();
        fs::write(config.join("settings.json"), "{}").unwrap();
        fs::write(data.join("bible.db"), "x").unwrap();

        assert!(!apply_pending(&config, &[&config, &data]));
        assert!(data.join("bible.db").exists());

        request(&config).unwrap();
        assert!(apply_pending(&config, &[&config, &data]));
        assert_eq!(fs::read_dir(&data).unwrap().count(), 0);
        assert_eq!(fs::read_dir(&config).unwrap().count(), 0);
        assert!(!apply_pending(&config, &[&config, &data]));
    }
}
