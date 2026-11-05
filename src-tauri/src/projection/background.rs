//! Monta a imagem de fundo de uma tela: a foto cortada para cobrir a tela,
//! o escurecido dos slides (ou o degradê dos avisos) e o logo da tela de
//! espera. Fica em disco, pela descrição: o mesmo fundo do mesmo hino
//! nunca é montado duas vezes, nem de um culto para o outro.
//!
//! Assim o mpv só mostra uma imagem pronta, sem efeitos para calcular a cada
//! quadro, o que pesa pouco até num computador fraco.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use image::{imageops, ImageReader, Rgb, RgbImage};

use super::render::{Compose, Shade};

/// O arquivo do fundo montado, criando se ainda não existir.
pub fn compose(c: &Compose, w: u32, h: u32, cache: &Path) -> std::io::Result<PathBuf> {
    let (w, h) = (w.max(16), h.max(16));
    let local = c.image.as_deref().filter(|p| Path::new(p).is_file());
    let mut hasher = DefaultHasher::new();
    format!("{c:?}|{w}x{h}|v1").hash(&mut hasher);
    // Arquivo trocado com o mesmo nome: a data muda e o fundo é refeito.
    for path in local
        .into_iter()
        .chain(c.logo.as_ref().map(|(p, _)| p.as_str()))
    {
        if let Ok(modified) = std::fs::metadata(path).and_then(|m| m.modified()) {
            modified.hash(&mut hasher);
        }
    }
    let out = cache.join(format!("{:016x}.jpg", hasher.finish()));
    if out.is_file() {
        return Ok(out);
    }
    std::fs::create_dir_all(cache)?;

    let photo = local.and_then(|p| cover(p, c.position, w, h));
    // Foto que não abriu: o degradê azul padrão dos slides, nunca preto.
    let fallback = Shade::Radial([(0x1d, 0x45, 0x60), (0x0b, 0x1d, 0x2b), (0x05, 0x0b, 0x11)]);
    let chosen = match (&c.shade, &photo) {
        (Shade::Slide, None) => &fallback,
        (s, _) => s,
    };
    let has_photo = photo.is_some();
    let mut canvas = photo.unwrap_or_else(|| RgbImage::from_pixel(w, h, Rgb([0, 0, 0])));
    shade(&mut canvas, chosen, has_photo);
    if let Some((path, rect)) = &c.logo {
        paste_logo(&mut canvas, path, rect.x, rect.y, rect.w, rect.h);
    }

    let tmp = out.with_extension("tmp");
    let file = std::fs::File::create(&tmp)?;
    let mut encoder =
        image::codecs::jpeg::JpegEncoder::new_with_quality(std::io::BufWriter::new(file), 90);
    encoder
        .encode_image(&canvas)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    std::fs::rename(&tmp, &out)?;
    Ok(out)
}

/// A foto cortada para cobrir a tela (o `background-size: cover`), ancorada
/// na grade 3×3 do LouvorJA.
fn cover(path: &str, position: u8, w: u32, h: u32) -> Option<RgbImage> {
    let img = ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;
    let (iw, ih) = (img.width() as f32, img.height() as f32);
    let scale = (w as f32 / iw).max(h as f32 / ih);
    // Recorta primeiro na foto original e só depois redimensiona: bem menos
    // contas que redimensionar a foto inteira.
    let (cw, ch) = ((w as f32 / scale).min(iw), (h as f32 / scale).min(ih));
    let p = if (1..=9).contains(&position) {
        position - 1
    } else {
        4
    };
    let (fx, fy) = (f32::from(p % 3) / 2.0, f32::from(p / 3) / 2.0);
    let (x, y) = (((iw - cw) * fx) as u32, ((ih - ch) * fy) as u32);
    let crop = img.crop_imm(x, y, cw.max(1.0) as u32, ch.max(1.0) as u32);
    Some(imageops::resize(
        &crop.to_rgb8(),
        w,
        h,
        imageops::FilterType::Triangle,
    ))
}

/// Posição num degradê radial "ellipse at (cx, cy)" até o canto mais
/// distante: 0 no centro, 1 na borda.
fn radial(x: f32, y: f32, cx: f32, cy: f32, rx: f32, ry: f32) -> f32 {
    let (dx, dy) = ((x - cx) / rx, (y - cy) / ry);
    (dx * dx + dy * dy).sqrt()
}

fn shade(canvas: &mut RgbImage, shade: &Shade, has_photo: bool) {
    let (w, h) = (canvas.width() as f32, canvas.height() as f32);
    let sqrt2 = std::f32::consts::SQRT_2;
    match shade {
        Shade::Slide => {
            // radial-gradient(ellipse at center, .35 → .62) + linear(.1 → .25).
            let (cx, cy) = (w / 2.0, h / 2.0);
            let (rx, ry) = (cx * sqrt2, cy * sqrt2);
            for (x, y, px) in canvas.enumerate_pixels_mut() {
                let t = radial(x as f32, y as f32, cx, cy, rx, ry).min(1.0);
                let a = 0.35 + 0.27 * t;
                let l = 0.1 + 0.15 * (y as f32 / h);
                let keep = (1.0 - a) * (1.0 - l);
                for c in px.0.iter_mut() {
                    *c = (f32::from(*c) * keep) as u8;
                }
            }
        }
        Shade::Darken(a) => {
            let keep = 1.0 - a.clamp(0.0, 1.0);
            for px in canvas.pixels_mut() {
                for c in px.0.iter_mut() {
                    *c = (f32::from(*c) * keep) as u8;
                }
            }
        }
        Shade::Solid(rgb) if !has_photo => {
            for px in canvas.pixels_mut() {
                *px = Rgb([rgb.0, rgb.1, rgb.2]);
            }
        }
        Shade::Radial(stops) if !has_photo => {
            // "ellipse at 50% 35%": centro, 70% e borda.
            let (cx, cy) = (w * 0.5, h * 0.35);
            let (rx, ry) = (cx.max(w - cx) * sqrt2, cy.max(h - cy) * sqrt2);
            let lerp = |a: (u8, u8, u8), b: (u8, u8, u8), t: f32| {
                let m = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t) as u8;
                Rgb([m(a.0, b.0), m(a.1, b.1), m(a.2, b.2)])
            };
            for (x, y, px) in canvas.enumerate_pixels_mut() {
                let t = radial(x as f32, y as f32, cx, cy, rx, ry).min(1.0);
                *px = if t < 0.7 {
                    lerp(stops[0], stops[1], t / 0.7)
                } else {
                    lerp(stops[1], stops[2], (t - 0.7) / 0.3)
                };
            }
        }
        Shade::Solid(_) | Shade::Radial(_) => {}
    }
}

fn paste_logo(canvas: &mut RgbImage, path: &str, x: f32, y: f32, w: f32, h: f32) {
    let Some(logo) = ImageReader::open(path)
        .ok()
        .and_then(|r| r.with_guessed_format().ok())
        .and_then(|r| r.decode().ok())
    else {
        return;
    };
    let (w, h) = (w.round().max(1.0) as u32, h.round().max(1.0) as u32);
    let logo = imageops::resize(&logo.to_rgba8(), w, h, imageops::FilterType::Triangle);
    let (ox, oy) = (x.round() as i64, y.round() as i64);
    for (lx, ly, p) in logo.enumerate_pixels() {
        let (cx, cy) = (ox + i64::from(lx), oy + i64::from(ly));
        if cx < 0 || cy < 0 || cx >= i64::from(canvas.width()) || cy >= i64::from(canvas.height()) {
            continue;
        }
        let a = f32::from(p.0[3]) / 255.0;
        if a <= 0.0 {
            continue;
        }
        let dst = canvas.get_pixel_mut(cx as u32, cy as u32);
        for i in 0..3 {
            dst.0[i] = (f32::from(p.0[i]) * a + f32::from(dst.0[i]) * (1.0 - a)) as u8;
        }
    }
}

/// Miniatura em BGRA cru (o formato do `overlay-add` do mpv), guardada.
pub fn thumbnail(path: &str, w: u32, h: u32, cache: &Path) -> std::io::Result<PathBuf> {
    let (w, h) = (w.max(1), h.max(1));
    let mut hasher = DefaultHasher::new();
    format!("{path}|{w}x{h}|thumb1").hash(&mut hasher);
    if let Ok(modified) = std::fs::metadata(path).and_then(|m| m.modified()) {
        modified.hash(&mut hasher);
    }
    let out = cache.join(format!("{:016x}.bgra", hasher.finish()));
    if out.is_file() {
        return Ok(out);
    }
    std::fs::create_dir_all(cache)?;
    let img = ImageReader::open(path)?
        .with_guessed_format()?
        .decode()
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    let small = imageops::resize(&img.to_rgba8(), w, h, imageops::FilterType::Triangle);
    let mut raw = Vec::with_capacity((w * h * 4) as usize);
    for p in small.pixels() {
        // Pré-multiplicado, como o mpv espera.
        let a = u16::from(p.0[3]);
        let m = |c: u8| ((u16::from(c) * a) / 255) as u8;
        raw.extend_from_slice(&[m(p.0[2]), m(p.0[1]), m(p.0[0]), p.0[3]]);
    }
    let tmp = out.with_extension("tmp");
    std::fs::write(&tmp, raw)?;
    std::fs::rename(&tmp, &out)?;
    Ok(out)
}

/// Fundo que só existe na internet (hino ainda não baixado): baixa uma vez
/// para a pasta da projeção e devolve o arquivo. Sem internet, `None` (e o
/// fundo vira o degradê padrão).
///
/// Nunca segura a letra: espera o download no máximo um instante e, se não
/// chegou, a letra entra com o fundo padrão enquanto ele continua baixando.
/// Um endereço que falhou não é tentado de novo por um tempo (num Wi‑Fi sem
/// saída para a internet, cada tentativa demoraria segundos).
pub async fn fetch(url: &str, cache: &Path) -> Option<String> {
    fetch_waiting(url, cache, std::time::Duration::from_millis(800)).await
}

async fn fetch_waiting(url: &str, cache: &Path, wait: std::time::Duration) -> Option<String> {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, OnceLock};
    use std::time::{Duration, Instant};
    use tokio::sync::Notify;

    static IN_FLIGHT: OnceLock<Mutex<HashMap<String, Arc<Notify>>>> = OnceLock::new();
    static FAILED: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();
    const RETRY_AFTER: Duration = Duration::from_secs(600);

    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return None;
    }
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let out = cache
        .join("baixados")
        .join(format!("{:016x}.img", hasher.finish()));
    if out.is_file() {
        return Some(out.to_string_lossy().into_owned());
    }
    let failed = FAILED.get_or_init(Mutex::default);
    if failed
        .lock()
        .ok()?
        .get(url)
        .is_some_and(|t| t.elapsed() < RETRY_AFTER)
    {
        return None;
    }
    let notify = {
        let mut map = IN_FLIGHT.get_or_init(Mutex::default).lock().ok()?;
        match map.get(url) {
            Some(n) => n.clone(),
            None => {
                let n = Arc::new(Notify::new());
                map.insert(url.to_owned(), n.clone());
                let (url, out, done) = (url.to_owned(), out.clone(), n.clone());
                tokio::spawn(async move {
                    let ok = download(&url, &out).await.is_some();
                    if !ok {
                        log::info!("fundo sem internet, usando o padrão: {url}");
                        if let Ok(mut f) = FAILED.get_or_init(Mutex::default).lock() {
                            f.insert(url.clone(), Instant::now());
                        }
                    }
                    if let Ok(mut m) = IN_FLIGHT.get_or_init(Mutex::default).lock() {
                        m.remove(&url);
                    }
                    done.notify_waiters();
                });
                n
            }
        }
    };
    let _ = tokio::time::timeout(wait, notify.notified()).await;
    out.is_file().then(|| out.to_string_lossy().into_owned())
}

async fn download(url: &str, out: &Path) -> Option<()> {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    let client = CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(4))
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .unwrap_or_default()
    });
    let bytes = client
        .get(url)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .bytes()
        .await
        .ok()?;
    std::fs::create_dir_all(out.parent()?).ok()?;
    let tmp = out.with_extension("tmp");
    std::fs::write(&tmp, &bytes).ok()?;
    std::fs::rename(&tmp, out).ok()?;
    Some(())
}

/// Largura e altura de uma imagem, lendo só o cabeçalho.
pub fn image_size(path: &str) -> Option<(u32, u32)> {
    image::image_dimensions(path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projection::render::Rect;

    /// Sem internet (endereço que não responde), a letra não espera.
    #[tokio::test]
    async fn never_holds_the_slide_without_internet() {
        let dir = tempfile::tempdir().unwrap();
        // Porta fechada no próprio computador: falha como sem rede.
        let url = "http://127.0.0.1:9/fundo.jpg";
        let started = std::time::Instant::now();
        assert_eq!(fetch(url, dir.path()).await, None);
        assert!(started.elapsed() < std::time::Duration::from_secs(2));
        // Depois de falhar, nem tenta de novo por um tempo.
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let again = std::time::Instant::now();
        assert_eq!(fetch(url, dir.path()).await, None);
        assert!(again.elapsed() < std::time::Duration::from_millis(50));
        // Wi‑Fi sem saída para a internet: o endereço nem responde. A letra
        // entra em menos de um segundo mesmo assim.
        let started = std::time::Instant::now();
        assert_eq!(
            fetch("http://10.255.255.1/fundo.jpg", dir.path()).await,
            None
        );
        assert!(started.elapsed() < std::time::Duration::from_millis(1500));
    }

    #[test]
    fn composes_once_and_reuses() {
        let dir = tempfile::tempdir().unwrap();
        let photo = dir.path().join("foto.png");
        RgbImage::from_pixel(400, 200, Rgb([200, 200, 200]))
            .save(&photo)
            .unwrap();
        let c = Compose {
            image: Some(photo.to_string_lossy().into_owned()),
            position: 5,
            shade: Shade::Slide,
            logo: None,
        };
        let cache = dir.path().join("cache");
        let a = compose(&c, 320, 180, &cache).unwrap();
        let b = compose(&c, 320, 180, &cache).unwrap();
        assert_eq!(a, b);
        let img = image::open(&a).unwrap().to_rgb8();
        assert_eq!((img.width(), img.height()), (320, 180));
        // Escurecido: mais claro no meio que no canto.
        let center = img.get_pixel(160, 90).0[0];
        let corner = img.get_pixel(2, 178).0[0];
        assert!(center > corner && center < 200, "{center} {corner}");
    }

    #[test]
    fn gradient_and_logo_without_photo() {
        let dir = tempfile::tempdir().unwrap();
        let logo = dir.path().join("logo.png");
        image::RgbaImage::from_pixel(10, 10, image::Rgba([255, 0, 0, 255]))
            .save(&logo)
            .unwrap();
        let c = Compose {
            image: None,
            position: 5,
            shade: Shade::Radial([(0, 0, 200), (0, 0, 100), (0, 0, 0)]),
            logo: Some((
                logo.to_string_lossy().into_owned(),
                Rect {
                    x: 10.0,
                    y: 10.0,
                    w: 20.0,
                    h: 20.0,
                },
            )),
        };
        let out = compose(&c, 200, 100, dir.path()).unwrap();
        let img = image::open(out).unwrap().to_rgb8();
        assert!(img.get_pixel(20, 20).0[0] > 200, "logo vermelho");
        assert!(
            img.get_pixel(100, 35).0[2] > img.get_pixel(199, 99).0[2],
            "centro mais claro"
        );
        assert_eq!(image_size(&logo.to_string_lossy()), Some((10, 10)));
    }
}
