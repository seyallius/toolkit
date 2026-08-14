//! module ts2mp4 - Convert TS files to MP4 via stream copy.

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
use std::path::PathBuf;

// --------------------------------- Types, Constants & Variables ------------------------------- //

/// Extension for transport stream files.
const TS_EXT: &str = "ts";

/// Extension for MP4 files.
const MP4_EXT: &str = "mp4";

/// Arguments for the `ts2mp4` subcommand.
#[derive(Debug, Args)]
pub struct Ts2mp4Args {
    /// Common batch options like output directory and force overwrite.
    #[command(flatten)]
    pub batch: BatchArgs,

    /// TS files; scans the current directory when omitted.
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}

// ----------------------------------------- Public API ----------------------------------------- //

/// Runs the TS to MP4 conversion for each input file.
pub fn run<R: ProcessRunner>(args_cli: Ts2mp4Args, ffmpeg: &Ffmpeg<R>) -> Result<()> {
    output::ensure_directory(&args_cli.batch.output_dir)?;
    let inputs = inputs(args_cli.files)?;
    let mut succeeded = 0;
    let mut skipped = 0;
    let mut failed = 0;
    for input in inputs {
        let out = output::output_path(&input, &args_cli.batch.output_dir, MP4_EXT)?;
        if output::decision(&out, args_cli.batch.force) == OutputDecision::SkipExisting {
            println!("SKIPPED: {} already exists", out.display());
            skipped += 1;
            continue;
        }
        match ffmpeg.run(args::remux_copy(&input, &out, args_cli.batch.force)) {
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

/// Collects input files: either the given list or all TS files in the current directory.
fn inputs(given: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    if given.is_empty() {
        let files = files::discover(&std::env::current_dir()?, TS_EXT)?;
        if files.is_empty() {
            println!("No TS files found to process.");
        }
        Ok(files)
    } else {
        Ok(given
            .into_iter()
            .filter(|p| {
                if !p.is_file() || !files::has_extension(p, TS_EXT) {
                    eprintln!("WARNING: skipping invalid TS input: {}", p.display());
                    false
                } else {
                    true
                }
            })
            .collect())
    }
}
