//! module ts2mp4 - Convert TS files to MP4 via stream copy.
//! Converts TS files to MP4 via stream copy using the batch pipeline.

use crate::{
    cli::BatchArgs,
    commands::batch::{run_batch, BatchTask, FileOutcome},
    ffmpeg::{args, Ffmpeg, ProcessRunner},
};
use anyhow::Result;
use clap::Args;
use std::path::{Path, PathBuf};

// --------------------------------- Types, Constants & Variables ------------------------------- //

/// Extension for transport stream files.
const TS_EXT: &str = "ts";

/// Extension for MP4 files.
const MP4_EXT: &str = "mp4";

/// Human readable name for TS files.
const FILE_TYPE_NAME: &str = "TS";

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

/// Task definition for TS to MP4 remuxing.
struct Ts2Mp4Task {
    /// Whether to force overwrite existing files.
    force: bool,
}

// ----------------------------------------- Public API ----------------------------------------- //

/// Runs the TS to MP4 conversion using the generic batch pipeline.
pub fn run<R: ProcessRunner>(args_cli: Ts2mp4Args, ffmpeg: &Ffmpeg<R>) -> Result<()> {
    let task = Ts2Mp4Task {
        force: args_cli.batch.force,
    };
    run_batch(
        &task,
        args_cli.files,
        &args_cli.batch.output_dir,
        args_cli.batch.force,
        ffmpeg,
    )
}

// -------------------------------------- Internal Helpers -------------------------------------- //

impl BatchTask for Ts2Mp4Task {
    fn input_extension(&self) -> &str {
        TS_EXT
    }

    fn output_extension(&self) -> &str {
        MP4_EXT
    }

    fn file_type_name(&self) -> &str {
        FILE_TYPE_NAME
    }

    fn process_file<R: ProcessRunner>(
        &self,
        input: &Path,
        output: &Path,
        ffmpeg: &Ffmpeg<R>,
    ) -> Result<FileOutcome> {
        ffmpeg.run(args::remux_copy(input, output, self.force))?;
        Ok(FileOutcome::Success)
    }
}
