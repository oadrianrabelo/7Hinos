//! Servidor HTTP mínimo, só em 127.0.0.1, para a interface tocar arquivos do
//! computador (áudios, vídeos, imagens).
//!
//! Existe porque o WebView do Linux (WebKitGTK) não toca áudio nem vídeo pelo
//! protocolo próprio do Tauri, e o do macOS tem limitações parecidas; por HTTP
//! funciona igual em todo lugar, inclusive com avanço (requisições `Range`).
//!
//! Só responde a quem conhece o token aleatório da sessão, e só entrega
//! arquivos dentro das pastas (ou exatamente os arquivos) liberados.

use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::hash::{BuildHasher, Hasher};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use percent_encoding::percent_decode_str;

#[derive(Default)]
struct Allowed {
    dirs: Vec<PathBuf>,
    files: HashSet<PathBuf>,
}

pub struct MediaServer {
    base: String,
    allowed: Arc<RwLock<Allowed>>,
}

impl MediaServer {
    pub fn start() -> io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let token = random_token();
        let base = format!("http://{}/{token}", listener.local_addr()?);
        let allowed: Arc<RwLock<Allowed>> = Default::default();

        let shared = allowed.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let allowed = shared.clone();
                let token = token.clone();
                std::thread::spawn(move || {
                    let _ = handle(stream, &token, &allowed);
                });
            }
        });
        Ok(Self { base, allowed })
    }

    /// Prefixo das URLs; a interface acrescenta `/` + caminho codificado.
    pub fn base_url(&self) -> &str {
        &self.base
    }

    /// Substitui as pastas liberadas (os arquivos avulsos continuam).
    pub fn set_dirs(&self, dirs: impl IntoIterator<Item = PathBuf>) {
        let dirs = dirs
            .into_iter()
            .filter_map(|d| d.canonicalize().ok())
            .collect();
        self.allowed.write().expect("liberados").dirs = dirs;
    }

    pub fn allow_file(&self, path: &Path) {
        if let Ok(path) = path.canonicalize() {
            self.allowed.write().expect("liberados").files.insert(path);
        }
    }
}

fn random_token() -> String {
    // RandomState é semeado pelo sistema operacional a cada instância.
    (0..2)
        .map(|_| format!("{:016x}", RandomState::new().build_hasher().finish()))
        .collect()
}

fn handle(stream: TcpStream, token: &str, allowed: &RwLock<Allowed>) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut stream = stream;

    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut range = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line.trim().is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.trim().eq_ignore_ascii_case("range") {
                range = Some(value.trim().to_owned());
            }
        }
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("");
    if method != "GET" && method != "HEAD" {
        return respond(&mut stream, "405 Method Not Allowed", &[]);
    }

    let Some(path) = resolve(target, token, allowed) else {
        return respond(&mut stream, "404 Not Found", &[]);
    };
    let mut file = std::fs::File::open(&path)?;
    let size = file.metadata()?.len();
    let content_type = mime(&path);

    let (status, start, end) = match range.as_deref().and_then(|r| parse_range(r, size)) {
        Some((start, end)) => ("206 Partial Content", start, end),
        None if size == 0 => ("200 OK", 0, 0),
        None => ("200 OK", 0, size - 1),
    };
    let length = if size == 0 { 0 } else { end - start + 1 };
    let content_range = format!("bytes {start}-{end}/{size}");
    let length_text = length.to_string();
    let mut headers = vec![
        ("Content-Type", content_type),
        ("Content-Length", length_text.as_str()),
        ("Accept-Ranges", "bytes"),
        ("Cache-Control", "no-cache"),
        ("Access-Control-Allow-Origin", "*"),
    ];
    if status.starts_with("206") {
        headers.push(("Content-Range", content_range.as_str()));
    }
    respond(&mut stream, status, &headers)?;
    if method == "GET" && length > 0 {
        file.seek(SeekFrom::Start(start))?;
        io::copy(&mut file.take(length), &mut stream)?;
    }
    stream.flush()
}

/// `/<token>/<caminho codificado>` → arquivo liberado e existente.
fn resolve(target: &str, token: &str, allowed: &RwLock<Allowed>) -> Option<PathBuf> {
    let rest = target
        .strip_prefix('/')?
        .strip_prefix(token)?
        .strip_prefix('/')?;
    let rest = rest.split(['?', '#']).next()?;
    let decoded = percent_decode_str(rest).decode_utf8().ok()?;
    let path = Path::new(decoded.as_ref()).canonicalize().ok()?;
    if !path.is_file() {
        return None;
    }
    let allowed = allowed.read().ok()?;
    let ok = allowed.files.contains(&path) || allowed.dirs.iter().any(|d| path.starts_with(d));
    ok.then_some(path)
}

fn respond(stream: &mut TcpStream, status: &str, headers: &[(&str, &str)]) -> io::Result<()> {
    let mut head = format!("HTTP/1.1 {status}\r\nConnection: close\r\n");
    for (name, value) in headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    if headers.is_empty() {
        head.push_str("Content-Length: 0\r\n");
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes())
}

/// `bytes=início-fim`, `bytes=início-` ou `bytes=-últimos`.
fn parse_range(value: &str, size: u64) -> Option<(u64, u64)> {
    if size == 0 {
        return None;
    }
    let spec = value.strip_prefix("bytes=")?.split(',').next()?.trim();
    let (start, end) = spec.split_once('-')?;
    let (start, end) = match (start.trim(), end.trim()) {
        ("", last) => {
            let last: u64 = last.parse().ok()?;
            (size.saturating_sub(last), size - 1)
        }
        (start, "") => (start.parse().ok()?, size - 1),
        (start, end) => (start.parse().ok()?, end.parse::<u64>().ok()?.min(size - 1)),
    };
    (start <= end && start < size).then_some((start, end))
}

fn mime(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    match ext.as_str() {
        "mp3" => "audio/mpeg",
        "opus" | "ogg" | "oga" => "audio/ogg",
        "m4a" | "aac" => "audio/mp4",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        "mp4" | "m4v" => "video/mp4",
        "mov" => "video/quicktime",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "ogv" => "video/ogg",
        "avi" => "video/x-msvideo",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "bmp" => "image/bmp",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

    impl MediaServer {
        /// O mesmo formato que `fileSrc` monta na interface.
        fn url_for(&self, path: &Path) -> String {
            let path = path.to_string_lossy();
            format!(
                "{}/{}",
                self.base,
                utf8_percent_encode(&path, NON_ALPHANUMERIC)
            )
        }
    }

    fn get(url: &str, range: Option<&str>) -> (String, Vec<u8>) {
        let rest = url.strip_prefix("http://").unwrap();
        let (host, path) = rest.split_once('/').unwrap();
        let mut stream = TcpStream::connect(host).unwrap();
        let range = range.map(|r| format!("Range: {r}\r\n")).unwrap_or_default();
        write!(
            stream,
            "GET /{path} HTTP/1.1\r\nHost: {host}\r\n{range}\r\n"
        )
        .unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();
        let split = response.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
        let head = String::from_utf8_lossy(&response[..split]).into_owned();
        (head, response[split + 4..].to_vec())
    }

    #[test]
    fn serves_allowed_files_with_ranges_only() {
        let dir = tempfile::tempdir().unwrap();
        let music = dir.path().join("musicas/Álbum 1");
        std::fs::create_dir_all(&music).unwrap();
        let song = music.join("Faixa ?#1.mp3");
        std::fs::write(&song, b"0123456789").unwrap();
        let secret = dir.path().join("segredo.txt");
        std::fs::write(&secret, b"x").unwrap();

        let server = MediaServer::start().unwrap();
        let url = server.url_for(&song);
        assert_eq!(
            get(&url, None).0.lines().next(),
            Some("HTTP/1.1 404 Not Found")
        );

        server.set_dirs([dir.path().join("musicas")]);
        let (head, body) = get(&url, None);
        assert!(head.starts_with("HTTP/1.1 200 OK"));
        assert!(head.contains("Content-Type: audio/mpeg"));
        assert_eq!(body, b"0123456789");

        let (head, body) = get(&url, Some("bytes=2-4"));
        assert!(head.starts_with("HTTP/1.1 206"));
        assert!(head.contains("Content-Range: bytes 2-4/10"));
        assert_eq!(body, b"234");

        assert!(get(&server.url_for(&secret), None).0.contains("404"));
        let wrong_token = url.replace(&server.base[server.base.len() - 8..], "deadbeef");
        assert!(get(&wrong_token, None).0.contains("404"));

        server.allow_file(&secret);
        assert_eq!(get(&server.url_for(&secret), None).1, b"x");
    }

    #[test]
    fn parses_ranges() {
        assert_eq!(parse_range("bytes=0-", 10), Some((0, 9)));
        assert_eq!(parse_range("bytes=5-100", 10), Some((5, 9)));
        assert_eq!(parse_range("bytes=-3", 10), Some((7, 9)));
        assert_eq!(parse_range("bytes=20-", 10), None);
        assert_eq!(parse_range("items=0-1", 10), None);
    }
}
