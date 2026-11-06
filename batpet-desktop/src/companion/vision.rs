//! Rule-based "sight" without a model. The focused monitor is captured by
//! Noctalia, which pipes the PNG to `batty analyze`. That subprocess measures
//! how much of the screen has an error-like red and writes only that number.
//! The image is never saved; it lives in memory for the length of the call.

use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, Instant},
};

/// Fraction of pixels that look like an error red. Tuned so the purple bat,
/// pink wallpapers and small syntax highlights do not count.
pub const ERROR_RED_LIMIT: f32 = 0.003;

const ANALYSIS_TIMEOUT: Duration = Duration::from_secs(5);
const RESULT_FILE: &str = "look.txt";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenStats {
    pub red_fraction: f32,
}

/// Error-like red: strong red channel, little green and blue.
pub fn is_error_red(r: u8, g: u8, b: u8) -> bool {
    r >= 160 && g <= 90 && b <= 90 && i32::from(r) - i32::from(g) >= 100
}

/// `pixels` is RGBA8, four bytes per pixel.
pub fn analyze_rgba(pixels: &[u8]) -> ScreenStats {
    let total = (pixels.len() / 4).max(1);
    let red = pixels
        .chunks_exact(4)
        .filter(|px| is_error_red(px[0], px[1], px[2]))
        .count();
    ScreenStats {
        red_fraction: red as f32 / total as f32,
    }
}

pub fn encode(stats: &ScreenStats) -> String {
    format!("red_fraction={:.6}\n", stats.red_fraction)
}

pub fn decode(text: &str) -> Result<ScreenStats, String> {
    let red_fraction = text
        .lines()
        .find_map(|line| line.strip_prefix("red_fraction="))
        .ok_or("resultado sem red_fraction")?
        .trim()
        .parse::<f32>()
        .map_err(|error| format!("red_fraction inválido: {error}"))?;
    if !red_fraction.is_finite() {
        return Err("red_fraction não finito".into());
    }
    Ok(ScreenStats { red_fraction })
}

/// `batty analyze --out <file>`: invoked by Noctalia with the PNG on stdin.
/// Returns the process exit code.
pub fn run_analyze_command(args: &[String]) -> i32 {
    let Some(out) = args
        .windows(2)
        .find(|pair| pair[0] == "--out")
        .map(|pair| PathBuf::from(&pair[1]))
    else {
        eprintln!("uso: batty analyze --out <arquivo>");
        return 2;
    };

    let mut png = Vec::new();
    if let Err(error) = std::io::stdin().read_to_end(&mut png) {
        eprintln!("batty: não consegui ler a captura: {error}");
        return 1;
    }
    let image = match image::load_from_memory(&png) {
        Ok(image) => image.to_rgba8(),
        Err(error) => {
            eprintln!("batty: captura não é um PNG válido: {error}");
            return 1;
        }
    };
    drop(png);

    let stats = analyze_rgba(image.as_raw());
    match write_atomic(&out, &encode(&stats)) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("batty: não consegui gravar o resultado: {error}");
            1
        }
    }
}

/// Captures the focused monitor and waits for the analysis result.
/// Only the numeric result is kept; the screenshot is never written to disk.
pub fn look() -> Result<ScreenStats, String> {
    let dir = runtime_dir();
    fs::create_dir_all(&dir).map_err(|error| format!("pasta temporária: {error}"))?;
    let result = dir.join(RESULT_FILE);
    let _ = fs::remove_file(&result);

    let exe = std::env::current_exe().map_err(|error| format!("caminho do batty: {error}"))?;
    let exe = path_str(&exe)?;
    let result_path = path_str(&result)?;
    let pipe_command = format!("{exe} analyze --out {result_path}");

    let output = Command::new("noctalia")
        .args([
            "msg",
            "screenshot-fullscreen",
            "--pipe-to-command",
            "--pipe-command",
            &pipe_command,
            "--no-save-to-file",
            "--no-copy-to-clipboard",
            "--no-annotate",
            "--format",
            "png",
        ])
        .output()
        .map_err(|error| format!("noctalia indisponível: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "noctalia recusou a captura: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let deadline = Instant::now() + ANALYSIS_TIMEOUT;
    loop {
        if let Ok(text) = fs::read_to_string(&result) {
            let _ = fs::remove_file(&result);
            return decode(&text);
        }
        if Instant::now() >= deadline {
            return Err("a análise da captura não terminou a tempo".into());
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn runtime_dir() -> PathBuf {
    match std::env::var_os("XDG_RUNTIME_DIR") {
        Some(base) => PathBuf::from(base).join("batty"),
        None => std::env::var_os("HOME")
            .map_or_else(|| PathBuf::from("."), PathBuf::from)
            .join(".cache")
            .join("batty"),
    }
}

/// Noctalia passes the command as one string, so paths must not contain spaces.
fn path_str(path: &Path) -> Result<&str, String> {
    let text = path
        .to_str()
        .ok_or_else(|| format!("caminho não é UTF-8: {}", path.display()))?;
    if text.contains(char::is_whitespace) {
        return Err(format!("caminho com espaço não suportado: {text}"));
    }
    Ok(text)
}

fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    let temp = path.with_extension("tmp");
    fs::write(&temp, text)?;
    fs::rename(&temp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(r: u8, g: u8, b: u8) -> [u8; 4] {
        [r, g, b, 255]
    }

    #[test]
    fn error_red_needs_strong_red_and_little_green_blue() {
        assert!(is_error_red(241, 76, 76));
        assert!(is_error_red(200, 30, 40));
        // Pink wallpaper, purple bat and white text stay out.
        assert!(!is_error_red(240, 150, 200));
        assert!(!is_error_red(120, 60, 180));
        assert!(!is_error_red(255, 255, 255));
    }

    #[test]
    fn red_fraction_counts_only_error_like_pixels() {
        let mut pixels = Vec::new();
        for _ in 0..3 {
            pixels.extend(pixel(241, 76, 76));
        }
        for _ in 0..7 {
            pixels.extend(pixel(20, 20, 30));
        }
        let stats = analyze_rgba(&pixels);
        assert!((stats.red_fraction - 0.3).abs() < 1e-6);
    }

    #[test]
    fn empty_input_has_no_red() {
        assert_eq!(analyze_rgba(&[]).red_fraction, 0.0);
    }

    #[test]
    fn result_file_round_trips() {
        let stats = ScreenStats {
            red_fraction: 0.0125,
        };
        let decoded = decode(&encode(&stats)).expect("valid result");
        assert!((decoded.red_fraction - 0.0125).abs() < 1e-6);
    }

    #[test]
    fn malformed_result_is_rejected() {
        assert!(decode("").is_err());
        assert!(decode("red_fraction=nan\n").is_err());
        assert!(decode("red_fraction=abc\n").is_err());
    }

    #[test]
    fn paths_with_spaces_are_refused_for_the_noctalia_command() {
        assert!(path_str(Path::new("/home/g/projetos/batty")).is_ok());
        assert!(path_str(Path::new("/home/g/meus projetos/batty")).is_err());
    }
}
