//! `batty`, the companion that only exists while it is invoked from a terminal.
//!
//! Nothing runs in the background: no autostart, service or tray. The command
//! reads cheap signals, speaks in the terminal, shows the bat for a few seconds
//! and exits. `--look` also captures the focused monitor and reads a single
//! rule-based number from it; the image is never written to disk.

pub mod brain;
pub mod senses;
pub mod vision;

use std::{path::PathBuf, time::SystemTime};

use bevy::{
    app::AppExit,
    prelude::{MessageWriter, NextState, Res, ResMut, Resource, Time},
};

use crate::pet::BatState;
use brain::{Action, decide};
use senses::Senses;

/// Time on screen when `batty` runs as a companion, without `--stay`.
pub const DEFAULT_STAY_SECONDS: f32 = 8.0;

pub const USAGE: &str = "\
uso: batty [--look] [--stay SEGUNDOS]

  sem opções   fala no terminal o que notou, mostra o morcego e sai sozinho
  --look       olha o monitor em foco (captura pelo Noctalia; nada é gravado)
  --stay N     tempo na tela em segundos; 0 mantém a janela até fechar
  batty analyze --out ARQUIVO   uso interno: recebe um PNG pela entrada padrão
";

#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    pub look: bool,
    /// Explicit `--stay` value. Zero means "until the window is closed".
    pub stay: Option<f32>,
    /// False when a development flag is used, which keeps the old behaviour.
    pub companion: bool,
}

impl Options {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let mut look = false;
        let mut stay = None;
        let mut iter = args.iter().skip(1);
        while let Some(arg) = iter.next() {
            if arg == "--look" {
                look = true;
            } else if arg == "--stay" {
                let value = iter
                    .next()
                    .ok_or("--stay precisa de um número de segundos")?;
                stay = Some(parse_stay(value)?);
            } else if let Some(value) = arg.strip_prefix("--stay=") {
                stay = Some(parse_stay(value)?);
            }
        }

        Ok(Self {
            look,
            stay,
            companion: !args.iter().skip(1).any(|arg| is_development_flag(arg)),
        })
    }

    /// Whether the companion speaks, reacts and checks the environment.
    pub fn briefing(&self) -> bool {
        self.companion || self.look
    }

    pub fn stay_seconds(&self) -> Option<f32> {
        match (self.stay, self.companion) {
            (Some(seconds), _) if seconds > 0.0 => Some(seconds),
            (Some(_), _) => None,
            (None, true) => Some(DEFAULT_STAY_SECONDS),
            (None, false) => None,
        }
    }
}

fn parse_stay(value: &str) -> Result<f32, String> {
    value
        .parse::<f32>()
        .ok()
        .filter(|seconds| seconds.is_finite() && *seconds >= 0.0)
        .ok_or_else(|| format!("--stay inválido: {value}"))
}

fn is_development_flag(arg: &str) -> bool {
    arg.starts_with("--review")
        || matches!(
            arg,
            "--debug" | "--habitat-loop" | "--flight-loop" | "--flight-once"
        )
}

/// What the Bevy app needs from the companion. Built before the window opens,
/// so the terminal remark appears first and the bat is not captured by its own look.
#[derive(Debug, Clone, Resource, PartialEq)]
pub struct CompanionRun {
    pub stay_seconds: Option<f32>,
    pub reaction: bool,
}

impl CompanionRun {
    pub fn prepare(options: &Options) -> Self {
        let mut reaction = false;

        if options.briefing() {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let senses = Senses::gather(&cwd, SystemTime::now());

            let screen = if options.look {
                match vision::look() {
                    Ok(stats) => {
                        // Shows the measurement so the rule threshold can be tuned by eye.
                        println!(
                            "batty (visão): {:.2}% da tela com vermelho de erro.",
                            stats.red_fraction * 100.0
                        );
                        Some(stats)
                    }
                    Err(error) => {
                        println!("batty: não consegui olhar a tela: {error}");
                        None
                    }
                }
            } else {
                None
            };

            let decision = decide(&senses, screen.as_ref());
            match &decision.message {
                Some(message) => println!("batty: {message}"),
                None => println!("batty: nada urgente por agora."),
            }
            reaction = decision.action != Action::Quiet;
        }

        Self {
            stay_seconds: options.stay_seconds(),
            reaction,
        }
    }
}

/// Startup: a remark is accompanied by the existing click reaction, so the
/// bat looks up for a moment before it is put away.
pub fn start_briefing_reaction(run: Res<CompanionRun>, mut next: ResMut<NextState<BatState>>) {
    if run.reaction {
        next.set(BatState::Reacting);
    }
}

pub fn exit_after_stay(time: Res<Time>, run: Res<CompanionRun>, mut exits: MessageWriter<AppExit>) {
    if let Some(stay) = run.stay_seconds
        && time.elapsed_secs() >= stay
    {
        exits.write(AppExit::Success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        std::iter::once("batty")
            .chain(values.iter().copied())
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn plain_command_is_a_companion_that_leaves_by_itself() {
        let options = Options::parse(&args(&[])).expect("valid");
        assert!(options.companion);
        assert!(!options.look);
        assert_eq!(options.stay_seconds(), Some(DEFAULT_STAY_SECONDS));
    }

    #[test]
    fn stay_zero_keeps_the_window_until_it_is_closed() {
        let options = Options::parse(&args(&["--stay", "0"])).expect("valid");
        assert_eq!(options.stay_seconds(), None);
        let options = Options::parse(&args(&["--stay=3.5"])).expect("valid");
        assert_eq!(options.stay_seconds(), Some(3.5));
    }

    #[test]
    fn invalid_stay_is_rejected() {
        assert!(Options::parse(&args(&["--stay"])).is_err());
        assert!(Options::parse(&args(&["--stay", "-1"])).is_err());
        assert!(Options::parse(&args(&["--stay", "muito"])).is_err());
    }

    #[test]
    fn development_flags_keep_the_window_open_until_closed() {
        for flags in [
            &["--debug"][..],
            &["--habitat-loop"][..],
            &["--review-dir", "/tmp/x"][..],
        ] {
            let options = Options::parse(&args(flags)).expect("valid");
            assert!(!options.companion, "{flags:?} should not be companion mode");
            assert_eq!(options.stay_seconds(), None);
        }
    }

    #[test]
    fn look_is_allowed_next_to_a_development_flag() {
        let options = Options::parse(&args(&["--debug", "--look"])).expect("valid");
        assert!(options.look);
        assert!(options.briefing());
    }
}
