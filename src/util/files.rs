//! module files - File discovery and companion image lookup.
//! Handles file discovery, companion image lookup, and temporary file creation.

use anyhow::{bail, Context, Result};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
    time::SystemTime,
};

// --------------------------------- Types, Constants & Variables ------------------------------- //

/// Supported image extensions for companion image lookup.
pub const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "bmp", "gif", "webp"];

/// Prefix prepended to each extension in error messages.
const EXTENSION_DISPLAY_PREFIX: &str = ".";

/// Fallback parent directory when a path has no parent component.
const FALLBACK_PARENT_DIR: &str = ".";

// ----------------------------------------- Public API ----------------------------------------- //

/// Finds regular files in one directory with a case-insensitive extension.
///
/// # Arguments
/// * `directory` - The directory to search.
/// * `extension` - The file extension to match (without dot).
///
/// # Returns
/// A sorted vector of matching file paths.
pub fn discover(directory: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let wanted = extension.trim_start_matches(EXTENSION_DISPLAY_PREFIX);
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
///
/// # Arguments
/// * `video` - Path to the video file.
///
/// # Returns
/// The path to the first existing image with the same stem.
///
/// # Errors
/// Returns an error if no image is found.
pub fn companion_image(video: &Path) -> Result<PathBuf> {
    let stem = video.file_stem().context("video has no file stem")?;
    let parent = video
        .parent()
        .unwrap_or_else(|| Path::new(FALLBACK_PARENT_DIR));
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
            .map(|e| format!("{EXTENSION_DISPLAY_PREFIX}{e}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Checks if a path has the given extension (case-insensitive).
pub fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension().is_some_and(|e| {
        e.eq_ignore_ascii_case(extension.trim_start_matches(EXTENSION_DISPLAY_PREFIX))
    })
}

/// Creates a temporary file path with the given prefix and suffix.
///
/// Unlike `tempfile::Builder::keep()`, this does not create an empty file on disk,
/// preventing orphaned 0-byte files if the process crashes before FFmpeg writes to it.
pub fn temp_path(prefix: &str, suffix: &str) -> Result<PathBuf> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("system time before Unix epoch")?
        .as_nanos();

    let pid = process::id();
    let filename = format!("{}{}_{}{}", prefix, pid, now, suffix);

    Ok(env::temp_dir().join(filename))
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
