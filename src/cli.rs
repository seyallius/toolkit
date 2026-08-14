//! module cli - Defines command-line interface structures and parsing.

use crate::commands::{
    mkv2mp3::Mkv2mp3Args, mp32mp4::Mp32mp4Args, ts2mp4::Ts2mp4Args, vidwrap::VidwrapArgs,
};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

// -------------------------------------------- Types ------------------------------------------- //

/// Command line entry point for the toolkit.
#[derive(Debug, Parser)]
#[command(
    name = "toolkit",
    version,
    about = "FFmpeg workflows for common media tasks"
)]
pub struct Cli {
    /// Print commands and diagnostic output.
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Disable ANSI styling.
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Explicit path to the ffmpeg executable.
    #[arg(long, global = true)]
    pub ffmpeg_path: Option<PathBuf>,

    /// Explicit path to ffprobe (reserved for commands that need it).
    #[arg(long, global = true)]
    pub ffprobe_path: Option<PathBuf>,

    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Command,
}

/// Available subcommands for the toolkit.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Convert TS files to MP4 via stream copy.
    Ts2mp4(Ts2mp4Args),

    /// Convert MKV files to MP3 with optional cover art.
    Mkv2mp3(Mkv2mp3Args),

    /// Convert MP3 files to MP4 videos with optional cover art.
    Mp32mp4(Mp32mp4Args),

    /// Wrap a video with a companion image to create a new video with thumbnail.
    Vidwrap(VidwrapArgs),
}

/// Common conversion options shared by multiple commands.
#[derive(Debug, Clone, Args)]
pub struct BatchArgs {
    /// Directory for generated media.
    #[arg(long, default_value = "out")]
    pub output_dir: PathBuf,

    /// Overwrite an existing output file.
    #[arg(long)]
    pub force: bool,
}
