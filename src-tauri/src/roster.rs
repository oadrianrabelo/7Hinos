//! Escala da igreja (pregação, plataforma, recepção, Escola Sabatina…).
//!
//! O documento inteiro é montado e validado pela interface (lib/roster.ts);
//! aqui ele só é guardado em `escala.json`, de forma atômica, e viaja no
//! pacote de pendrive junto com as músicas próprias.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::error::Result;
use crate::sync::LockExt;

pub const FILE: &str = "escala.json";

pub struct Roster {
    file: PathBuf,
    lock: Mutex<()>,
}

impl Roster {
    pub fn new(data_dir: &Path) -> Self {
        Roster {
            file: data_dir.join(FILE),
            lock: Mutex::default(),
        }
    }

    /// O documento salvo, ou `null` se ainda não houver escala.
    pub fn load(&self) -> serde_json::Value {
        let _guard = self.lock.locked();
        std::fs::read(&self.file)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or(serde_json::Value::Null)
    }

    pub fn save(&self, doc: &serde_json::Value) -> Result<()> {
        let _guard = self.lock.locked();
        if let Some(dir) = self.file.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let partial = self.file.with_extension("part");
        std::fs::write(&partial, serde_json::to_vec_pretty(doc)?)?;
        std::fs::rename(partial, &self.file)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_and_loads() {
        let dir = tempfile::tempdir().unwrap();
        let roster = Roster::new(dir.path());
        assert!(roster.load().is_null());
        let doc = serde_json::json!({ "version": 1, "people": [{ "id": "a", "name": "Maria" }] });
        roster.save(&doc).unwrap();
        assert_eq!(roster.load(), doc);
        // Arquivo estragado não derruba nada: volta como vazio.
        std::fs::write(dir.path().join(FILE), b"{oops").unwrap();
        assert!(roster.load().is_null());
    }
}
