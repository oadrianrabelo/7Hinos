//! A janela do mpv no Windows: põe no monitor certo, tira da barra de
//! tarefas e usa o ícone do 7Hinos.
//!
//! O mpv entende a posição de `--geometry` relativa ao monitor principal, e o
//! nome das telas de outro jeito que o sistema; com monitores iguais, o
//! telão e o retorno acabavam os dois no principal. Então o próprio 7Hinos
//! acha a janela do processo e a coloca nas coordenadas do monitor (as
//! mesmas que ele usa para listar as telas).

/// Espera a janela do processo `pid` aparecer e a ajeita. `rect` = monitor
/// (posição e tamanho em pixels); `None` = janela comum (só o ícone).
/// Devolve se achou a janela.
#[cfg(windows)]
pub fn place(pid: u32, rect: Option<(i32, i32, u32, u32)>) -> bool {
    use std::time::{Duration, Instant};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows_sys::Win32::UI::Shell::ExtractIconExW;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    struct Search {
        pid: u32,
        found: HWND,
    }

    unsafe extern "system" fn each(hwnd: HWND, lparam: LPARAM) -> i32 {
        let search = &mut *(lparam as *mut Search);
        let mut owner = 0u32;
        GetWindowThreadProcessId(hwnd, &mut owner);
        if owner == search.pid && IsWindowVisible(hwnd) != 0 && GetWindow(hwnd, GW_OWNER).is_null()
        {
            search.found = hwnd;
            return 0;
        }
        1
    }

    let started = Instant::now();
    let hwnd = loop {
        let mut search = Search {
            pid,
            found: std::ptr::null_mut(),
        };
        unsafe { EnumWindows(Some(each), &mut search as *mut Search as LPARAM) };
        if !search.found.is_null() {
            break search.found;
        }
        if started.elapsed() > Duration::from_secs(8) {
            return false;
        }
        std::thread::sleep(Duration::from_millis(50));
    };

    unsafe {
        // O ícone do 7Hinos no lugar do roxo do mpv.
        if let Ok(exe) = std::env::current_exe() {
            let wide: Vec<u16> = exe.as_os_str().encode_wide_null();
            let (mut large, mut small) = (std::ptr::null_mut(), std::ptr::null_mut());
            if ExtractIconExW(wide.as_ptr(), 0, &mut large, &mut small, 1) > 0 {
                SendMessageW(hwnd, WM_SETICON, ICON_BIG as WPARAM, large as LPARAM);
                SendMessageW(hwnd, WM_SETICON, ICON_SMALL as WPARAM, small as LPARAM);
            }
        }
        let Some((x, y, w, h)) = rect else {
            return true;
        };
        // Fora da barra de tarefas: a projeção não é um programa à parte.
        ShowWindow(hwnd, SW_HIDE);
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(
            hwnd,
            GWL_EXSTYLE,
            (style | WS_EX_TOOLWINDOW as isize) & !(WS_EX_APPWINDOW as isize),
        );
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            x,
            y,
            w as i32,
            h as i32,
            SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
        );
    }
    true
}

#[cfg(windows)]
trait WideNull {
    fn encode_wide_null(&self) -> Vec<u16>;
}

#[cfg(windows)]
impl WideNull for std::ffi::OsStr {
    fn encode_wide_null(&self) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;
        self.encode_wide().chain(std::iter::once(0)).collect()
    }
}

/// Fora do Windows o mpv acha a tela pelo nome (`--screen-name`).
#[cfg(not(windows))]
pub fn place(_pid: u32, _rect: Option<(i32, i32, u32, u32)>) -> bool {
    false
}
