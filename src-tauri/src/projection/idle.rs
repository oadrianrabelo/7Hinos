//! A tela de espera pode ir passando: a tela de sempre (logo, texto), os
//! aniversariantes da semana e os avisos marcados. A vez de cada uma sai do
//! relógio, então todas as telas (e a prévia do controle) trocam juntas.
//!
//! Mesma regra de `src/lib/idle.ts`.

use chrono::{Datelike, Days, NaiveDate};

use crate::settings::Settings;

#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    /// A tela de espera de sempre.
    Base,
    Text {
        title: String,
        body: String,
    },
}

pub const BIRTHDAYS_TITLE: &str = "Aniversariantes da semana";

/// Domingo da semana de `today` até o sábado.
fn week(today: NaiveDate) -> Vec<NaiveDate> {
    let sunday = today - Days::new(today.weekday().num_days_from_sunday() as u64);
    (0..7).map(|i| sunday + Days::new(i)).collect()
}

/// "05/11 · Ana Souza", na ordem dos dias. Quem nasceu em 29/02 aparece no
/// dia 28 nos anos sem 29.
pub fn birthdays_of_week(settings: &Settings, today: NaiveDate) -> Vec<String> {
    let mut lines = Vec::new();
    for day in week(today) {
        let leap_fill = day.month() == 2
            && day.day() == 28
            && NaiveDate::from_ymd_opt(day.year(), 2, 29).is_none();
        let mut names: Vec<&str> = settings
            .birthdays
            .iter()
            .filter(|b| !b.name.trim().is_empty())
            .filter(|b| {
                (b.month as u32 == day.month() && b.day as u32 == day.day())
                    || (leap_fill && b.month == 2 && b.day == 29)
            })
            .map(|b| b.name.trim())
            .collect();
        names.sort_by_key(|n| n.to_lowercase());
        for name in names {
            lines.push(format!("{:02}/{:02} · {name}", day.day(), day.month()));
        }
    }
    lines
}

pub fn pages(settings: &Settings, today: NaiveDate) -> Vec<Page> {
    let mut pages = vec![Page::Base];
    if settings.idle.birthdays {
        let lines = birthdays_of_week(settings, today);
        if !lines.is_empty() {
            pages.push(Page::Text {
                title: BIRTHDAYS_TITLE.into(),
                body: lines.join("\n"),
            });
        }
    }
    for notice in settings.notices.iter().filter(|n| n.idle) {
        if notice.title.trim().is_empty() && notice.body.trim().is_empty() {
            continue;
        }
        pages.push(Page::Text {
            title: notice.title.trim().into(),
            body: notice.body.trim().into(),
        });
    }
    pages
}

fn rotate_ms(settings: &Settings) -> f64 {
    let secs = match settings.idle.rotate_secs {
        0 => 12,
        s => s,
    };
    secs as f64 * 1000.0
}

/// A tela da vez em `now` (ms desde 1970) e quantos ms faltam para a próxima
/// (`None` quando só há uma).
pub fn current(settings: &Settings, today: NaiveDate, now: f64) -> (Page, Option<u64>) {
    let mut all = pages(settings, today);
    if all.len() == 1 {
        return (all.remove(0), None);
    }
    let every = rotate_ms(settings);
    let slot = (now / every).floor();
    let index = (slot as u64 % all.len() as u64) as usize;
    let left = ((slot + 1.0) * every - now).max(1.0).ceil() as u64;
    (all.swap_remove(index), Some(left))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{Birthday, Notice};

    fn person(name: &str, day: u8, month: u8) -> Birthday {
        Birthday {
            id: 0,
            name: name.into(),
            day,
            month,
        }
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn birthdays_from_sunday_to_saturday() {
        let s = Settings {
            birthdays: vec![
                person("Zeca", 7, 11),
                person("Ana", 2, 11),
                person("Bia", 7, 11),
                person("Fora", 9, 11),
                person("Antes", 31, 10),
            ],
            ..Default::default()
        };
        // Quarta, 4 de novembro de 2026: semana de 1 a 7.
        assert_eq!(
            birthdays_of_week(&s, date(2026, 11, 4)),
            vec!["02/11 · Ana", "07/11 · Bia", "07/11 · Zeca"]
        );
    }

    #[test]
    fn week_across_the_new_year_and_leap_day() {
        let s = Settings {
            birthdays: vec![person("Réveillon", 1, 1), person("Bissexto", 29, 2)],
            ..Default::default()
        };
        assert_eq!(
            birthdays_of_week(&s, date(2026, 12, 30)),
            vec!["01/01 · Réveillon"]
        );
        assert_eq!(
            birthdays_of_week(&s, date(2027, 3, 1)),
            vec!["28/02 · Bissexto"]
        );
        assert_eq!(
            birthdays_of_week(&s, date(2028, 2, 29)),
            vec!["29/02 · Bissexto"]
        );
    }

    #[test]
    fn rotates_by_the_clock() {
        let mut s = Settings::default();
        s.idle.rotate_secs = 10;
        let today = date(2026, 11, 4);
        assert_eq!(current(&s, today, 0.0), (Page::Base, None));

        s.idle.birthdays = true;
        s.birthdays = vec![person("Ana", 4, 11)];
        s.notices = vec![
            Notice {
                id: 1,
                title: "Santa Ceia".into(),
                body: "Sábado".into(),
                idle: true,
            },
            Notice {
                id: 2,
                title: "Só no botão".into(),
                body: "x".into(),
                idle: false,
            },
        ];
        assert_eq!(pages(&s, today).len(), 3);
        assert_eq!(current(&s, today, 2_500.0), (Page::Base, Some(7_500)));
        assert!(matches!(
            current(&s, today, 10_000.0).0,
            Page::Text { ref title, .. } if title == BIRTHDAYS_TITLE
        ));
        assert!(matches!(
            current(&s, today, 25_000.0).0,
            Page::Text { ref title, .. } if title == "Santa Ceia"
        ));
        assert_eq!(current(&s, today, 30_000.0).0, Page::Base);
    }
}
