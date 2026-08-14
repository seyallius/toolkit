//! module mkv2mp3 - Convert MKV files to MP3 with optional cover art extracted from video.

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

/// Extension for Matroska video files.
const MKV_EXT: &str = "mkv";

/// Extension for MP3 audio files.
const MP3_EXT: &str = "mp3";

/// Default audio bitrate for MP3 encoding (kbps).
const DEFAULT_BITRATE: u32 = 320;

/// Default size (width and height) for extracted cover art.
const DEFAULT_COVER_SIZE: u32 = 600;

/// Prefix for temporary cover image files.
const TEMP_COVER_PREFIX: &str = "toolkit-cover-";

/// Suffix for temporary cover image files.
const TEMP_COVER_SUFFIX: &str = ".jpg";

/// Arguments for the `mkv2mp3` subcommand.
#[derive(Debug, Args)]
pub struct Mkv2mp3Args {
    /// Common batch options like output directory and force overwrite.
    #[command(flatten)]
    pub batch: BatchArgs,

    /// Size (width and height) for the extracted cover art.
    #[arg(long, default_value_t = DEFAULT_COVER_SIZE)]
    pub cover_size: u32,

    /// Audio bitrate in kbps for the output MP3.
    #[arg(long, default_value_t = DEFAULT_BITRATE)]
    pub bitrate: u32,

    /// MKV files; scans the current directory when omitted.
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}

// ----------------------------------------- Public API ----------------------------------------- //

/// Runs the MKV to MP3 conversion for each input file.
pub fn run<R: ProcessRunner>(args_cli: Mkv2mp3Args, ffmpeg: &Ffmpeg<R>) -> Result<()> {
    output::ensure_directory(&args_cli.batch.output_dir)?;
    let inputs = collect(args_cli.files)?;
    let mut succeeded = 0;
    let mut skipped = 0;
    let mut failed = 0;
    for input in inputs {
        let out = output::output_path(&input, &args_cli.batch.output_dir, MP3_EXT)?;
        if output::decision(&out, args_cli.batch.force) == OutputDecision::SkipExisting {
            println!("SKIPPED: {} already exists", out.display());
            skipped += 1;
            continue;
        }
        let cover = temp_path()?;
        let has_cover = ffmpeg
            .run(args::extract_frame(&input, &cover, args_cli.cover_size))
            .is_ok()
            && cover.metadata().map(|m| m.len() > 0).unwrap_or(false);
        if !has_cover {
            eprintln!(
                "WARNING: cover extraction failed for {}; continuing without cover",
                input.display()
            );
        }
        let result = ffmpeg.run(args::encode_mp3(
            &input,
            has_cover.then_some(cover.as_path()),
            &out,
            args_cli.bitrate,
            args_cli.batch.force,
        ));
        let _ = fs::remove_file(&cover);
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

/// Collects input files: either the given list or all MKV files in the current directory.
fn collect(given: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    if given.is_empty() {
        let found = files::discover(&std::env::current_dir()?, MKV_EXT)?;
        if found.is_empty() {
            println!("No MKV files found to process.");
        }
        Ok(found)
    } else {
        Ok(given
            .into_iter()
            .filter(|p| p.is_file() && files::has_extension(p, MKV_EXT))
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
