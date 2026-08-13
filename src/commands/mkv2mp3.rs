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

#[derive(Debug, Args)]
pub struct Mkv2mp3Args {
    #[command(flatten)]
    pub batch: BatchArgs,

    #[arg(long, default_value_t = 600)]
    pub cover_size: u32,

    #[arg(long, default_value_t = 320)]
    pub bitrate: u32,

    /// MKV files; scans the current directory when omitted.
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}

pub fn run<R: ProcessRunner>(args_cli: Mkv2mp3Args, ffmpeg: &Ffmpeg<R>) -> Result<()> {
    output::ensure_directory(&args_cli.batch.output_dir)?;
    let inputs = collect(args_cli.files)?;
    let mut succeeded = 0;
    let mut skipped = 0;
    let mut failed = 0;
    for input in inputs {
        let out = output::output_path(&input, &args_cli.batch.output_dir, "mp3")?;
        if output::decision(&out, args_cli.batch.force) == OutputDecision::SkipExisting {
            println!("SKIPPED: {} already exists", out.display());
            skipped += 1;
            continue;
        }
        let cover = temp_path("toolkit-cover-", ".jpg")?;
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

fn collect(given: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    if given.is_empty() {
        let found = files::discover(&std::env::current_dir()?, "mkv")?;
        if found.is_empty() {
            println!("No MKV files found to process.");
        }
        Ok(found)
    } else {
        Ok(given
            .into_iter()
            .filter(|p| p.is_file() && files::has_extension(p, "mkv"))
            .collect())
    }
}

fn temp_path(prefix: &str, suffix: &str) -> Result<PathBuf> {
    let (_, path) = Builder::new()
        .prefix(prefix)
        .suffix(suffix)
        .tempfile()?
        .keep()?;
    Ok(path)
}
