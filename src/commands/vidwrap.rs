//! module vidwrap - Wrap a video with a companion image to create a new video with thumbnail.

use crate::{
    components::{
        banner, progress, prompt,
        spinner::{Spinner, SpinnerStyle},
    },
    ffmpeg::{args, Ffmpeg, ProcessRunner},
    util::files,
};
use anyhow::{Context, Result};
use clap::Args;
use console::Style;
use std::{
    fs,
    io::{self, BufReader},
    path::{Path, PathBuf},
};

// --------------------------------- Types, Constants & Variables ------------------------------- //

/// Total number of steps in the vidwrap workflow.
const TOTAL_STEPS: usize = 1;

/// Default choice index for post-processing prompt (1-based).
const DEFAULT_POST_CHOICE: usize = 2;

/// Arguments for the `vidwrap` subcommand.
#[derive(Debug, Args)]
pub struct VidwrapArgs {
    /// Video with a same-basename companion image.
    #[arg(value_name = "VIDEO")]
    pub video: PathBuf,
}

// ----------------------------------------- Public API ----------------------------------------- //

/// Runs the vidwrap workflow: creates a static image video with audio from the original.
///
/// Adapts its success output based on whether the spinner was enabled (TTY vs piped).
pub fn run<R: ProcessRunner>(args_cli: VidwrapArgs, ffmpeg: &Ffmpeg<R>) -> Result<()> {
    println!(
        "{}",
        banner::render(
            "Vidwrap",
            Some("Video + Image wrapper"),
            console::colors_enabled()
        )
    );

    let video = args_cli
        .video
        .canonicalize()
        .with_context(|| format!("video file not found: {}", args_cli.video.display()))?;
    let image = files::companion_image(&video)?;

    println!(
        "Found image: {}\nFound video: {}",
        image.display(),
        video.display()
    );

    let dir = video.parent().context("video has no parent")?;
    let stem = video
        .file_stem()
        .context("video has no stem")?
        .to_string_lossy();
    let output = dir.join(format!("{stem}_with_image.mp4"));

    let steps: [(String, SpinnerStyle, Vec<String>); TOTAL_STEPS] = [(
        "Creating video with static image".to_string(),
        SpinnerStyle::Bounce,
        args::replace_video_with_image(&image, &video, &output, &[]),
    )];

    for (index, (label, style, command)) in steps.into_iter().enumerate() {
        println!("{}", progress::render(index + 1, TOTAL_STEPS, &label));

        let spin = Spinner::start(style, label.clone(), false);
        let result = ffmpeg.run(command);

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
    }

    println!("Output: {}", output.display());
    post_process(&video, &image, &output)
}

// -------------------------------------- Internal Helpers -------------------------------------- //

/// Prompts the user for post-processing actions on the original video and image.
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
