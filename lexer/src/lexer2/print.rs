use super::Construct;
use colored::Colorize;
use std::fmt::Display;
use tracing::info;

pub fn from_cache<const PASS: bool>(pref: &str, c: Construct, l: usize) {
    print_tab(
        format!("{} {pref} {:?} from Cache", pass_or_fail::<PASS>(), c),
        l,
    );
}

pub fn pass_or_fail<const PASS: bool>() -> impl Display {
    if PASS {
        "✅ Pass"
    } else {
        "❌ Fail"
    }
}

pub fn print_colored(t: impl Display, l: usize) {
    print_tab(colored(t, l), l);
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
