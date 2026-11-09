//! "Testar este computador": confere, antes do culto, o que costuma dar
//! problema num computador de igreja (antigo, sem internet) e diz o que fazer.
//! Também junta os registros num `.zip` para mandar a quem ajuda.

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Ok,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub title: String,
    pub level: Level,
    pub detail: String,
}

impl Check {
    pub fn new(title: &str, level: Level, detail: impl Into<String>) -> Self {
        Check {
            title: title.into(),
            level,
            detail: detail.into(),
        }
    }
}

/// Memória e processador, para ligar sozinho o modo leve.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Machine {
    /// Memória total em MB (`None` se o sistema não disse).
    pub ram_mb: Option<u64>,
    pub cores: usize,
    /// Computador fraco: até 4 GB de memória ou até 2 núcleos.
    pub weak: bool,
}

pub fn machine() -> Machine {
    let ram_mb = total_ram_mb();
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
    Machine {
        ram_mb,
        cores,
        weak: is_weak(ram_mb, cores),
    }
}

pub fn is_weak(ram_mb: Option<u64>, cores: usize) -> bool {
    // 4 GB aparecem como um pouco menos (memória da placa de vídeo).
    ram_mb.is_some_and(|mb| mb <= 4300) || cores <= 2
}

#[cfg(windows)]
fn total_ram_mb() -> Option<u64> {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    let ok = unsafe { GlobalMemoryStatusEx(&mut status) } != 0;
    ok.then_some(status.ullTotalPhys / 1_048_576)
}

#[cfg(target_os = "linux")]
fn total_ram_mb() -> Option<u64> {
    let text = std::fs::read_to_string("/proc/meminfo").ok()?;
    let kb: u64 = text
        .lines()
        .find_map(|l| l.strip_prefix("MemTotal:"))?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    Some(kb / 1024)
}

#[cfg(not(any(windows, target_os = "linux")))]
fn total_ram_mb() -> Option<u64> {
    None
}

/// Espaço livre (bytes) no disco de uma pasta.
#[cfg(windows)]
pub fn free_space(path: &Path) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    let mut free = 0u64;
    let ok = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    (ok != 0).then_some(free)
}

#[cfg(target_os = "linux")]
pub fn free_space(path: &Path) -> Option<u64> {
    use std::os::unix::ffi::OsStrExt;
    let c = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    (unsafe { libc::statvfs(c.as_ptr(), &mut st) } == 0)
        .then(|| st.f_bavail as u64 * st.f_frsize as u64)
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn free_space(_path: &Path) -> Option<u64> {
    None
}

pub fn gb(bytes: u64) -> String {
    format!("{:.1} GB", bytes as f64 / 1_073_741_824.0).replace('.', ",")
}

/// O relógio do computador parece certo? Computador antigo com a bateria da
/// placa-mãe fraca volta para 2000-e-pouco a cada vez que é desligado da
/// tomada; aí avisos, aniversários e cópias de segurança se perdem.
pub fn clock_check(now_secs: u64) -> Check {
    // 1º de janeiro de 2026: esta versão não existia antes disso.
    const BUILT_AFTER: u64 = 1_767_225_600;
    if now_secs < BUILT_AFTER {
        Check::new(
            "Data e hora",
            Level::Fail,
            "A data do computador está atrasada (bateria da placa-mãe fraca?). Acerte a data e a hora do Windows: aniversários, cópias de segurança e o relógio da tela dependem dela.",
        )
    } else {
        Check::new(
            "Data e hora",
            Level::Ok,
            "A data do computador parece certa.",
        )
    }
}

/// Grava um `.zip` com os registros do app e do mpv e o resultado do teste.
pub fn save_report(log_dir: &Path, dest_dir: &Path, report: &str) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dest_dir)?;
    let stamp = chrono::Local::now().format("%Y-%m-%d-%H%M");
    let path = dest_dir.join(format!("7Hinos-diagnostico-{stamp}.zip"));
    let mut zip = zip::ZipWriter::new(std::fs::File::create(&path)?);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("teste-do-computador.txt", options)
        .map_err(std::io::Error::other)?;
    zip.write_all(report.as_bytes())?;
    if let Ok(entries) = std::fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            let file = entry.path();
            let Some(name) = file.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !file.is_file() || !(name.ends_with(".log") || name.ends_with(".txt")) {
                continue;
            }
            // Só o fim de registros enormes: o que interessa é o recente.
            let bytes = std::fs::read(&file)?;
            let tail = &bytes[bytes.len().saturating_sub(4_000_000)..];
            zip.start_file(name, options)
                .map_err(std::io::Error::other)?;
            zip.write_all(tail)?;
        }
    }
    zip.finish().map_err(std::io::Error::other)?;
    Ok(path)
}

/// O resultado do teste em texto, para o `.zip` e para colar numa mensagem.
pub fn report_text(version: &str, checks: &[Check]) -> String {
    let mut out = format!("7Hinos {version} — teste do computador\n\n");
    for c in checks {
        let mark = match c.level {
            Level::Ok => "OK   ",
            Level::Warn => "ATENÇÃO",
            Level::Fail => "PROBLEMA",
        };
        out.push_str(&format!("[{mark}] {}: {}\n", c.title, c.detail));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weak_computers() {
        // Pentium G580 com 4 GB (o Windows mostra ~3,9 GB).
        assert!(is_weak(Some(3_980), 2));
        assert!(is_weak(Some(8_000), 2));
        assert!(!is_weak(Some(8_000), 4));
        assert!(!is_weak(None, 4));
        assert!(machine().cores >= 1);
    }

    #[test]
    fn wrong_clock_is_reported() {
        assert_eq!(clock_check(946_684_800).level, Level::Fail); // ano 2000
        assert_eq!(clock_check(1_790_000_000).level, Level::Ok);
    }

    #[test]
    fn report_zip_has_logs_and_result() {
        let dir = tempfile::tempdir().unwrap();
        let logs = dir.path().join("logs");
        std::fs::create_dir_all(&logs).unwrap();
        std::fs::write(logs.join("7hinos.log"), "abriu").unwrap();
        std::fs::write(logs.join("mpv-telao.log"), "vo: gpu").unwrap();
        std::fs::write(logs.join("outro.bin"), "x").unwrap();
        let text = report_text("1.0", &[Check::new("Projeção leve", Level::Ok, "mpv")]);
        assert!(text.contains("[OK   ] Projeção leve: mpv"));
        let zip_path = save_report(&logs, &dir.path().join("pendrive"), &text).unwrap();
        let zip = zip::ZipArchive::new(std::fs::File::open(zip_path).unwrap()).unwrap();
        let mut names: Vec<String> = zip.file_names().map(|n| n.unwrap().into_owned()).collect();
        names.sort();
        assert_eq!(
            names,
            ["7hinos.log", "mpv-telao.log", "teste-do-computador.txt"]
        );
        assert!(free_space(dir.path()).is_some_and(|b| b > 0) || cfg!(not(target_os = "linux")));
    }
}
