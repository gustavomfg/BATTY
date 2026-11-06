//! A small utility scorer: every possible remark gets a score from what the
//! companion perceives, and only the best one is said, if it is above the
//! talk threshold. No learning and no model, so every decision can be explained
//! and tested.

use super::{senses::Senses, vision::ScreenStats};

/// Below this score the companion stays quiet.
pub const TALK_THRESHOLD: f32 = 0.35;

const CHANGES_MIN_AGE_SECS: u64 = 60 * 60;
const DAY_SECS: f32 = 24.0 * 60.0 * 60.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quiet,
    WarnChanges,
    PeekError,
    LateNight,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Decision {
    pub action: Action,
    pub score: f32,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct Remark {
    action: Action,
    score: f32,
    message: String,
}

pub fn decide(senses: &Senses, screen: Option<&ScreenStats>) -> Decision {
    let remarks = [
        screen.and_then(peek_error),
        senses.changes.as_ref().and_then(|changes| {
            warn_changes(changes.files, &changes.repo, changes.oldest_age_secs)
        }),
        senses.hour.and_then(late_night),
    ];

    remarks
        .into_iter()
        .flatten()
        .filter(|remark| remark.score >= TALK_THRESHOLD)
        .max_by(|a, b| a.score.total_cmp(&b.score))
        .map_or(
            Decision {
                action: Action::Quiet,
                score: 0.1,
                message: None,
            },
            |remark| Decision {
                action: remark.action,
                score: remark.score,
                message: Some(remark.message),
            },
        )
}

fn peek_error(screen: &ScreenStats) -> Option<Remark> {
    (screen.red_fraction >= super::vision::ERROR_RED_LIMIT).then(|| Remark {
        action: Action::PeekError,
        score: 0.9,
        message: "Tem muito vermelho na tela. Pode ser um erro ou um aviso.".to_owned(),
    })
}

fn warn_changes(files: usize, repo: &str, oldest_age_secs: u64) -> Option<Remark> {
    if files == 0 || oldest_age_secs < CHANGES_MIN_AGE_SECS {
        return None;
    }
    // Grows from 0.4 to 0.8 over one day, so older work is nudged harder.
    let age_factor = (oldest_age_secs as f32 / DAY_SECS).min(1.0);
    let noun = if files == 1 {
        "arquivo alterado"
    } else {
        "arquivos alterados"
    };
    Some(Remark {
        action: Action::WarnChanges,
        score: 0.4 + 0.4 * age_factor,
        message: format!(
            "{files} {noun} em {repo} há {}, sem commit.",
            human_age(oldest_age_secs)
        ),
    })
}

fn late_night(hour: u8) -> Option<Remark> {
    (hour >= 23 || hour < 5).then(|| Remark {
        action: Action::LateNight,
        score: 0.6,
        message: format!("Já são {hour:02}h. Que tal parar por hoje?"),
    })
}

pub fn human_age(secs: u64) -> String {
    let (value, unit) = if secs >= 24 * 3600 {
        (secs / (24 * 3600), ("dia", "dias"))
    } else if secs >= 3600 {
        (secs / 3600, ("hora", "horas"))
    } else {
        (secs / 60, ("minuto", "minutos"))
    };
    let unit = if value == 1 { unit.0 } else { unit.1 };
    format!("{value} {unit}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::companion::senses::PendingChanges;

    fn senses(changes: Option<(usize, u64)>, hour: Option<u8>) -> Senses {
        Senses {
            changes: changes.map(|(files, oldest_age_secs)| PendingChanges {
                repo: "BatPet".to_owned(),
                files,
                oldest_age_secs,
            }),
            hour,
        }
    }

    fn screen(red_fraction: f32) -> ScreenStats {
        ScreenStats { red_fraction }
    }

    #[test]
    fn nothing_to_say_stays_quiet() {
        let decision = decide(&senses(None, Some(14)), Some(&screen(0.0)));
        assert_eq!(decision.action, Action::Quiet);
        assert_eq!(decision.message, None);
    }

    #[test]
    fn fresh_changes_are_not_worth_mentioning() {
        let decision = decide(&senses(Some((3, 600)), Some(14)), None);
        assert_eq!(decision.action, Action::Quiet);
    }

    #[test]
    fn old_changes_are_mentioned_with_their_age() {
        let decision = decide(&senses(Some((6, 2 * 24 * 3600)), Some(14)), None);
        assert_eq!(decision.action, Action::WarnChanges);
        assert_eq!(
            decision.message.as_deref(),
            Some("6 arquivos alterados em BatPet há 2 dias, sem commit.")
        );
    }

    #[test]
    fn single_file_uses_singular_wording() {
        let decision = decide(&senses(Some((1, 3 * 3600)), None), None);
        assert_eq!(
            decision.message.as_deref(),
            Some("1 arquivo alterado em BatPet há 3 horas, sem commit.")
        );
    }

    #[test]
    fn visible_error_red_beats_the_other_remarks() {
        let decision = decide(
            &senses(Some((4, 5 * 24 * 3600)), Some(23)),
            Some(&screen(0.05)),
        );
        assert_eq!(decision.action, Action::PeekError);
        assert!(decision.message.is_some());
    }

    #[test]
    fn small_red_is_ignored() {
        let decision = decide(&senses(None, Some(14)), Some(&screen(0.001)));
        assert_eq!(decision.action, Action::Quiet);
    }

    #[test]
    fn late_night_gets_a_gentle_remark() {
        let decision = decide(&senses(None, Some(1)), None);
        assert_eq!(decision.action, Action::LateNight);
        assert_eq!(
            decision.message.as_deref(),
            Some("Já são 01h. Que tal parar por hoje?")
        );
        assert_eq!(decide(&senses(None, Some(14)), None).action, Action::Quiet);
    }

    #[test]
    fn only_one_remark_is_said_at_a_time() {
        let decision = decide(&senses(Some((2, 5 * 24 * 3600)), Some(23)), None);
        assert_eq!(decision.action, Action::WarnChanges);
        assert!(decision.message.is_some());
    }

    #[test]
    fn human_age_uses_the_largest_sensible_unit() {
        assert_eq!(human_age(90), "1 minuto");
        assert_eq!(human_age(45 * 60), "45 minutos");
        assert_eq!(human_age(3600), "1 hora");
        assert_eq!(human_age(26 * 3600), "1 dia");
        assert_eq!(human_age(3 * 24 * 3600), "3 dias");
    }
}
