//! module mp32mp4 - Convert MP3 files to MP4 videos with optional cover art.

use crate::{
    cli::BatchArgs,
    ffmpeg::{args, Ffmpeg, ProcessRunner},
    util::{
        files,
        output::{self, OutputDecision},
    },
};
use anyhow::{bail, Result};
use clap::Args;
use std::{fs, path::PathBuf};
use tempfile::Builder;

// --------------------------------- Types, Constants & Variables ------------------------------- //

/// Extension for MP3 audio files.
const MP3_EXT: &str = "mp3";

/// Extension for MP4 video files.
const MP4_EXT: &str = "mp4";

/// Default audio bitrate for MP3 encoding (kbps).
const DEFAULT_BITRATE: u32 = 320;

/// Prefix for temporary cover image files.
const TEMP_COVER_PREFIX: &str = "toolkit-cover-";

/// Suffix for temporary cover image files.
const TEMP_COVER_SUFFIX: &str = ".jpg";

/// Arguments for the `mp32mp4` subcommand.
#[derive(Debug, Args)]
pub struct Mp32mp4Args {
    /// Common batch options like output directory and force overwrite.
    #[command(flatten)]
    pub batch: BatchArgs,

    /// Audio bitrate in kbps for the MP4's audio stream.
    #[arg(long, default_value_t = DEFAULT_BITRATE)]
    pub bitrate: u32,

    /// Skip files without embedded cover art instead of using a black video.
    #[arg(long)]
    pub no_cover_fallback: bool,

    /// MP3 files; scans the current directory when omitted.
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}

// ----------------------------------------- Public API ----------------------------------------- //

/// Runs the MP3 to MP4 conversion for each input file.
pub fn run<R: ProcessRunner>(args_cli: Mp32mp4Args, ffmpeg: &Ffmpeg<R>) -> Result<()> {
    output::ensure_directory(&args_cli.batch.output_dir)?;
    let inputs = collect(args_cli.files)?;
    let (mut succeeded, mut skipped, mut failed) = (0, 0, 0);
    for input in inputs {
        let out = output::output_path(&input, &args_cli.batch.output_dir, MP4_EXT)?;
        if output::decision(&out, args_cli.batch.force) == OutputDecision::SkipExisting {
            println!("SKIPPED: {} already exists", out.display());
            skipped += 1;
            continue;
        }
        let cover = temp_path()?;
        let has_cover = ffmpeg
            .run(args::extract_embedded_cover(&input, &cover))
            .is_ok()
            && cover.metadata().map(|m| m.len() > 0).unwrap_or(false);
        if !has_cover && args_cli.no_cover_fallback {
            println!("SKIPPED: no cover art in {}", input.display());
            let _ = fs::remove_file(cover);
            skipped += 1;
            continue;
        }
        let result = ffmpeg.run(args::encode_mp4(
            has_cover.then_some(cover.as_path()),
            &input,
            &out,
            args_cli.bitrate,
            args_cli.batch.force,
        ));
        let _ = fs::remove_file(cover);
        match result {
            Ok(()) => {
                println!("SUCCESS: {}", out.display());
                succeeded += 1
            }
            Err(error) => {
                eprintln!("FAILED: {}: {error}", input.display());
                failed += 1
            }
        }
    }
    println!("SUMMARY: {succeeded} succeeded, {skipped} skipped, {failed} failed");
    if failed > 0 {
        bail!("one or more conversions failed")
    } else {
        Ok(())
    }
}

// -------------------------------------- Internal Helpers -------------------------------------- //

/// Collects input files: either the given list or all MP3 files in the current directory.
fn collect(given: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    if given.is_empty() {
        let found = files::discover(&std::env::current_dir()?, MP3_EXT)?;
        if found.is_empty() {
            println!("No MP3 files found to process.");
        }
        Ok(found)
    } else {
        Ok(given
            .into_iter()
            .filter(|p| p.is_file() && files::has_extension(p, MP3_EXT))
            .collect())
    }
}

/// Creates a temporary file path for a cover image.
fn temp_path() -> Result<PathBuf> {
    let (_, path) = Builder::new()
        .prefix(TEMP_COVER_PREFIX)
        .suffix(TEMP_COVER_SUFFIX)
        .tempfile()?
        .keep()?;
    Ok(path)
}
