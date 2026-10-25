//! O palco: o que está sendo projetado agora.
//!
//! O estado vive aqui, num lugar só. A janela de controle pede mudanças e cada
//! tela de projeção apenas desenha o estado que recebe; uma tela aberta no meio
//! do culto pergunta o estado atual e já entra sincronizada.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::catalog::model::Slide;
use crate::sync::LockExt;

pub const STAGE_EVENT: &str = "stage://changed";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Scene {
    #[default]
    Black,
    Slide {
        slide: Slide,
        title: String,
    },
}

/// Um vídeo em reprodução. As telas calculam a posição atual a partir de
/// `position + (agora - updatedAt)` enquanto `playing`, então um único
/// comando mantém várias telas no mesmo ponto.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VideoState {
    pub path: String,
    pub title: String,
    pub playing: bool,
    /// Segundos.
    pub position: f64,
    /// Instante (epoch em ms) em que `position` era verdade.
    pub updated_at: f64,
    pub looping: bool,
    pub volume: f64,
    /// Rótulos das telas que mostram o vídeo. Só a primeira toca o som.
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StageState {
    pub scene: Scene,
    pub video: Option<VideoState>,
}

#[derive(Default)]
pub struct Stage {
    state: Mutex<StageState>,
}

impl Stage {
    pub fn get(&self) -> StageState {
        self.state.locked().clone()
    }

    pub fn show(&self, app: &AppHandle, scene: Scene) {
        self.change(app, |s| s.scene = scene);
    }

    pub fn set_video(&self, app: &AppHandle, video: Option<VideoState>) {
        self.change(app, |s| s.video = video);
    }

    /// Uma tela fechada sai da lista do vídeo; sem telas, o vídeo para.
    pub fn forget_output(&self, app: &AppHandle, label: &str) {
        self.change(app, |s| {
            if let Some(video) = &mut s.video {
                video.targets.retain(|t| t != label);
                if video.targets.is_empty() {
                    s.video = None;
                }
            }
        });
    }

    fn change(&self, app: &AppHandle, f: impl FnOnce(&mut StageState)) {
        let snapshot = {
            let mut state = self.state.locked();
            let before = state.clone();
            f(&mut state);
            if *state == before {
                return;
            }
            state.clone()
        };
        let _ = app.emit(STAGE_EVENT, snapshot);
    }
}
