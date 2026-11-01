//! Do estado do palco ao que uma tela nativa mostra: o fundo (imagem, página
//! ou vídeo) e o texto por cima, em eventos ASS. Funções puras: mesma
//! entrada, mesma saída, para testar sem abrir o mpv.
//!
//! Os tamanhos seguem o CSS da projeção antiga (SlideView, SceneView e
//! MonitorView): `cqh`/`cqw` viram porcentagens da área, e a letra é medida
//! com a própria fonte.

use std::path::{Path, PathBuf};

use crate::catalog::model::{Slide, SlideKind};
use crate::displays::Role;
use crate::settings::Settings;
use crate::stage::{Scene, StageState};

use super::ass::{self, Line};
use super::text::{wrap, Font, Fonts};

/// Uma área da tela, em pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    /// `n`% da altura (o `cqh` do CSS).
    fn cqh(&self, n: f32) -> f32 {
        self.h * n / 100.0
    }

    fn cqw(&self, n: f32) -> f32 {
        self.w * n / 100.0
    }

    /// `min(a cqh, b cqw)`, o jeito do CSS de caber nas duas direções.
    fn min(&self, a: f32, b: f32) -> f32 {
        self.cqh(a).min(self.cqw(b))
    }

    fn inset(&self, dy: f32, dx: f32) -> Rect {
        Rect {
            x: self.x + dx,
            y: self.y + dy,
            w: (self.w - 2.0 * dx).max(1.0),
            h: (self.h - 2.0 * dy).max(1.0),
        }
    }
}

/// O que fica atrás do texto.
#[derive(Debug, Clone, PartialEq)]
pub enum Background {
    Black,
    /// Imagem montada pelo 7Hinos no tamanho da tela (ver background.rs).
    Composed(Compose),
    /// Página de apresentação mostrada pelo mpv, com zoom.
    Page(Page),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Compose {
    /// Foto de fundo (caminho ou endereço), cortada para cobrir a tela.
    pub image: Option<String>,
    /// Grade 3×3 do LouvorJA (5 = centro).
    pub position: u8,
    pub shade: Shade,
    /// Logo da tela de espera, já posicionado.
    pub logo: Option<(String, Rect)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Shade {
    /// O escurecido dos slides (radial + linear).
    Slide,
    /// Degradê circular a 50% 35%: centro, meio (70%) e borda.
    Radial([(u8, u8, u8); 3]),
    /// Cor lisa.
    Solid((u8, u8, u8)),
    /// Escurece por igual (0 a 1).
    Darken(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub path: String,
    /// Aproximação: centro (0 a 1) e escala (1 = página inteira).
    pub zoom: Option<(f32, f32, f32)>,
    /// Área onde a página aparece (o retorno usa só o meio da tela).
    pub area: Rect,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub background: Background,
    pub events: Vec<String>,
    /// De quanto em quanto tempo redesenhar (relógio, sorteio), em ms.
    pub tick: Option<u64>,
}

pub struct Ctx<'a> {
    pub fonts: &'a Fonts,
    pub settings: &'a Settings,
    pub user_media: &'a Path,
    /// Agora, em ms desde 1970.
    pub now: f64,
    /// Horas no fuso do computador ("19:42").
    pub clock: String,
    pub screen: Rect,
    /// Medidas (largura, altura) de uma imagem local, para o logo e as
    /// páginas. Nas provas, uma função de mentira.
    pub image_size: &'a dyn Fn(&str) -> Option<(u32, u32)>,
}

impl Ctx<'_> {
    fn user_file(&self, name: &Option<String>) -> Option<String> {
        let name = name.as_deref()?.trim();
        if name.is_empty() {
            return None;
        }
        let path: PathBuf = self.user_media.join(name);
        Some(path.to_string_lossy().into_owned())
    }
}

/// Uma tela (telão ou retorno) num instante, com o texto em `opacity`
/// (para as transições).
pub fn frame(ctx: &Ctx, state: &StageState, role: Role, opacity: f32) -> Frame {
    match role {
        Role::Audience => audience(ctx, &state.scene, opacity),
        Role::Monitor => monitor(ctx, state, opacity),
    }
}

fn audience(ctx: &Ctx, scene: &Scene, opacity: f32) -> Frame {
    let screen = ctx.screen;
    let mut f = Frame {
        background: Background::Black,
        events: Vec::new(),
        tick: None,
    };
    match scene {
        Scene::Black => {}
        Scene::Slide { slide, .. } if slide.kind == SlideKind::Text => {
            f.background = themed_background(ctx, None);
            notice(ctx, slide, screen, opacity, false, &mut f.events);
        }
        Scene::Slide { slide, .. } => {
            let style = &ctx.settings.slide;
            let image = style
                .show_images
                .then_some(slide.image.as_ref())
                .flatten()
                .map(|m| m.path.clone().unwrap_or_else(|| m.url.clone()))
                .filter(|s| !s.is_empty());
            f.background = Background::Composed(Compose {
                shade: if image.is_some() {
                    Shade::Slide
                } else {
                    Shade::Radial([(0x1d, 0x45, 0x60), (0x0b, 0x1d, 0x2b), (0x05, 0x0b, 0x11)])
                },
                image,
                position: slide.image_position,
                logo: None,
            });
            slide_text(ctx, slide, screen, opacity, &mut f.events);
        }
        Scene::Image { src, overlay, .. } => {
            let zoom = page_zoom(overlay);
            f.background = Background::Page(Page {
                path: src.clone(),
                zoom,
                area: screen,
            });
            page_overlay(ctx, src, overlay, screen, opacity, &mut f.events);
        }
        Scene::Idle => {
            idle(ctx, screen, opacity, false, &mut f);
        }
        Scene::Clock { until, label } => {
            f.background = themed_background(ctx, None);
            clock(ctx, *until, label, screen, opacity, false, &mut f.events);
            f.tick = Some(250);
        }
        Scene::Draw { .. } => {
            f.background = themed_background(ctx, None);
            f.tick = draw(ctx, scene, screen, opacity, false, &mut f.events);
        }
    }
    f
}

// --- Telão ---------------------------------------------------------------

fn px_size(r: Rect, cqw: f32, cqh: f32) -> f32 {
    r.cqw(cqw).min(r.cqh(cqh))
}

/// Tamanho da letra dos slides, igual ao `fitText` do CSS (lib/slides.ts).
pub fn fit_text(text: &str, kind: SlideKind, scale: f32) -> (f32, f32) {
    if matches!(kind, SlideKind::Verse | SlideKind::Text) {
        let chars = text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .count()
            .max(20) as f32;
        let lines = text.lines().filter(|l| !l.trim().is_empty()).count().max(1) as f32;
        let by_area = ((144.0 * 74.0) / (chars * 0.45 * 1.3)).sqrt();
        let by_lines = 74.0 / (lines * 1.3);
        let size = 9.5f32.min(by_area).min(by_lines);
        return (size * 1.78 * scale, size * scale);
    }
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let count = lines.len().max(1) as f32;
    let longest = lines
        .iter()
        .map(|l| l.trim().chars().count())
        .max()
        .unwrap_or(0)
        .max(8) as f32;
    let cap: f32 = if kind == SlideKind::Title { 14.0 } else { 12.0 };
    let by_height = cap.min(78.0 / (count * 1.12));
    let by_width = 90.0 / (longest * 0.5);
    (by_width * scale, by_height * scale)
}

/// Um bloco de texto: linhas já quebradas, com tamanho e altura de linha.
struct Block {
    lines: Vec<String>,
    font: Font,
    px: f32,
    line_height: f32,
    spacing: f32,
    color: String,
    opacity: f32,
}

impl Block {
    fn new(text: &str, font: Font, px: f32, line_height: f32, spacing: f32, max_w: f32) -> Block {
        let lines = wrap(text, max_w, &|s| font.width(s, px, spacing));
        Block {
            lines,
            font,
            px,
            line_height,
            spacing,
            color: "#ffffff".into(),
            opacity: 1.0,
        }
    }

    fn color(mut self, c: &str) -> Block {
        self.color = c.to_owned();
        self
    }

    fn opacity(mut self, o: f32) -> Block {
        self.opacity = o;
        self
    }

    fn height(&self) -> f32 {
        self.lines.len() as f32 * self.px * self.line_height
    }

    /// Diminui a letra até o bloco caber na altura (texto enorme).
    fn shrink_to(&mut self, text: &str, max_w: f32, max_h: f32) {
        for _ in 0..12 {
            if self.height() <= max_h || self.px < 8.0 {
                break;
            }
            self.px *= 0.92;
            let (font, px, sp) = (&self.font, self.px, self.spacing);
            self.lines = wrap(text, max_w, &|s| font.width(s, px, sp));
        }
    }
}

/// Empilha blocos no meio de `area` (como o flex-column centralizado).
fn stack(
    blocks: &[Block],
    gap: f32,
    area: Rect,
    align: u8,
    opacity: f32,
    shadow: f32,
    out: &mut Vec<String>,
) {
    let total: f32 =
        blocks.iter().map(Block::height).sum::<f32>() + gap * blocks.len().saturating_sub(1) as f32;
    let mut y = area.y + (area.h - total) / 2.0;
    let x = match align {
        4 => area.x,
        _ => area.x + area.w / 2.0,
    };
    for b in blocks {
        let lh = b.px * b.line_height;
        for line in &b.lines {
            out.push(
                Line {
                    text: line,
                    font: &b.font,
                    px: b.px,
                    x,
                    y: y + lh / 2.0,
                    align,
                    color: &b.color,
                    opacity: opacity * b.opacity,
                    spacing: b.spacing,
                    shadow,
                }
                .event(),
            );
            y += lh;
        }
        y += gap;
    }
}

fn upper_if(text: &str, upper: bool) -> String {
    if upper {
        text.to_uppercase()
    } else {
        text.to_owned()
    }
}

/// A letra de um hino ou um versículo, como o SlideView.
fn slide_text(ctx: &Ctx, slide: &Slide, screen: Rect, opacity: f32, out: &mut Vec<String>) {
    let style = &ctx.settings.slide;
    let prose = matches!(slide.kind, SlideKind::Verse | SlideKind::Text);
    let upper = style.uppercase && !prose;
    let font_id = if slide.kind == SlideKind::Verse && !style.bible_font.is_empty() {
        &style.bible_font
    } else {
        &style.font
    };
    let content = screen.inset(screen.cqh(6.0), screen.cqw(5.0));
    let mut blocks = Vec::new();
    if let Some(aux) = slide
        .aux
        .as_deref()
        .filter(|a| !a.is_empty() && slide.kind != SlideKind::Verse)
    {
        let px = screen.min(4.6, 5.0);
        let font = ctx.fonts.get(font_id, 600);
        blocks.push(
            Block::new(&upper_if(aux, upper), font, px, 1.2, 0.08, content.w)
                .color("#ffc94a")
                .opacity(0.95),
        );
    }
    let mut reserved = 0.0;
    if !slide.text.is_empty() {
        let (cqw, cqh) = fit_text(&slide.text, slide.kind, style.text_scale);
        let px = px_size(screen, cqw, cqh);
        let (weight, lh, spacing) = match slide.kind {
            SlideKind::Title => (700, 1.12, 0.02),
            _ if prose => (500, 1.28, 0.0),
            _ => (600, 1.12, 0.01),
        };
        let color = match slide.kind {
            SlideKind::Title => "#ffd36b",
            _ if slide.repeat => "#ffe39a",
            _ => "#ffffff",
        };
        let text = upper_if(&slide.text, upper);
        let max_w = if prose {
            content.w.min(screen.cqw(92.0))
        } else {
            content.w
        };
        let mut block = Block::new(
            &text,
            ctx.fonts.get(font_id, weight),
            px,
            lh,
            spacing,
            max_w,
        )
        .color(color);
        reserved = blocks.iter().map(Block::height).sum::<f32>() + screen.cqh(8.0);
        block.shrink_to(&text, max_w, (content.h - reserved).max(content.h * 0.5));
        blocks.push(block);
    }
    let _ = reserved;
    if let Some(reference) = slide
        .aux
        .as_deref()
        .filter(|a| !a.is_empty() && slide.kind == SlideKind::Verse)
    {
        let px = screen.min(4.4, 4.8);
        blocks.push(
            Block::new(
                &reference.to_uppercase(),
                ctx.fonts.get(font_id, 600),
                px,
                1.2,
                0.06,
                content.w,
            )
            .color("#ffc94a"),
        );
    }
    stack(&blocks, screen.cqh(2.2), content, 5, opacity, 1.0, out);
}

/// O fundo de avisos, espera, relógio e sorteio (Avisos e relógio → Aparência).
fn themed_background(ctx: &Ctx, logo: Option<(String, Rect)>) -> Background {
    let st = &ctx.settings.screen_style;
    if let Some(image) = ctx.user_file(&st.background_image) {
        return Background::Composed(Compose {
            image: Some(image),
            position: 5,
            shade: Shade::Darken(0.4),
            logo,
        });
    }
    Background::Composed(Compose {
        image: None,
        position: 5,
        shade: screen_shade(&st.background),
        logo,
    })
}

/// "#rrggbb", "radial:#centro,#borda" ou "gradient" (o azul de antes).
pub fn screen_shade(value: &str) -> Shade {
    let stops = if value == "gradient" {
        Some(("#1d4560", "#050b11"))
    } else {
        value
            .strip_prefix("radial:")
            .and_then(|v| v.split_once(','))
    };
    if let Some((c, e)) = stops {
        if let (Some(c), Some(e)) = (ass::rgb(c), ass::rgb(e)) {
            let mix = |a: u8, b: u8| (f32::from(a) * 0.35 + f32::from(b) * 0.65).round() as u8;
            return Shade::Radial([c, (mix(c.0, e.0), mix(c.1, e.1), mix(c.2, e.2)), e]);
        }
    }
    Shade::Solid(ass::rgb(value).unwrap_or((0, 0, 0)))
}

struct Themed<'a> {
    font: &'a str,
    text: &'a str,
    accent: &'a str,
}

fn themed<'a>(ctx: &'a Ctx, plain: bool) -> Themed<'a> {
    let st = &ctx.settings.screen_style;
    Themed {
        font: &st.font,
        text: if plain { "#ffffff" } else { &st.text_color },
        accent: &st.accent_color,
    }
}

fn notice(ctx: &Ctx, slide: &Slide, area: Rect, opacity: f32, plain: bool, out: &mut Vec<String>) {
    let t = themed(ctx, plain);
    let mut blocks = Vec::new();
    if let Some(aux) = slide.aux.as_deref().filter(|a| !a.is_empty()) {
        blocks.push(
            Block::new(
                &aux.to_uppercase(),
                ctx.fonts.get(t.font, 600),
                area.min(7.0, 5.5),
                1.2,
                0.06,
                area.cqw(88.0),
            )
            .color(t.accent),
        );
    }
    let (cqw, cqh) = fit_text(&slide.text, SlideKind::Text, ctx.settings.slide.text_scale);
    let px = px_size(area, cqw, cqh);
    let max_w = area.cqw(88.0);
    let mut block = Block::new(
        &slide.text,
        ctx.fonts.get(t.font, 600),
        px,
        1.25,
        0.0,
        max_w,
    )
    .color(t.text);
    block.shrink_to(&slide.text, max_w, area.cqh(80.0));
    blocks.push(block);
    stack(&blocks, area.cqh(4.0), area, 5, opacity, 0.6, out);
}

fn idle(ctx: &Ctx, area: Rect, opacity: f32, plain: bool, f: &mut Frame) {
    let idle = &ctx.settings.idle;
    let t = themed(ctx, plain);
    let mut blocks = Vec::new();
    let text = idle.text.trim();
    if !text.is_empty() {
        blocks.push(
            Block::new(
                text,
                ctx.fonts.get(t.font, 600),
                area.min(8.0, 6.0),
                1.15,
                0.0,
                area.cqw(85.0),
            )
            .color(t.text),
        );
    }
    // O logo vai na imagem de fundo; aqui só se calcula onde.
    let logo = ctx.user_file(&idle.image).and_then(|path| {
        let (iw, ih) = (ctx.image_size)(&path)?;
        let (mw, mh) = (area.cqw(70.0), area.cqh(55.0));
        let scale = (mw / iw as f32).min(mh / ih as f32);
        Some((path, iw as f32 * scale, ih as f32 * scale))
    });
    let gap = area.cqh(4.0);
    let text_h: f32 = blocks.iter().map(Block::height).sum();
    let logo_h = logo.as_ref().map_or(0.0, |l| l.2);
    let total = text_h
        + logo_h
        + if logo.is_some() && !blocks.is_empty() {
            gap
        } else {
            0.0
        };
    let top = area.y + (area.h - total) / 2.0;
    let logo_rect = logo.map(|(path, w, h)| {
        (
            path,
            Rect {
                x: area.x + (area.w - w) / 2.0,
                y: top,
                w,
                h,
            },
        )
    });
    if !blocks.is_empty() {
        let text_top = top + logo_h + if logo_rect.is_some() { gap } else { 0.0 };
        let text_area = Rect {
            x: area.x,
            y: text_top,
            w: area.w,
            h: text_h,
        };
        stack(&blocks, 0.0, text_area, 5, opacity, 0.6, &mut f.events);
    }
    if idle.show_clock {
        let px = area.cqh(6.0);
        let font = ctx.fonts.get(t.font, 600);
        f.events.push(
            Line {
                text: &ctx.clock,
                font: &font,
                px,
                x: area.x + area.w - area.cqw(4.0),
                y: area.y + area.h - area.cqh(4.0) - px * 0.6,
                align: 6,
                color: t.text,
                opacity: opacity * 0.8,
                spacing: 0.0,
                shadow: 0.6,
            }
            .event(),
        );
        f.tick = Some(1000);
    }
    if !plain {
        f.background = themed_background(ctx, logo_rect);
    } else if let Some((path, rect)) = logo_rect {
        f.background = Background::Composed(Compose {
            image: None,
            position: 5,
            shade: Shade::Solid((0, 0, 0)),
            logo: Some((path, rect)),
        });
    }
}

/// "04:59", "1:02:03" ou "00:00" quando acabou.
pub fn countdown_text(until: f64, now: f64) -> String {
    let total = ((until - now) / 1000.0).ceil().max(0.0) as u64;
    let (h, m, s) = (total / 3600, (total % 3600) / 60, total % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

fn clock(
    ctx: &Ctx,
    until: Option<f64>,
    label: &str,
    area: Rect,
    opacity: f32,
    plain: bool,
    out: &mut Vec<String>,
) {
    let t = themed(ctx, plain);
    let mut blocks = Vec::new();
    if !label.trim().is_empty() {
        blocks.push(
            Block::new(
                &label.to_uppercase(),
                ctx.fonts.get(t.font, 600),
                area.min(7.0, 5.5),
                1.2,
                0.06,
                area.cqw(90.0),
            )
            .color(t.accent),
        );
    }
    let (text, done) = match until {
        Some(u) => (countdown_text(u, ctx.now), ctx.now >= u),
        None => (ctx.clock.clone(), false),
    };
    blocks.push(
        Block::new(
            &text,
            ctx.fonts.get(t.font, 700),
            area.min(30.0, 24.0),
            1.0,
            0.02,
            area.w,
        )
        .color(if done { t.accent } else { t.text }),
    );
    stack(&blocks, area.cqh(4.0), area, 5, opacity, 0.6, out);
}

/// O sorteio: embaralha os candidatos cada vez mais devagar e para no
/// resultado (igual ao lib/draw.ts). Devolve de quanto em quanto redesenhar.
fn draw(
    ctx: &Ctx,
    scene: &Scene,
    area: Rect,
    opacity: f32,
    plain: bool,
    out: &mut Vec<String>,
) -> Option<u64> {
    let Scene::Draw {
        title,
        result,
        candidates,
        started_at,
        duration_ms,
    } = scene
    else {
        return None;
    };
    let t = themed(ctx, plain);
    let (text, done) = draw_frame(result, candidates, *started_at, *duration_ms, ctx.now);
    let mut blocks = Vec::new();
    if !title.trim().is_empty() {
        blocks.push(
            Block::new(
                &title.to_uppercase(),
                ctx.fonts.get(t.font, 600),
                area.min(7.0, 5.5),
                1.2,
                0.06,
                area.cqw(90.0),
            )
            .color(t.accent),
        );
    }
    let size = 26f32.min(150.0 / (text.chars().count().max(4) as f32));
    blocks.push(
        Block::new(
            &text,
            ctx.fonts.get(t.font, 700),
            area.min(size, size * 0.8),
            1.05,
            0.0,
            area.cqw(92.0),
        )
        .color(if done { t.accent } else { t.text })
        .opacity(if done { 1.0 } else { 0.85 }),
    );
    stack(&blocks, area.cqh(4.0), area, 5, opacity, 0.6, out);
    (!done).then_some(40)
}

pub fn draw_frame(
    result: &str,
    candidates: &[String],
    started_at: f64,
    duration_ms: u32,
    now: f64,
) -> (String, bool) {
    let t = (now - started_at) / f64::from(duration_ms.max(1));
    if t >= 1.0 || candidates.len() < 2 {
        return (result.to_owned(), t >= 1.0);
    }
    if t < 0.0 {
        return (candidates[0].clone(), false);
    }
    let ease = 1.0 - (1.0 - t).powi(3);
    let ticks = (ease * (f64::from(duration_ms) / 1000.0) * 14.0).floor() as usize;
    let index = (ticks * 7919 + 13) % candidates.len();
    let text = if candidates[index] == result && t < 0.9 {
        &candidates[(index + 1) % candidates.len()]
    } else {
        &candidates[index]
    };
    (text.clone(), false)
}

// --- Páginas (PDF, imagens, lição) -----------------------------------------

fn page_zoom(overlay: &serde_json::Value) -> Option<(f32, f32, f32)> {
    let z = overlay.get("zoom")?;
    let n = |k: &str| {
        z.get(k)
            .and_then(serde_json::Value::as_f64)
            .map(|v| v as f32)
    };
    let scale = n("scale")?;
    (scale > 1.001).then(|| (n("x").unwrap_or(0.5), n("y").unwrap_or(0.5), scale.min(8.0)))
}

/// Onde a página fica na tela (o `camera` de lib/overlay.ts).
pub fn camera(zoom: Option<(f32, f32, f32)>, area: Rect, aspect: f32) -> (Rect, f32, f32) {
    let (fw, fh) = if area.w / area.h > aspect {
        (area.h * aspect, area.h)
    } else {
        (area.w, area.w / aspect)
    };
    let (zx, zy, scale) = zoom.unwrap_or((0.5, 0.5, 1.0));
    let (w, h) = (fw * scale, fh * scale);
    let axis = |want: f32, size: f32, screen: f32| {
        if size <= screen {
            0.5
        } else {
            let half = screen / (2.0 * size);
            want.clamp(half, 1.0 - half)
        }
    };
    let (x, y) = (axis(zx, w, area.w), axis(zy, h, area.h));
    (
        Rect {
            x: area.x + area.w / 2.0 - x * w,
            y: area.y + area.h / 2.0 - y * h,
            w,
            h,
        },
        x,
        y,
    )
}

/// Caneta, marca-texto, foco e laser por cima da página.
fn page_overlay(
    ctx: &Ctx,
    src: &str,
    overlay: &serde_json::Value,
    area: Rect,
    opacity: f32,
    out: &mut Vec<String>,
) {
    if overlay.is_null() {
        return;
    }
    let Some((iw, ih)) = (ctx.image_size)(src) else {
        return;
    };
    let (page, _, _) = camera(page_zoom(overlay), area, iw as f32 / ih.max(1) as f32);
    let to_screen = |p: &serde_json::Value| -> Option<(f32, f32)> {
        let x = p.get(0)?.as_f64()? as f32;
        let y = p.get(1)?.as_f64()? as f32;
        Some((page.x + x * page.w, page.y + y * page.h))
    };
    if let Some(strokes) = overlay.get("strokes").and_then(|s| s.as_array()) {
        for stroke in strokes {
            let points: Vec<(f32, f32)> = stroke
                .get("points")
                .and_then(|p| p.as_array())
                .map(|pts| pts.iter().filter_map(to_screen).collect())
                .unwrap_or_default();
            let width = stroke.get("width").and_then(|w| w.as_f64()).unwrap_or(4.0) as f32 / 1000.0
                * page.w;
            let color = stroke
                .get("color")
                .and_then(|c| c.as_str())
                .unwrap_or("#ff3b30");
            let marker = stroke.get("tool").and_then(|t| t.as_str()) == Some("marker");
            if let Some(path) = stroke_path(&points, width, !marker) {
                out.push(ass::shape(
                    &path,
                    color,
                    opacity * if marker { 0.38 } else { 1.0 },
                    0.6,
                ));
            }
        }
    }
    if let Some(spot) = overlay.get("spotlight").filter(|s| !s.is_null()) {
        let n = |k: &str| spot.get(k).and_then(|v| v.as_f64()).unwrap_or(0.5) as f32;
        let (cx, cy, r) = (
            page.x + n("x") * page.w,
            page.y + n("y") * page.h,
            n("r") * page.w,
        );
        let s = ctx.screen;
        // Tela toda escura, com um furo redondo (\iclip) onde está o foco.
        out.push(format!(
            "{{\\an7\\pos(0,0)\\bord0\\shad0\\c&H000000&\\1a{}\\iclip({})\\p1}}m 0 0 l {w:.0} 0 {w:.0} {h:.0} 0 {h:.0}{{\\p0}}",
            ass::alpha(0.72 * opacity),
            circle(cx, cy, r),
            w = s.x + s.w,
            h = s.y + s.h,
        ));
    }
    if let Some(laser) = overlay.get("laser").filter(|l| !l.is_null()) {
        let n = |k: &str| laser.get(k).and_then(|v| v.as_f64()).unwrap_or(0.5) as f32;
        let (cx, cy) = (page.x + n("x") * page.w, page.y + n("y") * page.h);
        let r = ctx.screen.w * 0.008;
        out.push(ass::shape(
            &circle(cx, cy, r * 2.2),
            "#ff2a2a",
            0.45 * opacity,
            r,
        ));
        out.push(ass::shape(&circle(cx, cy, r), "#ff3b30", opacity, 1.0));
        out.push(ass::shape(
            &circle(cx, cy, r * 0.4),
            "#ffffff",
            opacity,
            0.5,
        ));
    }
}

/// Círculo como caminho ASS (quatro curvas de Bézier).
fn circle(cx: f32, cy: f32, r: f32) -> String {
    let k = r * 0.5523;
    format!(
        "m {:.1} {:.1} b {:.1} {:.1} {:.1} {:.1} {:.1} {:.1} b {:.1} {:.1} {:.1} {:.1} {:.1} {:.1} b {:.1} {:.1} {:.1} {:.1} {:.1} {:.1} b {:.1} {:.1} {:.1} {:.1} {:.1} {:.1}",
        cx + r, cy,
        cx + r, cy + k, cx + k, cy + r, cx, cy + r,
        cx - k, cy + r, cx - r, cy + k, cx - r, cy,
        cx - r, cy - k, cx - k, cy - r, cx, cy - r,
        cx + k, cy - r, cx + r, cy - k, cx + r, cy,
    )
}

/// Um traço de caneta vira um polígono: um lado na ida, o outro na volta,
/// com bolinhas nas pontas quando `round`.
fn stroke_path(points: &[(f32, f32)], width: f32, round: bool) -> Option<String> {
    let half = (width / 2.0).max(0.5);
    match points {
        [] => None,
        [p] => Some(circle(p.0, p.1, half)),
        _ => {
            let mut left = Vec::with_capacity(points.len());
            let mut right = Vec::with_capacity(points.len());
            for i in 0..points.len() {
                let a = points[i.saturating_sub(1)];
                let b = points[(i + 1).min(points.len() - 1)];
                let (dx, dy) = (b.0 - a.0, b.1 - a.1);
                let len = (dx * dx + dy * dy).sqrt().max(0.001);
                let (nx, ny) = (-dy / len * half, dx / len * half);
                let p = points[i];
                left.push((p.0 + nx, p.1 + ny));
                right.push((p.0 - nx, p.1 - ny));
            }
            let mut path = format!("m {:.1} {:.1} l", left[0].0, left[0].1);
            for p in left.iter().skip(1).chain(right.iter().rev()) {
                path.push_str(&format!(" {:.1} {:.1}", p.0, p.1));
            }
            if round {
                let (s, e) = (points[0], points[points.len() - 1]);
                path.push(' ');
                path.push_str(&circle(s.0, s.1, half));
                path.push(' ');
                path.push_str(&circle(e.0, e.1, half));
            }
            Some(path)
        }
    }
}

// --- Retorno ---------------------------------------------------------------

/// Altura do cabeçalho do retorno (título, cronômetro e horas).
fn header_height(screen: Rect) -> f32 {
    screen.cqh(12.0)
}

fn monitor(ctx: &Ctx, state: &StageState, opacity: f32) -> Frame {
    let screen = ctx.screen;
    let opts = &ctx.settings.monitor;
    let scene = &state.scene;
    let ui = "sans";
    let mut f = Frame {
        background: Background::Black,
        events: Vec::new(),
        tick: Some(500),
    };
    let top = monitor_top(ctx, state, false, &mut f.events);

    // Rodapé: o que vem a seguir.
    let footer_h = if opts.show_next && !mirrored(scene) {
        screen.cqh(9.0)
    } else {
        0.0
    };
    let bottom = screen.y + screen.h - footer_h;
    if footer_h > 0.0 {
        let next = match scene {
            Scene::Slide { next, .. } => Some(
                next.as_ref()
                    .map_or("Fim".to_owned(), |n| n.text.replace('\n', " / ")),
            ),
            Scene::Image { next, page, .. } => Some(if next.is_some() {
                format!("Página {}", page + 1)
            } else {
                "Fim".into()
            }),
            _ => Some("—".into()),
        };
        footer(
            ctx,
            ui,
            next.as_deref().unwrap_or(""),
            bottom,
            footer_h,
            &mut f.events,
        );
    }

    let main = Rect {
        x: screen.x,
        y: top,
        w: screen.w,
        h: (bottom - top).max(1.0),
    }
    .inset(screen.cqh(2.0), screen.cqw(3.0));
    match scene {
        _ if mirrored(scene) => {
            // Avisos, espera, relógio e sorteio: o telão em miniatura, em 16:9.
            let w = main.w.min(main.h * 16.0 / 9.0);
            let h = w * 9.0 / 16.0;
            let mini = Rect {
                x: main.x + (main.w - w) / 2.0,
                y: main.y + (main.h - h) / 2.0,
                w,
                h,
            };
            let mut inner = Frame {
                background: Background::Black,
                events: Vec::new(),
                tick: None,
            };
            match scene {
                Scene::Slide { slide, .. } => notice(
                    ctx,
                    slide,
                    mini,
                    opacity,
                    !opts.show_background,
                    &mut inner.events,
                ),
                Scene::Idle => idle(ctx, mini, opacity, true, &mut inner),
                Scene::Clock { until, label } => clock(
                    ctx,
                    *until,
                    label,
                    mini,
                    opacity,
                    !opts.show_background,
                    &mut inner.events,
                ),
                _ => {
                    inner.tick = draw(
                        ctx,
                        scene,
                        mini,
                        opacity,
                        !opts.show_background,
                        &mut inner.events,
                    );
                }
            }
            f.events.extend(inner.events);
            if let Some(t) = inner.tick {
                f.tick = Some(t.min(500));
            }
            if let Background::Composed(c) = inner.background {
                f.background = Background::Composed(c);
            }
        }
        Scene::Slide { slide, .. } => {
            if opts.show_background {
                if let Some(image) = slide
                    .image
                    .as_ref()
                    .map(|m| m.path.clone().unwrap_or_else(|| m.url.clone()))
                {
                    f.background = Background::Composed(Compose {
                        image: Some(image),
                        position: slide.image_position,
                        shade: Shade::Darken(0.62),
                        logo: None,
                    });
                }
            }
            monitor_slide(
                ctx,
                slide,
                main,
                opacity,
                opts.show_background,
                &mut f.events,
            );
        }
        Scene::Image { src, overlay, .. } => {
            f.background = Background::Page(Page {
                path: src.clone(),
                zoom: page_zoom(overlay),
                area: main,
            });
            page_overlay(ctx, src, overlay, main, opacity, &mut f.events);
        }
        _ => {
            let font = ctx.fonts.get(ui, 500);
            let px = main.cqh(10.0);
            f.events.push(
                Line {
                    text: &status_text(state),
                    font: &font,
                    px,
                    x: main.x + main.w / 2.0,
                    y: main.y + main.h / 2.0,
                    align: 5,
                    color: "#ffffff",
                    opacity: 0.45,
                    spacing: 0.0,
                    shadow: 0.0,
                }
                .event(),
            );
        }
    }
    f
}

/// Cabeçalho do retorno (título, cronômetro e horas) e o recado do púlpito.
/// Também vai por cima do vídeo. Devolve onde o resto da tela começa.
pub fn monitor_top(ctx: &Ctx, state: &StageState, video: bool, out: &mut Vec<String>) -> f32 {
    let screen = ctx.screen;
    let opts = &ctx.settings.monitor;
    let scene = &state.scene;
    let ui = "sans";
    let header = header_height(screen);
    if video {
        // Faixa escura para o cabeçalho aparecer sobre qualquer vídeo.
        out.push(ass::rect(
            screen.x, screen.y, screen.w, header, "#000000", 0.55, 0.0,
        ));
    }
    let title = match scene {
        _ if video => status_text(state),
        Scene::Image {
            title, page, total, ..
        } if opts.show_title => format!("{title} · {page}/{total}"),
        Scene::Slide { slide, .. } if slide.kind == SlideKind::Text => "Aviso".into(),
        Scene::Slide { title, .. } if opts.show_title => title.clone(),
        Scene::Slide { .. } | Scene::Image { .. } => String::new(),
        _ => status_text(state),
    };
    let mut right = screen.x + screen.w - screen.cqw(2.5);
    let mid = screen.y + header / 2.0;
    if opts.show_clock {
        right = clock_badge(
            ctx, ui, "AGORA", &ctx.clock, "#ffd36b", None, right, mid, out,
        ) - screen.cqw(2.0);
    }
    if let Some(timer) = &state.pulpit.timer {
        let view = super::timer::display(timer, ctx.now);
        let (fg, bg, bg_opacity) = if view.over || view.warn {
            (
                "#ffffff",
                if view.over { "#d93025" } else { "#c62828" },
                1.0,
            )
        } else if view.limited {
            ("#ff8a80", "#d93025", 0.18)
        } else {
            ("#ffffff", "#ffffff", 0.1)
        };
        right = clock_badge(
            ctx,
            ui,
            &view.label.to_uppercase(),
            &view.text,
            fg,
            Some((bg, bg_opacity)),
            right,
            mid,
            out,
        ) - screen.cqw(2.0);
    }
    let title_font = ctx.fonts.get(ui, 600);
    let title_px = screen.cqh(3.6);
    let left = screen.x + screen.cqw(2.5);
    let title = truncate(&title, &title_font, title_px, (right - left).max(0.0));
    out.push(
        Line {
            text: &title,
            font: &title_font,
            px: title_px,
            x: left,
            y: mid,
            align: 4,
            color: "#ffffff",
            opacity: 0.7,
            spacing: 0.0,
            shadow: 0.0,
        }
        .event(),
    );
    out.push(ass::rect(
        screen.x,
        screen.y + header - 1.0,
        screen.w,
        1.0,
        "#ffffff",
        0.12,
        0.0,
    ));

    // Recado do púlpito, numa faixa colorida.
    let mut top = screen.y + header;
    if let Some(message) = state
        .pulpit
        .message
        .as_deref()
        .filter(|m| !m.trim().is_empty())
    {
        let font_id = if opts.message_font.is_empty() {
            ui
        } else {
            opts.message_font.as_str()
        };
        let font = ctx.fonts.get(font_id, 700);
        let px = screen.cqh(6.5);
        let pad = screen.cqh(1.8);
        let block = Block::new(message, font, px, 1.15, 0.0, screen.w - screen.cqw(5.0))
            .color(&opts.message_color);
        let h = block.height() + pad * 2.0;
        out.push(ass::rect(
            screen.x,
            top,
            screen.w,
            h,
            &opts.message_background,
            1.0,
            0.0,
        ));
        let band = Rect {
            x: screen.x,
            y: top,
            w: screen.w,
            h,
        };
        stack(&[block], 0.0, band, 5, 1.0, 0.0, out);
        top += h;
    }
    top
}

/// Avisos, espera, relógio e sorteio aparecem no retorno como no telão.
fn mirrored(scene: &Scene) -> bool {
    match scene {
        Scene::Idle | Scene::Clock { .. } | Scene::Draw { .. } => true,
        Scene::Slide { slide, .. } => slide.kind == SlideKind::Text,
        _ => false,
    }
}

fn status_text(state: &StageState) -> String {
    if let Some(v) = state.video.as_ref().filter(|v| v.playing) {
        return format!("Vídeo: {}", v.title);
    }
    match &state.scene {
        Scene::Black => "Tela preta".into(),
        Scene::Idle => "Tela de espera".into(),
        Scene::Clock {
            until: Some(_),
            label,
        } if !label.is_empty() => label.clone(),
        Scene::Clock { until: Some(_), .. } => "Contagem regressiva".into(),
        Scene::Clock { .. } => "Relógio no telão".into(),
        Scene::Draw { .. } => "Sorteio".into(),
        _ => String::new(),
    }
}

/// A letra no retorno: maior, alinhada à esquerda e sem o escurecido.
fn monitor_slide(
    ctx: &Ctx,
    slide: &Slide,
    main: Rect,
    opacity: f32,
    with_bg: bool,
    out: &mut Vec<String>,
) {
    let style = &ctx.settings.slide;
    let prose = matches!(slide.kind, SlideKind::Verse | SlideKind::Text);
    let font_id = if slide.kind == SlideKind::Verse && !style.bible_font.is_empty() {
        &style.bible_font
    } else {
        &style.font
    };
    let mut blocks = Vec::new();
    if let Some(aux) = slide.aux.as_deref().filter(|a| !a.is_empty()) {
        blocks.push(
            Block::new(
                aux,
                ctx.fonts.get(font_id, 600),
                main.cqh(5.0),
                1.2,
                0.0,
                main.w,
            )
            .color("#ffd36b"),
        );
    }
    if !slide.text.is_empty() {
        let (cqw, cqh) = fit_text(
            &slide.text,
            slide.kind,
            1.3 * ctx.settings.monitor.text_scale,
        );
        let px = px_size(main, cqw, cqh);
        let text = if prose {
            slide.text.clone()
        } else {
            slide.text.to_uppercase()
        };
        let mut block = Block::new(
            &text,
            ctx.fonts.get(font_id, 600),
            px,
            if prose { 1.25 } else { 1.12 },
            0.0,
            main.w,
        );
        block.shrink_to(&text, main.w, main.h * 0.95);
        blocks.push(block);
    }
    stack(
        &blocks,
        main.cqh(2.0),
        main,
        4,
        opacity,
        if with_bg { 1.0 } else { 0.0 },
        out,
    );
}

/// Um mostrador do cabeçalho ("AGORA 19:42"), alinhado à direita em `right`.
/// Devolve onde ele começa, para o próximo ficar à esquerda.
#[allow(clippy::too_many_arguments)]
fn clock_badge(
    ctx: &Ctx,
    ui: &str,
    label: &str,
    value: &str,
    color: &str,
    background: Option<(&str, f32)>,
    right: f32,
    mid: f32,
    out: &mut Vec<String>,
) -> f32 {
    let s = ctx.screen;
    let value_font = ctx.fonts.get(ui, 700);
    let label_font = ctx.fonts.get(ui, 700);
    let (value_px, label_px) = (s.cqh(6.0), s.cqh(1.9));
    let pad = if background.is_some() {
        s.cqw(1.4)
    } else {
        0.0
    };
    let w = value_font
        .width(value, value_px, 0.0)
        .max(label_font.width(label, label_px, 0.12))
        + pad * 2.0;
    let cx = right - w / 2.0;
    let h = label_px + s.cqh(0.6) + value_px + s.cqh(1.6);
    if let Some((bg, opacity)) = background {
        out.push(ass::rect(
            right - w,
            mid - h / 2.0,
            w,
            h,
            bg,
            opacity,
            s.cqh(1.2),
        ));
    }
    let top = mid - (label_px + s.cqh(0.6) + value_px) / 2.0;
    out.push(
        Line {
            text: label,
            font: &label_font,
            px: label_px,
            x: cx,
            y: top + label_px / 2.0,
            align: 5,
            color,
            opacity: 0.85,
            spacing: 0.12,
            shadow: 0.0,
        }
        .event(),
    );
    out.push(
        Line {
            text: value,
            font: &value_font,
            px: value_px,
            x: cx,
            y: top + label_px + s.cqh(0.6) + value_px / 2.0,
            align: 5,
            color,
            opacity: 1.0,
            spacing: 0.0,
            shadow: 0.0,
        }
        .event(),
    );
    right - w
}

fn footer(ctx: &Ctx, ui: &str, next: &str, top: f32, h: f32, out: &mut Vec<String>) {
    let s = ctx.screen;
    out.push(ass::rect(s.x, top, s.w, h, "#111111", 1.0, 0.0));
    out.push(ass::rect(s.x, top, s.w, 1.0, "#ffffff", 0.12, 0.0));
    let mid = top + h / 2.0;
    let tag_font = ctx.fonts.get(ui, 700);
    let tag_px = s.cqh(2.6);
    let tag = "A SEGUIR";
    let tag_w = tag_font.width(tag, tag_px, 0.05) + s.cqw(2.0);
    let tag_h = tag_px * 1.2 + s.cqh(0.8);
    let left = s.x + s.cqw(2.5);
    let empty = next == "Fim" || next == "—";
    out.push(ass::rect(
        left,
        mid - tag_h / 2.0,
        tag_w,
        tag_h,
        if empty { "#ffffff" } else { "#8fd18f" },
        if empty { 0.3 } else { 1.0 },
        s.cqh(0.6),
    ));
    out.push(
        Line {
            text: tag,
            font: &tag_font,
            px: tag_px,
            x: left + tag_w / 2.0,
            y: mid,
            align: 5,
            color: "#000000",
            opacity: 1.0,
            spacing: 0.05,
            shadow: 0.0,
        }
        .event(),
    );
    let next_font = ctx.fonts.get(ui, 500);
    let next_px = s.cqh(4.2);
    let x = left + tag_w + s.cqw(2.0);
    let text = truncate(next, &next_font, next_px, s.x + s.w - s.cqw(2.5) - x);
    out.push(
        Line {
            text: &text,
            font: &next_font,
            px: next_px,
            x,
            y: mid,
            align: 4,
            color: "#ffffff",
            opacity: 0.75,
            spacing: 0.0,
            shadow: 0.0,
        }
        .event(),
    );
}

/// Corta com reticências para caber em `max` pixels.
fn truncate(text: &str, font: &Font, px: f32, max: f32) -> String {
    if font.width(text, px, 0.0) <= max {
        return text.to_owned();
    }
    let chars: Vec<char> = text.chars().collect();
    let (mut lo, mut hi) = (0, chars.len());
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        let candidate: String = chars[..mid].iter().collect::<String>() + "…";
        if font.width(&candidate, px, 0.0) <= max {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    chars[..lo].iter().collect::<String>().trim_end().to_owned() + "…"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stage::{Pulpit, Timer};

    fn fonts() -> Fonts {
        Fonts::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fonts"))
    }

    fn ctx<'a>(fonts: &'a Fonts, settings: &'a Settings) -> Ctx<'a> {
        Ctx {
            fonts,
            settings,
            user_media: Path::new("/tmp/7h-user"),
            now: 1_000_000.0,
            clock: "19:42".into(),
            screen: Rect {
                x: 0.0,
                y: 0.0,
                w: 1920.0,
                h: 1080.0,
            },
            image_size: &|_| Some((1600, 900)),
        }
    }

    fn lyric(text: &str) -> Slide {
        Slide {
            kind: SlideKind::Lyric,
            text: text.into(),
            aux: None,
            image: None,
            image_position: 5,
            repeat: false,
            time_ms: None,
            instrumental_time_ms: None,
        }
    }

    fn slide_state(slide: Slide) -> StageState {
        StageState {
            scene: Scene::Slide {
                slide,
                title: "Santo".into(),
                next: None,
            },
            ..Default::default()
        }
    }

    #[test]
    fn fit_text_matches_the_css() {
        let (cqw, cqh) = fit_text(
            "Santo! Santo! Santo!\nDeus onipotente!",
            SlideKind::Lyric,
            1.0,
        );
        assert!((cqh - 12.0).abs() < 0.01);
        assert!((cqw - 9.0).abs() < 0.01);
    }

    #[test]
    fn lyric_is_uppercase_centered_one_event_per_line() {
        let (fonts, settings) = (fonts(), Settings::default());
        let f = frame(
            &ctx(&fonts, &settings),
            &slide_state(lyric("Santo! Santo! Santo!\nDeus onipotente!")),
            Role::Audience,
            1.0,
        );
        assert_eq!(f.events.len(), 2);
        assert!(f.events[0].contains("SANTO! SANTO! SANTO!"));
        assert!(f.events[0].contains("\\an5\\pos(960.0,"));
        assert!(matches!(
            f.background,
            Background::Composed(Compose {
                shade: Shade::Radial(_),
                ..
            })
        ));
        assert_eq!(f.tick, None);
    }

    #[test]
    fn long_lines_wrap_inside_the_screen() {
        let (fonts, mut settings) = (fonts(), Settings::default());
        // Letra grande: a linha comprida precisa quebrar para caber.
        settings.slide.text_scale = 1.6;
        let long = "Uma linha muito comprida que não cabe de jeito nenhum numa tela só de projeção";
        let f = frame(
            &ctx(&fonts, &settings),
            &slide_state(lyric(long)),
            Role::Audience,
            1.0,
        );
        assert!(f.events.len() >= 2, "{:?}", f.events);
        let font = fonts.get("condensed", 600);
        for e in &f.events {
            let text = e.rsplit('}').next().unwrap();
            let fs: f32 = e
                .split("\\fs")
                .nth(1)
                .unwrap()
                .split('\\')
                .next()
                .unwrap()
                .parse()
                .unwrap();
            let px = fs / font.ass_size(1.0);
            assert!(font.width(text, px, 0.01) <= 1920.0 * 0.9 + 1.0, "{text}");
        }
    }

    #[test]
    fn countdown_and_draw_follow_the_web_version() {
        assert_eq!(countdown_text(65_000.0, 0.0), "01:05");
        assert_eq!(countdown_text(3_725_000.0, 0.0), "1:02:05");
        assert_eq!(countdown_text(0.0, 10.0), "00:00");
        let c: Vec<String> = ["1", "2", "3"].map(String::from).to_vec();
        assert_eq!(draw_frame("2", &c, 0.0, 1000, 2000.0), ("2".into(), true));
        let (mid, done) = draw_frame("2", &c, 0.0, 1000, 500.0);
        assert!(!done && mid != "2");
    }

    #[test]
    fn camera_fills_the_sides_when_zoomed() {
        let area = Rect {
            x: 0.0,
            y: 0.0,
            w: 1600.0,
            h: 900.0,
        };
        let (fit, _, _) = camera(None, area, 0.75);
        assert!((fit.h - 900.0).abs() < 0.1 && (fit.x - 462.5).abs() < 0.1);
        let (zoomed, x, _) = camera(Some((0.0, 0.5, 3.0)), area, 0.75);
        // Não deixa a borda entrar: o canto esquerdo da página fica na tela.
        assert!(zoomed.x <= 0.01 && x > 0.0);
    }

    #[test]
    fn monitor_shows_clock_timer_message_and_next() {
        let (fonts, mut settings) = (fonts(), Settings::default());
        settings.monitor.show_next = true;
        let mut state = slide_state(lyric("Castelo forte"));
        if let Scene::Slide { next, .. } = &mut state.scene {
            *next = Some(Box::new(lyric("É nosso Deus")));
        }
        state.pulpit = Pulpit {
            message: Some("Faltam 5 minutos".into()),
            timer: Some(Timer {
                elapsed_ms: 0.0,
                started_at: None,
                limit_ms: Some(60_000.0),
            }),
        };
        let f = frame(&ctx(&fonts, &settings), &state, Role::Monitor, 1.0);
        let all = f.events.join("\n");
        for needle in [
            "19:42",
            "AGORA",
            "FALTAM",
            "01:00",
            "Faltam 5 minutos",
            "A SEGUIR",
            "É nosso Deus",
            "CASTELO FORTE",
        ] {
            assert!(all.contains(needle), "faltou {needle}:\n{all}");
        }
        assert!(f.tick.is_some());
    }

    #[test]
    fn notice_and_idle_use_the_screen_style() {
        let (fonts, settings) = (fonts(), Settings::default());
        let notice_slide = Slide {
            kind: SlideKind::Text,
            aux: Some("Santa Ceia".into()),
            ..lyric("Vamos celebrar")
        };
        let f = frame(
            &ctx(&fonts, &settings),
            &slide_state(notice_slide),
            Role::Audience,
            1.0,
        );
        assert!(f.events.iter().any(|e| e.contains("SANTA CEIA")));
        assert!(f.events.iter().any(|e| e.contains("Vamos celebrar")));
        let idle = StageState {
            scene: Scene::Idle,
            ..Default::default()
        };
        let f = frame(&ctx(&fonts, &settings), &idle, Role::Audience, 1.0);
        assert!(matches!(f.background, Background::Composed(_)));
    }

    #[test]
    fn page_with_zoom_and_pen() {
        let (fonts, settings) = (fonts(), Settings::default());
        let overlay = serde_json::json!({
            "zoom": { "x": 0.3, "y": 0.4, "scale": 2.0 },
            "strokes": [{ "tool": "pen", "color": "#ff0000", "width": 6, "points": [[0.1, 0.1], [0.5, 0.5]] }],
            "spotlight": { "x": 0.5, "y": 0.5, "r": 0.1 },
            "laser": { "x": 0.2, "y": 0.2 }
        });
        let state = StageState {
            scene: Scene::Image {
                src: "/p/1.jpg".into(),
                title: "Sermão".into(),
                page: 1,
                total: 3,
                next: None,
                overlay,
            },
            ..Default::default()
        };
        let f = frame(&ctx(&fonts, &settings), &state, Role::Audience, 1.0);
        let Background::Page(page) = &f.background else {
            panic!("{:?}", f.background)
        };
        assert_eq!(page.zoom, Some((0.3, 0.4, 2.0)));
        assert!(f.events.iter().any(|e| e.contains("\\iclip(")));
        assert!(f.events.len() >= 4);
    }

    #[test]
    fn truncates_with_ellipsis() {
        let fonts = fonts();
        let font = fonts.get("sans", 500);
        let t = truncate("Um texto bem comprido para caber", &font, 40.0, 200.0);
        assert!(
            t.ends_with('…') && font.width(&t, 40.0, 0.0) <= 200.0,
            "{t}"
        );
        assert_eq!(truncate("Oi", &font, 40.0, 200.0), "Oi");
    }
}
