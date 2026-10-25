//! Preferências do usuário, guardadas num JSON legível na pasta de configuração.

use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::catalog::model::Lang;
use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UiMode {
    /// Botões grandes e só o essencial, para quem tem pouca familiaridade.
    #[default]
    Simple,
    /// Biblioteca completa, painel de projeção e atalhos.
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SlideStyle {
    pub uppercase: bool,
    /// Multiplicador do tamanho da letra nos slides.
    pub text_scale: f32,
    pub show_images: bool,
    /// Duração da transição entre slides, em milissegundos.
    pub transition_ms: u32,
}

impl Default for SlideStyle {
    fn default() -> Self {
        Self {
            uppercase: true,
            text_scale: 1.0,
            show_images: true,
            transition_ms: 450,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoItem {
    pub path: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    /// `false` até a pessoa passar pela tela de boas-vindas.
    pub onboarded: bool,
    pub mode: UiMode,
    pub theme: Theme,
    pub language: Lang,
    /// Multiplicador do tamanho da interface (texto e botões).
    pub ui_scale: f32,
    pub slide: SlideStyle,
    pub favorites: Vec<u32>,
    pub recent: Vec<u32>,
    pub videos: Vec<VideoItem>,
    /// Guarda no disco as músicas tocadas para funcionar sem internet.
    pub keep_played: bool,
    pub auto_update_catalog: bool,
    pub auto_update_app: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            onboarded: false,
            mode: UiMode::Simple,
            theme: Theme::System,
            language: Lang::Pt,
            ui_scale: 1.0,
            slide: SlideStyle::default(),
            favorites: Vec::new(),
            recent: Vec::new(),
            videos: Vec::new(),
            keep_played: true,
            auto_update_catalog: true,
            auto_update_app: true,
        }
    }
}

pub const RECENT_LIMIT: usize = 30;

impl Settings {
    pub fn push_recent(&mut self, id: u32) {
        self.recent.retain(|&r| r != id);
        self.recent.insert(0, id);
        self.recent.truncate(RECENT_LIMIT);
    }

    /// Mantém valores dentro do que a interface sabe mostrar.
    fn sanitized(mut self) -> Self {
        self.ui_scale = self.ui_scale.clamp(0.8, 1.6);
        self.slide.text_scale = self.slide.text_scale.clamp(0.6, 1.6);
        self.slide.transition_ms = self.slide.transition_ms.min(2_000);
        self.recent.truncate(RECENT_LIMIT);
        self
    }
}

pub struct SettingsStore {
    path: PathBuf,
    current: Mutex<Settings>,
}

impl SettingsStore {
    /// Um arquivo ausente ou ilegível começa com os padrões, nunca com erro.
    pub fn load(path: PathBuf) -> Self {
        let current = std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Settings>(&bytes).ok())
            .unwrap_or_default()
            .sanitized();
        Self {
            path,
            current: Mutex::new(current),
        }
    }

    pub fn get(&self) -> Settings {
        self.current.lock().expect("preferências").clone()
    }

    pub fn update(&self, change: impl FnOnce(&mut Settings)) -> Result<Settings> {
        let mut current = self.current.lock().expect("preferências");
        let mut next = current.clone();
        change(&mut next);
        let next = next.sanitized();
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let partial = self.path.with_extension("part");
        std::fs::write(&partial, serde_json::to_vec_pretty(&next)?)?;
        std::fs::rename(&partial, &self.path)?;
        *current = next.clone();
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_list_is_unique_newest_first_and_bounded() {
        let mut s = Settings::default();
        for id in 0..40 {
            s.push_recent(id);
        }
        s.push_recent(35);
        assert_eq!(s.recent.len(), RECENT_LIMIT);
        assert_eq!(&s.recent[..3], &[35, 39, 38]);
    }

    #[test]
    fn persists_and_tolerates_partial_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(
            &path,
            r#"{ "mode": "full", "uiScale": 9, "desconhecido": 1 }"#,
        )
        .unwrap();

        let store = SettingsStore::load(path.clone());
        let s = store.get();
        assert_eq!(s.mode, UiMode::Full);
        assert_eq!(s.ui_scale, 1.6);
        assert!(s.keep_played);

        store.update(|s| s.favorites.push(7)).unwrap();
        assert_eq!(SettingsStore::load(path).get().favorites, [7]);
    }
}
