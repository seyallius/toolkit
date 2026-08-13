use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputDecision {
    Process,
    SkipExisting,
}
/// Computes `output_dir/<input-stem>.<extension>` without string path manipulation.
pub fn output_path(input: &Path, output_dir: &Path, extension: &str) -> Result<PathBuf> {
    let stem = input.file_stem().context("input has no file stem")?;
    Ok(output_dir
        .join(stem)
        .with_extension(extension.trim_start_matches('.')))
}
pub fn decision(output: &Path, force: bool) -> OutputDecision {
    if output.exists() && !force {
        OutputDecision::SkipExisting
    } else {
        OutputDecision::Process
    }
}
pub fn ensure_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("creating output directory {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    #[test]
    fn names_output_and_skips() {
        let dir = tempdir().unwrap();
        let out = output_path(Path::new("a.b.mkv"), dir.path(), "mp3").unwrap();
        assert!(out.ends_with("a.b.mp3"));
        fs::write(&out, "").unwrap();
        assert_eq!(decision(&out, false), OutputDecision::SkipExisting);
        assert_eq!(decision(&out, true), OutputDecision::Process);
    }
}
