//! module batch - Generic interactive batch processing pipeline for media conversion commands.

use crate::{
    cli::{BatchArgs, BatchOnError},
    components::{
        banner,
        prompt::{self, ContinueChoice, SiblingBatchChoice},
    },
    ffmpeg::{Ffmpeg, ProcessRunner},
    util::{
        batch::{BatchPolicy, BatchReport},
        files,
        output::{self, OutputDecision},
    },
};
use anyhow::{bail, Context, Result};
use std::{
    io::{self, BufReader, IsTerminal},
    path::{Path, PathBuf},
};

// --------------------------------- Types, Constants & Variables ------------------------------- //

/// Warning prefix printed for invalid inputs.
const WARNING_PREFIX: &str = "WARNING";

/// Outcome of processing a single file in a batch.
#[derive(Debug, Clone)]
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

    /// Optional stem suffix to exclude from batch discovery (e.g., "_with_image").
    fn exclude_stem_suffix(&self) -> Option<&str> {
        None
    }

    /// Executes the conversion logic for a single file.
    fn process_file<R: ProcessRunner>(
        &self,
        input: &Path,
        output: &Path,
        ffmpeg: &Ffmpeg<R>,
    ) -> Result<FileOutcome>;
}

// ----------------------------------------- Public API ----------------------------------------- //

/// Executes an interactive batch process over a collection of files.
///
/// Handles queue resolution (including sibling discovery), error policies,
/// continuation prompts, and final reporting.
pub fn run_batch<R: ProcessRunner, T: BatchTask>(
    task: &T,
    args: &BatchArgs,
    explicit_files: Vec<PathBuf>,
    ffmpeg: &Ffmpeg<R>,
) -> Result<()> {
    println!(
        "{}",
        banner::render(
            task.file_type_name(),
            Some("Batch Processing"),
            console::colors_enabled()
        )
    );

    let (queue, policy) = resolve_queue_and_policy(task, args, explicit_files)?;

    if queue.is_empty() {
        println!("No {} files found to process.", task.file_type_name());
        return Ok(());
    }

    // Ensure output directory exists if we are going to process anything
    output::ensure_directory(&args.output_dir)?;

    execute_queue(task, args, &queue, policy, ffmpeg)
}

// -------------------------------------- Internal Helpers -------------------------------------- //

/// Resolves the execution queue and batch policy from CLI arguments and interactive prompts.
fn resolve_queue_and_policy<T: BatchTask>(
    task: &T,
    args: &BatchArgs,
    explicit_files: Vec<PathBuf>,
) -> Result<(Vec<PathBuf>, BatchPolicy)> {
    let ext = task.input_extension();
    let exclude = task.exclude_stem_suffix();

    match (explicit_files.len(), &args.input_dir, args.batch) {
        // Explicit directory scan
        (0, Some(dir), _) | (_, Some(dir), _) => {
            if !explicit_files.is_empty() {
                bail!("Cannot combine explicit files with --input-dir");
            }
            let dir = dir
                .canonicalize()
                .with_context(|| format!("directory not found: {}", dir.display()))?;
            let queue = files::queue_from_directory(&dir, ext, exclude)?;
            let policy = resolve_explicit_policy(args.on_error);
            Ok((queue, policy))
        }
        // Explicit batch flag without files or input-dir -> scan CWD
        (0, None, true) => {
            let cwd = std::env::current_dir().context("reading current directory")?;
            let queue = files::queue_from_directory(&cwd, ext, exclude)?;
            let policy = resolve_explicit_policy(args.on_error);
            Ok((queue, policy))
        }
        // Single file provided, no batch flags -> sibling discovery
        (1, None, false) => {
            if args.on_error.is_some() {
                bail!("--on-error can only be used with --batch or --input-dir");
            }
            let file = explicit_files.into_iter().next().unwrap();
            resolve_single_file_with_siblings(task, &file)
        }
        // Multiple files provided, no batch flags
        (n, None, false) if n > 1 => {
            if args.on_error.is_some() {
                bail!("--on-error can only be used with --batch or --input-dir");
            }
            // Just use the provided files, default to skip on error for safety
            let queue = filter_and_canonicalize(explicit_files, task.file_type_name());
            Ok((queue, BatchPolicy::SkipOnError))
        }
        // No files, no flags -> Fallback: scan CWD
        (0, None, false) => {
            let cwd = std::env::current_dir().context("reading current directory")?;
            let queue = files::queue_from_directory(&cwd, ext, exclude)?;
            let policy = resolve_explicit_policy(args.on_error);
            Ok((queue, policy))
        }
        _ => bail!("Invalid combination of batch arguments"),
    }
}

/// Resolves policy for explicit batch runs (--batch or --input-dir).
fn resolve_explicit_policy(on_error: Option<BatchOnError>) -> BatchPolicy {
    match on_error {
        Some(BatchOnError::Stop) => BatchPolicy::StopOnError,
        Some(BatchOnError::Skip) => BatchPolicy::SkipOnError,
        Some(BatchOnError::Prompt) => BatchPolicy::PromptEach,
        None => {
            if io::stdin().is_terminal() {
                BatchPolicy::PromptEach
            } else {
                BatchPolicy::SkipOnError
            }
        }
    }
}

/// Handles sibling discovery and prompting for a single explicit file.
fn resolve_single_file_with_siblings<T: BatchTask>(
    task: &T,
    file: &Path,
) -> Result<(Vec<PathBuf>, BatchPolicy)> {
    let canonical = file
        .canonicalize()
        .with_context(|| format!("file not found: {}", file.display()))?;

    let queue = files::queue_from_entry(
        &canonical,
        task.input_extension(),
        task.exclude_stem_suffix(),
    )?;

    if queue.len() <= 1 {
        return Ok((vec![canonical], BatchPolicy::Single));
    }

    let parent = canonical.parent().unwrap_or_else(|| Path::new("."));
    let stdin = io::stdin();
    let mut input = BufReader::new(stdin.lock());
    let mut stdout = io::stdout();

    let choice = prompt::sibling_batch_choice(&mut input, &mut stdout, parent, queue.len())?;

    let (policy, final_queue) = match choice {
        SiblingBatchChoice::ProcessInputOnly => (BatchPolicy::Single, vec![canonical]),
        SiblingBatchChoice::ProcessAllStopOnError => (BatchPolicy::StopOnError, queue),
        SiblingBatchChoice::ProcessAllSkipOnError => (BatchPolicy::SkipOnError, queue),
        SiblingBatchChoice::ProcessAllPromptEach => (BatchPolicy::PromptEach, queue),
    };

    Ok((final_queue, policy))
}

/// Filters explicit files, warning on invalid ones, and canonicalizes them.
fn filter_and_canonicalize(files: Vec<PathBuf>, type_name: &str) -> Vec<PathBuf> {
    files
        .into_iter()
        .filter_map(|p| match p.canonicalize() {
            Ok(c) => Some(c),
            Err(_) => {
                eprintln!(
                    "{WARNING_PREFIX}: skipping invalid {type_name} input: {}",
                    p.display()
                );
                None
            }
        })
        .collect()
}

/// Executes the resolved queue with the given policy.
fn execute_queue<R: ProcessRunner, T: BatchTask>(
    task: &T,
    args: &BatchArgs,
    queue: &[PathBuf],
    initial_policy: BatchPolicy,
    ffmpeg: &Ffmpeg<R>,
) -> Result<()> {
    let total = queue.len();
    let is_single = initial_policy == BatchPolicy::Single && total == 1;

    let mut policy = initial_policy;
    let mut report = BatchReport::new();

    for (index, input) in queue.iter().enumerate() {
        if total > 1 {
            println!("File {}/{}: {}", index + 1, total, input.display());
        }

        let out = match output::output_path(input, &args.output_dir, task.output_extension()) {
            Ok(p) => p,
            Err(e) => {
                report.record_failed(input.clone(), e.to_string());
                if policy == BatchPolicy::StopOnError {
                    report.print_summary();
                    bail!("stopped on output path error: {e}");
                }
                continue;
            }
        };

        let mut processed = false;

        if output::decision(&out, args.force) == OutputDecision::SkipExisting {
            println!("⏭ SKIPPED: {} already exists", out.display());
            report.record_skipped(input.clone(), "output exists");
        } else {
            processed = true;
            match task.process_file(input, &out, ffmpeg) {
                Ok(FileOutcome::Success) => {
                    println!("✔ SUCCESS: {}", out.display());
                    report.record_success(input.clone());
                }
                Ok(FileOutcome::Skipped(reason)) => {
                    println!("⏭ SKIPPED: {reason}");
                    report.record_skipped(input.clone(), reason);
                    processed = false; // Task decided to skip, don't prompt
                }
                Err(error) => {
                    eprintln!("✖ FAILED: {}: {error}", input.display());
                    report.record_failed(input.clone(), error.to_string());

                    match policy {
                        BatchPolicy::Single | BatchPolicy::StopOnError => {
                            report.print_summary();
                            bail!("stopped after error on {}: {error}", input.display());
                        }
                        BatchPolicy::SkipOnError => {
                            eprintln!("Continuing past error...");
                        }
                        BatchPolicy::PromptEach => {
                            eprintln!("Error occurred.");
                        }
                    }
                }
            }
        }

        if processed && policy == BatchPolicy::PromptEach && index + 1 < total {
            let next = &queue[index + 1];
            let stdin = io::stdin();
            let mut input_stream = BufReader::new(stdin.lock());
            let mut stdout = io::stdout();

            match prompt::continue_to_next(&mut input_stream, &mut stdout, next)? {
                ContinueChoice::Yes => {}
                ContinueChoice::YesToAll => {
                    policy = BatchPolicy::SkipOnError;
                }
                ContinueChoice::No => break,
            }
        }
    }

    if !is_single {
        report.print_summary();
    }

    if report.has_failures() {
        bail!("one or more conversions failed")
    } else {
        Ok(())
    }
}
