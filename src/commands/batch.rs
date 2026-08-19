//! module batch - Generic batch processing pipeline for media conversion commands.

use crate::{
    components::banner,
    ffmpeg::{Ffmpeg, ProcessRunner},
    util::{
        files,
        output::{self, OutputDecision},
    },
};
use anyhow::{bail, Result};
use console::Style;
use std::{
    io::{self, IsTerminal},
    path::{Path, PathBuf},
};

// --------------------------------- Types, Constants & Variables ------------------------------- //

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
    // Print a beautiful TTY-aware banner at the start of the batch job
    println!(
        "{}",
        banner::render(
            task.file_type_name(),
            Some("Batch Processing"),
            console::colors_enabled()
        )
    );

    output::ensure_directory(output_dir)?;
    let inputs = collect_inputs(
        explicit_files,
        task.input_extension(),
        task.file_type_name(),
    )?;

    let is_tty = io::stdout().is_terminal();

    let (success_tag, skipped_tag, failed_tag) = if is_tty {
        (
            Style::new()
                .green()
                .bold()
                .apply_to("✔ SUCCESS")
                .to_string(),
            Style::new()
                .yellow()
                .bold()
                .apply_to("⏭ SKIPPED")
                .to_string(),
            Style::new().red().bold().apply_to("✖ FAILED").to_string(),
        )
    } else {
        // Log-friendly format for CI/CD and file redirection
        (
            "[SUCCESS]".to_string(),
            "[SKIPPED]".to_string(),
            "[FAILED]".to_string(),
        )
    };

    let mut succeeded = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for input in inputs {
        let out = output::output_path(&input, output_dir, task.output_extension())?;

        if output::decision(&out, force) == OutputDecision::SkipExisting {
            println!("{skipped_tag}: {} already exists", out.display());
            skipped += 1;
            continue;
        }

        match task.process_file(&input, &out, ffmpeg) {
            Ok(FileOutcome::Success) => {
                println!("{success_tag}: {}", out.display());
                succeeded += 1;
            }
            Ok(FileOutcome::Skipped(reason)) => {
                println!("{skipped_tag}: {reason}");
                skipped += 1;
            }
            Err(error) => {
                eprintln!("{failed_tag}: {}: {error}", input.display());
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
