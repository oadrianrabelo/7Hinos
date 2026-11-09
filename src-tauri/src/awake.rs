//! Enquanto alguma tela está projetando, o Windows não apaga o monitor nem
//! suspende o computador (no meio do sermão, a tela do telão apagava com a
//! economia de energia). Sem telas abertas, volta ao normal.

use std::time::Duration;

use tauri::AppHandle;

pub fn keep_screens_on(app: AppHandle) {
    #[cfg(windows)]
    std::thread::spawn(move || {
        use windows_sys::Win32::System::Power::{
            SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
        };
        // O pedido vale enquanto esta thread viver: por isso uma só, fixa.
        let mut on = false;
        loop {
            let projecting = !crate::displays::outputs(&app).is_empty();
            if projecting != on {
                let flags = if projecting {
                    ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED
                } else {
                    ES_CONTINUOUS
                };
                unsafe { SetThreadExecutionState(flags) };
                log::info!(
                    "economia de energia {}",
                    if projecting {
                        "suspensa (projetando)"
                    } else {
                        "de volta ao normal"
                    }
                );
                on = projecting;
            }
            std::thread::sleep(Duration::from_secs(15));
        }
    });
    #[cfg(not(windows))]
    let _ = (app, Duration::ZERO);
}
