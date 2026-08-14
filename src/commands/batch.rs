//! module batch - Generic batch processing pipeline for media conversion commands.
//! Provides a generic batch processing pipeline for media conversion commands.

use crate::{
    ffmpeg::{Ffmpeg, ProcessRunner},
    util::{
        files,
        output::{self, OutputDecision},
    },
};
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

// --------------------------------- Types, Constants & Variables ------------------------------- //

/// Status prefix printed for successfully converted files.
const STATUS_SUCCESS: &str = "SUCCESS";

/// Status prefix printed for skipped files.
const STATUS_SKIPPED: &str = "SKIPPED";

/// Status prefix printed for failed files.
const STATUS_FAILED: &str = "FAILED";

/// Warning prefix printed for invalid inputs.
const WARNING_PREFIX: &str = "WARNING";

/// Summary label for the final tally line.
const SUMMARY_LABEL: &str = "SUMMARY";

/// Outcome of processing a single file in a batch.
pub enum FileOutcome {
    /// The file was successfully processed.
    Success,

    /// The file was intentionally skipped with a provided reason.
    Skipped(String),
}

/// Trait representing a single file conversion task within a batch.
pub trait BatchTask {
    /// The file extension to discover (e.g., "ts", "mkv").
    fn input_extension(&self) -> &str;

    /// The output file extension (e.g., "mp4", "mp3").
    fn output_extension(&self) -> &str;

    /// The human-readable name of the file type for log messages.
    fn file_type_name(&self) -> &str;

    /// Executes the conversion logic for a single file.
    fn process_file<R: ProcessRunner>(
        &self,
        input: &Path,
        output: &Path,
        ffmpeg: &Ffmpeg<R>,
    ) -> Result<FileOutcome>;
}

// ----------------------------------------- Public API ----------------------------------------- //

/// Executes a batch process over a collection of files, handling discovery, skipping, and metrics.
pub fn run_batch<R: ProcessRunner, T: BatchTask>(
    task: &T,
    explicit_files: Vec<PathBuf>,
    output_dir: &Path,
    force: bool,
    ffmpeg: &Ffmpeg<R>,
) -> Result<()> {
    output::ensure_directory(output_dir)?;
    let inputs = collect_inputs(
        explicit_files,
        task.input_extension(),
        task.file_type_name(),
    )?;

    let mut succeeded = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for input in inputs {
        let out = output::output_path(&input, output_dir, task.output_extension())?;

        if output::decision(&out, force) == OutputDecision::SkipExisting {
            println!("{STATUS_SKIPPED}: {} already exists", out.display());
            skipped += 1;
            continue;
        }

        match task.process_file(&input, &out, ffmpeg) {
            Ok(FileOutcome::Success) => {
                println!("{STATUS_SUCCESS}: {}", out.display());
                succeeded += 1;
            }
            Ok(FileOutcome::Skipped(reason)) => {
                println!("{STATUS_SKIPPED}: {reason}");
                skipped += 1;
            }
            Err(error) => {
                eprintln!("{STATUS_FAILED}: {}: {error}", input.display());
                failed += 1;
            }
        }
    }

    println!("{SUMMARY_LABEL}: {succeeded} succeeded, {skipped} skipped, {failed} failed");
    if failed > 0 {
        bail!("one or more conversions failed")
    } else {
        Ok(())
    }
}

// -------------------------------------- Internal Helpers -------------------------------------- //

/// Resolves input files either from explicit arguments or by scanning the current directory.
fn collect_inputs(given: Vec<PathBuf>, extension: &str, type_name: &str) -> Result<Vec<PathBuf>> {
    if given.is_empty() {
        let found = files::discover(&std::env::current_dir()?, extension)?;
        if found.is_empty() {
            println!("No {type_name} files found to process.");
        }
        Ok(found)
    } else {
        Ok(given
            .into_iter()
            .filter(|p| {
                if !p.is_file() || !files::has_extension(p, extension) {
                    eprintln!(
                        "{WARNING_PREFIX}: skipping invalid {type_name} input: {}",
                        p.display()
                    );
                    false
                } else {
                    true
                }
            })
            .collect())
    }
}
