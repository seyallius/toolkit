use std::path::PathBuf;

use crate::commands::{
    mkv2mp3::Mkv2mp3Args, mp32mp4::Mp32mp4Args, ts2mp4::Ts2mp4Args, vidwrap::VidwrapArgs,
};
use clap::{Args, Parser, Subcommand};

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
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Ts2mp4(Ts2mp4Args),
    Mkv2mp3(Mkv2mp3Args),
    Mp32mp4(Mp32mp4Args),
    Vidwrap(VidwrapArgs),
}

/// Common conversion options.
#[derive(Debug, Clone, Args)]
pub struct BatchArgs {
    /// Directory for generated media.
    #[arg(long, default_value = "out")]
    pub output_dir: PathBuf,
    /// Overwrite an existing output file.
    #[arg(long)]
    pub force: bool,
}
