//! Cheap signals the companion can read without any model or capture:
//! uncommitted changes in the repository where `batty` was started, and the
//! local hour. Nothing here runs in the background.

use std::{
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingChanges {
    pub repo: String,
    pub files: usize,
    /// Age of the oldest changed file, in seconds.
    pub oldest_age_secs: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Senses {
    pub changes: Option<PendingChanges>,
    pub hour: Option<u8>,
}

impl Senses {
    pub fn gather(cwd: &Path, now: SystemTime) -> Self {
        Self {
            changes: pending_changes(cwd, now),
            hour: local_hour(),
        }
    }
}

/// Reads `git status --porcelain` from the repository that contains `cwd`.
/// Returns `None` outside a repository or when the tree is clean.
pub fn pending_changes(cwd: &Path, now: SystemTime) -> Option<PendingChanges> {
    let root = PathBuf::from(git_output(cwd, &["rev-parse", "--show-toplevel"])?.trim_end());
    let status = git_output(cwd, &["status", "--porcelain", "--untracked-files=normal"])?;
    let paths = parse_porcelain(&status);
    if paths.is_empty() {
        return None;
    }

    let oldest = paths
        .iter()
        .filter_map(|path| std::fs::metadata(root.join(path)).ok()?.modified().ok())
        .min();
    let oldest_age_secs = oldest
        .and_then(|time| now.duration_since(time).ok())
        .map_or(0, |age| age.as_secs());

    Some(PendingChanges {
        repo: root
            .file_name()
            .map_or_else(String::new, |name| name.to_string_lossy().into_owned()),
        files: paths.len(),
        oldest_age_secs,
    })
}

/// Extracts file paths from porcelain v1 lines (`XY path`). Renames keep the
/// new path. Paths are relative to the repository root.
pub fn parse_porcelain(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let path = line.get(3..)?.trim();
            if path.is_empty() {
                return None;
            }
            let path = path.rsplit(" -> ").next().unwrap_or(path);
            Some(path.trim_matches('"').to_owned())
        })
        .collect()
}

/// Local hour (0–23) through `date`, so the time zone is respected without
/// adding a date dependency.
pub fn local_hour() -> Option<u8> {
    let output = Command::new("date").arg("+%H").output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u8>()
        .ok()
        .filter(|hour| *hour < 24)
}

fn git_output(cwd: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn porcelain_lines_yield_relative_paths() {
        let output = " M src/main.rs\n?? notes.md\nR  old.rs -> new.rs\nA  \"with space.txt\"\n";
        assert_eq!(
            parse_porcelain(output),
            vec!["src/main.rs", "notes.md", "new.rs", "with space.txt"]
        );
    }

    #[test]
    fn clean_or_empty_status_has_no_paths() {
        assert!(parse_porcelain("").is_empty());
        assert!(parse_porcelain("\n").is_empty());
    }
}
