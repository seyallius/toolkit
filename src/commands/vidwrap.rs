//! module vidwrap - Wrap videos with companion image to create a new video with thumbnail
//! images in single or batch mode.

use crate::{
    components::{
        banner, progress,
        prompt::{self, ContinueChoice, SiblingBatchChoice},
        spinner::{Spinner, SpinnerStyle},
    },
    ffmpeg::{args, Ffmpeg, ProcessRunner},
    util::{
        batch::{BatchPolicy, BatchReport},
        files,
    },
};
use anyhow::{bail, Context, Result};
use clap::{Args, ValueEnum};
use console::Style;
use std::{
    fs,
    io::{self, BufReader, IsTerminal},
    path::{Path, PathBuf},
};

// --------------------------------- Types, Constants & Variables ------------------------------- //

/// Total number of steps in the vidwrap workflow for one file.
const TOTAL_STEPS: usize = 1;

/// Default choice index for post-processing prompt (1-based).
const DEFAULT_POST_CHOICE: usize = 2;

/// Extension scanned for batch video discovery.
const MP4_EXT: &str = "mp4";

/// Suffix used by vidwrap outputs.
///
/// We exclude these from batch discovery so a directory scan does not
/// repeatedly re-wrap previously generated `*_with_image.mp4` files.
const VIDWRAP_OUTPUT_STEM_SUFFIX: &str = "_with_image";

/// Arguments for the `vidwrap` subcommand.
#[derive(Debug, Args)]
pub struct VidwrapArgs {
    /// Video with a same-basename companion image.
    ///
    /// Omit this when using `--batch` or `--input-dir`.
    #[arg(value_name = "VIDEO")]
    pub video: Option<PathBuf>,

    /// Process all MP4 videos in the current directory.
    #[arg(long)]
    pub batch: bool,

    /// Directory to scan for MP4 videos.
    ///
    /// This implies batch processing.
    #[arg(long, value_name = "DIR")]
    pub input_dir: Option<PathBuf>,

    /// Error policy for explicit batch mode.
    ///
    /// If omitted, interactive terminals default to prompt-each, while
    /// non-interactive terminals default to skip-on-error.
    #[arg(long, value_enum)]
    pub on_error: Option<OnError>,
}

/// CLI error policy for explicit batch scans.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OnError {
    /// Stop and report on the first error.
    Stop,

    /// Skip failed files and continue.
    Skip,

    /// Prompt after each video.
    Prompt,
}

/// Resolved execution plan for vidwrap.
///
/// This separates queue construction from execution, which makes the command
/// easier to reason about and easier to test later.
struct VidwrapPlan {
    /// Videos to process, in processing order.
    queue: Vec<PathBuf>,

    /// Policy for errors and continuation.
    policy: BatchPolicy,

    /// Whether the original single-file post-processing prompt should run.
    ///
    /// This is enabled only for true single-file mode. In batch mode we keep
    /// both files automatically to avoid prompting destructively per file.
    interactive_post: bool,
}

// ----------------------------------------- Public API ----------------------------------------- //

/// Runs the vidwrap workflow in either single-file or batch mode.
///
/// Single-file mode:
/// ```bash
/// toolkitrs vidwrap input.mp4
/// ```
///
/// Batch mode:
/// ```bash
/// toolkitrs vidwrap --batch
/// toolkitrs vidwrap --input-dir /videos
/// ```
pub fn run<R: ProcessRunner>(args_cli: VidwrapArgs, ffmpeg: &Ffmpeg<R>) -> Result<()> {
    println!(
        "{}",
        banner::render(
            "Vidwrap",
            Some("Video + Image wrapper"),
            console::colors_enabled()
        )
    );

    let plan = resolve_plan(&args_cli)?;

    if plan.queue.is_empty() {
        println!("No MP4 videos found to process.");
        return Ok(());
    }

    execute_plan(plan, ffmpeg)
}

// -------------------------------------- Internal Helpers -------------------------------------- //

/// Resolves the execution plan from CLI arguments and interactive prompts.
fn resolve_plan(args_cli: &VidwrapArgs) -> Result<VidwrapPlan> {
    match (&args_cli.video, &args_cli.input_dir, args_cli.batch) {
        (Some(video), None, false) => {
            if args_cli.on_error.is_some() {
                bail!("--on-error can only be used with --batch or --input-dir");
            }

            resolve_explicit_video(video)
        }
        (Some(_), _, _) => {
            bail!("VIDEO cannot be combined with --batch or --input-dir. Omit VIDEO to scan a directory.")
        }
        (None, Some(dir), _) => resolve_directory(dir, args_cli.on_error),
        (None, None, true) => {
            let cwd = std::env::current_dir().context("reading current directory")?;
            resolve_directory(&cwd, args_cli.on_error)
        }
        (None, None, false) => {
            bail!("provide VIDEO, or use --batch / --input-dir <DIR>")
        }
    }
}

/// Resolves a plan when the user explicitly supplies one video.
///
/// If additional MP4 siblings are discovered, this prompts the user with:
/// - process input only
/// - process whole path, stop on error
/// - process whole path, skip on error
/// - process whole path, prompt each
fn resolve_explicit_video(video: &Path) -> Result<VidwrapPlan> {
    let video = video
        .canonicalize()
        .with_context(|| format!("video file not found: {}", video.display()))?;

    let queue = files::queue_from_entry(&video, MP4_EXT, Some(VIDWRAP_OUTPUT_STEM_SUFFIX))?;

    if queue.len() <= 1 {
        return Ok(VidwrapPlan {
            queue: vec![video],
            policy: BatchPolicy::Single,
            interactive_post: true,
        });
    }

    let parent = video.parent().context("video has no parent")?;

    let stdin = io::stdin();
    let mut input = BufReader::new(stdin.lock());
    let mut stdout = io::stdout();

    let choice = prompt::sibling_batch_choice(&mut input, &mut stdout, parent, queue.len())?;

    let (policy, queue, interactive_post) = match choice {
        SiblingBatchChoice::ProcessInputOnly => (BatchPolicy::Single, vec![video], true),
        SiblingBatchChoice::ProcessAllStopOnError => (BatchPolicy::StopOnError, queue, false),
        SiblingBatchChoice::ProcessAllSkipOnError => (BatchPolicy::SkipOnError, queue, false),
        SiblingBatchChoice::ProcessAllPromptEach => (BatchPolicy::PromptEach, queue, false),
    };

    Ok(VidwrapPlan {
        queue,
        policy,
        interactive_post,
    })
}

/// Resolves a plan for explicit directory batch mode.
fn resolve_directory(dir: &Path, on_error: Option<OnError>) -> Result<VidwrapPlan> {
    let dir = dir
        .canonicalize()
        .with_context(|| format!("directory not found: {}", dir.display()))?;

    let queue = files::queue_from_directory(&dir, MP4_EXT, Some(VIDWRAP_OUTPUT_STEM_SUFFIX))?;

    let policy = match on_error {
        Some(OnError::Stop) => BatchPolicy::StopOnError,
        Some(OnError::Skip) => BatchPolicy::SkipOnError,
        Some(OnError::Prompt) => BatchPolicy::PromptEach,
        None => {
            if io::stdin().is_terminal() {
                BatchPolicy::PromptEach
            } else {
                BatchPolicy::SkipOnError
            }
        }
    };

    Ok(VidwrapPlan {
        queue,
        policy,
        interactive_post: false,
    })
}

/// Executes the resolved vidwrap plan.
///
/// This owns the batch loop, error policy, continue prompts, and final report.
fn execute_plan<R: ProcessRunner>(plan: VidwrapPlan, ffmpeg: &Ffmpeg<R>) -> Result<()> {
    let total = plan.queue.len();
    let is_single = plan.policy == BatchPolicy::Single && total == 1;

    let mut policy = plan.policy;
    let mut report = BatchReport::new();
    let mut index = 0;

    while index < total {
        let video = plan.queue[index].clone();

        if total > 1 {
            println!("Video {}/{}: {}", index + 1, total, video.display());
        }

        match process_one(&video, ffmpeg, plan.interactive_post && total == 1) {
            Ok(_) => {
                report.record_success(video.clone());
            }
            Err(error) => {
                report.record_failed(video.clone(), error.to_string());

                match policy {
                    BatchPolicy::Single => {
                        if !is_single {
                            report.print_summary();
                        }
                        return Err(error);
                    }
                    BatchPolicy::StopOnError => {
                        report.print_summary();
                        bail!(
                            "vidwrap stopped after error on {}: {error}",
                            video.display()
                        );
                    }
                    BatchPolicy::SkipOnError => {
                        eprintln!("Failed; continuing: {error}");
                    }
                    BatchPolicy::PromptEach => {
                        eprintln!("Failed: {error}");
                    }
                }
            }
        }

        index += 1;

        if policy == BatchPolicy::PromptEach && index < total {
            let next = &plan.queue[index];

            let stdin = io::stdin();
            let mut input = BufReader::new(stdin.lock());
            let mut stdout = io::stdout();

            match prompt::continue_to_next(&mut input, &mut stdout, next)? {
                ContinueChoice::Yes => {}
                ContinueChoice::YesToAll => {
                    // After "yes to all", stop prompting and continue
                    // automatically. Errors are skipped so the batch can
                    // finish and report at the end.
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
        bail!("one or more vidwrap operations failed");
    }

    Ok(())
}

/// Processes one video with its companion image.
///
/// When `interactive_post` is true, the original post-processing prompt is
/// shown after success. In batch mode this is disabled so we do not ask
/// destructive per-file questions for every video.
fn process_one<R: ProcessRunner>(
    video: &Path,
    ffmpeg: &Ffmpeg<R>,
    interactive_post: bool,
) -> Result<PathBuf> {
    let video = video
        .canonicalize()
        .with_context(|| format!("video file not found: {}", video.display()))?;

    let image = files::companion_image(&video)?;

    println!(
        "Found image: {}
Found video: {}",
        image.display(),
        video.display()
    );

    let output = output_path_for(&video)?;

    let label = "Creating video with static image".to_string();
    println!("{}", progress::render(1, TOTAL_STEPS, &label));

    let spin = Spinner::start(SpinnerStyle::Bounce, label.clone(), false);
    let result = ffmpeg.run(args::replace_video_with_image(&image, &video, &output, &[]));
    let was_enabled = spin.enabled();
    spin.stop();

    if let Err(error) = result {
        eprintln!(
            "Failed; output file may be incomplete: {}",
            output.display()
        );
        return Err(error);
    }

    if was_enabled {
        let green = Style::new().green().bold();
        println!("  {} Success", green.apply_to("✔"));
    } else {
        println!("  [OK] Success");
    }

    println!("Output: {}", output.display());

    if interactive_post {
        post_process(&video, &image, &output)?;
    }

    Ok(output)
}

/// Computes the vidwrap output path for a video.
///
/// Example:
/// `/videos/input.mp4` -> `/videos/input_with_image.mp4`
fn output_path_for(video: &Path) -> Result<PathBuf> {
    let dir = video.parent().context("video has no parent")?;
    let stem = video
        .file_stem()
        .context("video has no stem")?
        .to_string_lossy();

    Ok(dir.join(format!("{stem}{VIDWRAP_OUTPUT_STEM_SUFFIX}.mp4")))
}

/// Prompts the user for post-processing actions on the original video and image.
///
/// This remains enabled only for true single-file mode. In batch mode we keep
/// all files automatically to avoid destructive prompts for every file.
fn post_process(original: &Path, image: &Path, new_video: &Path) -> Result<()> {
    let stdin = io::stdin();
    let mut input = BufReader::new(stdin.lock());
    let mut stdout = io::stdout();

    let choice = prompt::choice(
        &mut input,
        &mut stdout,
        "What would you like to do?",
        &[
            "Replace original",
            "Keep both files",
            "Delete original only",
        ],
        DEFAULT_POST_CHOICE,
    )?;

    match choice {
        1 => {
            fs::remove_file(original)?;
            fs::rename(new_video, original)?;
            let _ = fs::remove_file(image);
            println!("Replaced original and cleaned up")
        }
        3 => {
            fs::remove_file(original)?;
            let _ = fs::remove_file(image);
            println!("Deleted original and source image")
        }
        _ => println!("Kept all files"),
    }

    Ok(())
}
