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
        /// O slide seguinte, para o monitor de retorno.
        #[serde(default)]
        next: Option<Box<Slide>>,
    },
    /// Uma página de apresentação (PDF, imagem, lição), mostrada inteira.
    Image {
        /// Caminho do arquivo no computador.
        src: String,
        title: String,
        page: u32,
        total: u32,
        /// A página seguinte, para o monitor de retorno.
        #[serde(default)]
        next: Option<String>,
    },
    /// Tela de espera: logo e texto da igreja (configurados em Ajustes).
    Idle,
    /// Relógio, ou contagem regressiva até `until` (epoch em ms).
    Clock { until: Option<f64>, label: String },
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

/// Só para quem está à frente: aparece nos monitores de retorno, nunca no
/// telão.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Pulpit {
    /// Recado do operador ("5 minutos", "Pode encerrar").
    pub message: Option<String>,
    pub timer: Option<Timer>,
}

/// Cronômetro do pregador. Como no vídeo, cada tela calcula o tempo atual:
/// `elapsedMs + (agora - startedAt)` enquanto corre.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Timer {
    pub elapsed_ms: f64,
    /// Instante (epoch em ms) em que voltou a correr; `None` = pausado.
    pub started_at: Option<f64>,
    /// Tempo combinado; passou dele, o retorno fica vermelho.
    pub limit_ms: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StageState {
    pub scene: Scene,
    pub video: Option<VideoState>,
    #[serde(default)]
    pub pulpit: Pulpit,
    /// Sobe a cada mudança. Uma tela que vê um número diferente do seu sabe
    /// que perdeu alguma atualização e pede o estado de novo.
    #[serde(default)]
    pub rev: u64,
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

    pub fn set_pulpit(&self, app: &AppHandle, pulpit: Pulpit) {
        self.change(app, |s| s.pulpit = pulpit);
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

    /// Uma tela reaberta continua mostrando o vídeo que a anterior mostrava.
    pub fn rename_output(&self, app: &AppHandle, old: &str, new: &str) {
        self.change(app, |s| {
            if let Some(video) = &mut s.video {
                for target in &mut video.targets {
                    if target == old {
                        *target = new.to_owned();
                    }
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
            state.rev = before.rev.wrapping_add(1);
            state.clone()
        };
        let _ = app.emit(STAGE_EVENT, snapshot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slide_scene_without_next_still_reads() {
        let json = r#"{"kind":"slide","title":"Castelo Forte","slide":{"kind":"lyric",
            "text":"Castelo forte","aux":null,"image":null,"imagePosition":5,"repeat":false,
            "timeMs":null,"instrumentalTimeMs":null}}"#;
        let scene: Scene = serde_json::from_str(json).unwrap();
        assert!(matches!(scene, Scene::Slide { next: None, .. }));
    }

    #[test]
    fn idle_and_clock_round_trip() {
        for scene in [
            Scene::Idle,
            Scene::Clock {
                until: Some(1.0),
                label: "O culto começa em".into(),
            },
        ] {
            let json = serde_json::to_string(&scene).unwrap();
            assert_eq!(serde_json::from_str::<Scene>(&json).unwrap(), scene);
        }
    }
}
