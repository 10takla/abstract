use super::Construct;
use colored::Colorize;
use std::fmt::Display;
use tracing::info;

#[derive(Clone, Debug, Default)]
pub(super) struct Print {
    pub(super) level: usize,
    pub(super) max_fail_level: usize,
}

impl Print {
    pub(super) fn from_cache<const PASS: bool>(&self, pref: &str, c: Construct) {
        print_tab(
            format!("{} {pref} {:?} from Cache", tmp_pass_or_fail::<PASS>(), c),
            self.level,
        );
    }

    pub(super) fn pass_or_fail<const PASS: bool>(&self) {
        self.print_colored(tmp_pass_or_fail::<PASS>());
    }

    pub(super) fn print_colored(&self, t: impl Display) {
        print_tab(colored(t, self.level), self.level);
    }
}

fn tmp_pass_or_fail<const PASS: bool>() -> impl Display {
    if PASS {
        "✅ Pass"
    } else {
        "❌ Fail"
    }
}

fn print_tab(t: impl Display, l: usize) {
    info!("{}{t}", tab(l));
}

fn colored(t: impl Display, l: usize) -> std::string::String {
    let (r, g, b) = hsv_to_rgb(((l * 47) % 360) as f32, 1.0, 1.0);
    format!("{}", t.to_string().truecolor(r, g, b))
}

fn tab(l: usize) -> std::string::String {
    if l == 0 {
        Default::default()
    } else {
        (0..l).map(|i| colored("|  ", l - i - 1)).rev().collect()
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match h as u32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let (r, g, b) = ((r + m) * 255.0, (g + m) * 255.0, (b + m) * 255.0);
    (r as u8, g as u8, b as u8)
}
