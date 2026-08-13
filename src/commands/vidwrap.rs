use crate::{
    components::{
        progress, prompt,
        spinner::{Spinner, SpinnerStyle},
    },
    ffmpeg::{args, Ffmpeg, ProcessRunner},
    util::files,
};
use anyhow::{Context, Result};
use clap::Args;
use std::{
    fs,
    io::{self, BufReader},
    path::{Path, PathBuf},
};
use tempfile::Builder;

#[derive(Debug, Args)]
pub struct VidwrapArgs {
    /// Video with a same-basename companion image.
    #[arg(value_name = "VIDEO")]
    pub video: PathBuf,
}

pub fn run<R: ProcessRunner>(args_cli: VidwrapArgs, ffmpeg: &Ffmpeg<R>) -> Result<()> {
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
    let temp_jpg = temp_in(dir, "toolkit-image-", ".jpg")?;
    let temp_clean = temp_in(dir, "toolkit-clean-", ".mp4")?;
    let temp_video = temp_in(dir, "toolkit-video-", ".mp4")?;
    let steps: [(String, SpinnerStyle, Vec<String>); 4] = [
        (
            "Converting image to JPG".into(),
            SpinnerStyle::Bounce,
            args::image_to_jpg(&image, &temp_jpg),
        ),
        (
            "Removing existing thumbnail".into(),
            SpinnerStyle::Pulse,
            args::strip_thumbnail(&video, &temp_clean),
        ),
        (
            "Encoding video with image".into(),
            SpinnerStyle::Circle,
            args::encode_loop(&temp_jpg, &temp_clean, &temp_video),
        ),
        (
            "Adding thumbnail".into(),
            SpinnerStyle::Arrow,
            args::attach_thumbnail(&temp_video, &temp_jpg, &output),
        ),
    ];
    for (index, (label, style, command)) in steps.into_iter().enumerate() {
        println!("{}", progress::render(index + 1, 4, &label));
        let spin = Spinner::start(style, label.clone(), false);
        let result = ffmpeg.run(command);
        spin.stop();
        if let Err(error) = result {
            eprintln!(
                "Failed; temporary files were kept for debugging: {}, {}, {}",
                temp_jpg.display(),
                temp_clean.display(),
                temp_video.display()
            );
            return Err(error);
        }
        println!("  Success");
    }
    for path in [&temp_jpg, &temp_clean, &temp_video] {
        let _ = fs::remove_file(path);
    }
    println!("Output: {}", output.display());
    post_process(&video, &image, &output)
}

fn temp_in(dir: &Path, prefix: &str, suffix: &str) -> Result<PathBuf> {
    let (_, path) = Builder::new()
        .prefix(prefix)
        .suffix(suffix)
        .tempfile_in(dir)?
        .keep()?;
    Ok(path)
}

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
        2,
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
