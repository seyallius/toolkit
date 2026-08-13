use anyhow::{bail, Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "bmp", "gif", "webp"];

/// Finds regular files in one directory with a case-insensitive extension.
pub fn discover(directory: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let wanted = extension.trim_start_matches('.');
    let mut paths = Vec::new();
    for entry in
        fs::read_dir(directory).with_context(|| format!("reading {}", directory.display()))?
    {
        let path = entry?.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case(wanted))
        {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

/// Finds a same-basename companion image in the documented priority order.
pub fn companion_image(video: &Path) -> Result<PathBuf> {
    let stem = video.file_stem().context("video has no file stem")?;
    let parent = video.parent().unwrap_or_else(|| Path::new("."));
    for ext in IMAGE_EXTENSIONS {
        let candidate = parent.join(stem).with_extension(ext);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    bail!(
        "no image found for {} (tried: {})",
        video.display(),
        IMAGE_EXTENSIONS
            .iter()
            .map(|e| format!(".{e}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(extension.trim_start_matches('.')))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn discovers_no_subdirectories() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.TS"), "").unwrap();
        fs::create_dir(dir.path().join("nested.ts")).unwrap();
        assert_eq!(discover(dir.path(), "ts").unwrap().len(), 1);
    }

    #[test]
    fn finds_priority_and_special_names() {
        let dir = tempdir().unwrap();
        let video = dir.path().join("a [x].video.mp4");
        fs::write(&video, "").unwrap();
        fs::write(dir.path().join("a [x].video.jpg"), "").unwrap();
        assert!(companion_image(&video)
            .unwrap()
            .ends_with("a [x].video.jpg"));
    }
}
