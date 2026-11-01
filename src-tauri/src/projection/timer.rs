//! Cronômetro do pregador no retorno (igual ao lib/timer.ts).

use crate::stage::Timer;

pub struct View {
    /// "Faltam", "Passou" ou "Tempo".
    pub label: &'static str,
    pub text: String,
    pub limited: bool,
    pub warn: bool,
    pub over: bool,
}

fn elapsed(timer: &Timer, now: f64) -> f64 {
    timer.elapsed_ms + timer.started_at.map_or(0.0, |s| (now - s).max(0.0))
}

/// "12:05", "1:02:03".
pub fn text(ms: f64) -> String {
    let total = (ms.abs() / 1000.0).floor() as u64;
    let (h, m, s) = (total / 3600, (total % 3600) / 60, total % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

pub fn display(timer: &Timer, now: f64) -> View {
    let elapsed = elapsed(timer, now);
    let Some(limit) = timer.limit_ms else {
        return View {
            label: "Tempo",
            text: text(elapsed),
            limited: false,
            warn: false,
            over: false,
        };
    };
    let left = limit - elapsed;
    let over = left < 0.0;
    View {
        label: if over { "Passou" } else { "Faltam" },
        text: text(left),
        limited: true,
        warn: !over && left <= 5.0 * 60_000.0,
        over,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_down_and_past_the_limit() {
        let t = Timer {
            elapsed_ms: 0.0,
            started_at: Some(0.0),
            limit_ms: Some(60_000.0),
        };
        let v = display(&t, 30_000.0);
        assert_eq!(
            (v.label, v.text.as_str(), v.warn),
            ("Faltam", "00:30", true)
        );
        let v = display(&t, 125_000.0);
        assert_eq!(
            (v.label, v.text.as_str(), v.over),
            ("Passou", "01:05", true)
        );
        let free = Timer {
            elapsed_ms: 3_725_000.0,
            started_at: None,
            limit_ms: None,
        };
        assert_eq!(display(&free, 0.0).text, "1:02:05");
    }
}
