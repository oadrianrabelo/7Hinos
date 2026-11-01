//! Eventos ASS: o formato de legenda que o mpv desenha por cima da imagem
//! (`osd-overlay`). Cada linha de texto e cada caixa vira um evento com
//! posição absoluta numa tela de `res_x` × `res_y`.

use std::fmt::Write;

use super::text::Font;

/// "#rrggbb" → "&HBBGGRR&" (o ASS guarda as cores ao contrário).
pub fn color(hex: &str) -> String {
    let (r, g, b) = rgb(hex).unwrap_or((255, 255, 255));
    format!("&H{b:02X}{g:02X}{r:02X}&")
}

pub fn rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let h = hex.strip_prefix('#')?;
    if h.len() != 6 || !h.is_ascii() {
        return None;
    }
    let p = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok();
    Some((p(0)?, p(2)?, p(4)?))
}

/// Opacidade (0 a 1) → alfa do ASS (00 = opaco, FF = invisível).
pub fn alpha(opacity: f32) -> String {
    let a = (255.0 * (1.0 - opacity.clamp(0.0, 1.0))).round() as u8;
    format!("&H{a:02X}&")
}

/// Texto puro dentro de um evento: sem chaves (abririam um bloco de
/// comandos) e sem barras invertidas (comandos como `\N`).
pub fn escape(text: &str) -> String {
    text.replace('\\', "\u{29F5}")
        .replace('{', "(")
        .replace('}', ")")
}

/// Uma linha de texto.
pub struct Line<'a> {
    pub text: &'a str,
    pub font: &'a Font,
    /// Tamanho da letra em pixels (o font-size do CSS).
    pub px: f32,
    pub x: f32,
    pub y: f32,
    /// Alinhamento do teclado numérico: 5 = centro, 4 = meio-esquerda…
    pub align: u8,
    pub color: &'a str,
    pub opacity: f32,
    /// Espaço entre as letras, em em (letter-spacing).
    pub spacing: f32,
    /// Sombra suave atrás do texto (a do telão), de 0 a 1.
    pub shadow: f32,
}

impl Line<'_> {
    pub fn event(&self) -> String {
        let fs = self.font.ass_size(self.px);
        let mut out = format!(
            "{{\\an{}\\pos({:.1},{:.1})\\q2\\fn{}\\b{}\\fs{:.1}\\fsp{:.2}\\c{}\\1a{}",
            self.align,
            self.x,
            self.y,
            self.font.family,
            self.font.weight,
            fs,
            self.spacing * self.px,
            color(self.color),
            alpha(self.opacity),
        );
        if self.shadow > 0.0 {
            // Contorno escuro desfocado = o brilho do text-shadow; com
            // contorno, o desfoque não pega a letra, que fica nítida.
            let glow = (self.px * 0.06).max(1.0);
            let _ = write!(
                out,
                "\\bord{:.1}\\3c&H000000&\\3a{}\\blur{:.1}\\shad{:.1}\\4c&H000000&\\4a{}",
                glow,
                alpha(0.45 * self.shadow * self.opacity),
                glow * 2.0,
                self.px * 0.04,
                alpha(0.5 * self.shadow * self.opacity),
            );
        } else {
            out.push_str("\\bord0\\shad0");
        }
        out.push('}');
        out.push_str(&escape(self.text));
        out
    }
}

/// Retângulo cheio (fundo do cronômetro, faixa do recado, rodapé).
pub fn rect(x: f32, y: f32, w: f32, h: f32, fill: &str, opacity: f32, radius: f32) -> String {
    let r = radius.min(w / 2.0).min(h / 2.0).max(0.0);
    let path = if r < 0.5 {
        format!("m 0 0 l {w:.1} 0 {w:.1} {h:.1} 0 {h:.1}")
    } else {
        // Cantos arredondados com curvas de Bézier.
        let k = r * 0.45;
        format!(
            "m {r:.1} 0 l {a:.1} 0 b {b:.1} 0 {w:.1} {k:.1} {w:.1} {r:.1} l {w:.1} {c:.1} b {w:.1} {d:.1} {b:.1} {h:.1} {a:.1} {h:.1} l {r:.1} {h:.1} b {k:.1} {h:.1} 0 {d:.1} 0 {c:.1} l 0 {r:.1} b 0 {k:.1} {k:.1} 0 {r:.1} 0",
            a = w - r,
            b = w - k,
            c = h - r,
            d = h - k,
        )
    };
    format!(
        "{{\\an7\\pos({x:.1},{y:.1})\\bord0\\shad0\\c{}\\1a{}\\p1}}{path}{{\\p0}}",
        color(fill),
        alpha(opacity)
    )
}

/// Desenho livre (caminho ASS em pixels), para a caneta e o foco.
pub fn shape(path: &str, fill: &str, opacity: f32, blur: f32) -> String {
    format!(
        "{{\\an7\\pos(0,0)\\bord0\\shad0\\blur{blur:.1}\\c{}\\1a{}\\p1}}{path}{{\\p0}}",
        color(fill),
        alpha(opacity)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_are_reversed() {
        assert_eq!(color("#ffc94a"), "&H4AC9FF&");
        assert_eq!(color("estranho"), "&HFFFFFF&");
        assert_eq!(alpha(1.0), "&H00&");
        assert_eq!(alpha(0.0), "&HFF&");
    }

    #[test]
    fn text_cannot_inject_commands() {
        assert_eq!(escape("{\\b1}Oi\\N"), "(\u{29F5}b1)Oi\u{29F5}N");
    }

    #[test]
    fn rect_is_a_drawing() {
        let r = rect(10.0, 20.0, 100.0, 50.0, "#000000", 0.5, 0.0);
        assert!(r.starts_with("{\\an7\\pos(10.0,20.0)"));
        assert!(r.contains("\\p1}m 0 0 l 100.0 0"));
        assert!(rect(0.0, 0.0, 100.0, 50.0, "#000000", 1.0, 10.0).contains(" b "));
    }
}
