//! module cli - Defines command-line interface structures and parsing.
//! Uses clap derive macros to generate argument parsing, help text, and validation
//! from annotated struct definitions.

use crate::commands::{
    mkv2mp3::Mkv2mp3Args, mp32mp4::Mp32mp4Args, ts2mp4::Ts2mp4Args, vidwrap::VidwrapArgs,
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

// -------------------------------------------- Types ------------------------------------------- //

/// Command line entry point for the toolkit.
/// Global options defined here apply to all subcommands automatically.
#[derive(Debug, Parser)]
#[command(
    name = "toolkit",
    version,
    about = "FFmpeg workflows for common media tasks"
)]
pub struct Cli {
    /// Print commands and diagnostic output to stderr.
    /// Useful for debugging failed conversions or verifying argument construction.
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Disable ANSI color codes and styling in all output.
    /// Automatically respected by banner and spinner components.
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Explicit path to the ffmpeg executable.
    /// Overrides PATH lookup when ffmpeg is installed in a non-standard location.
    #[arg(long, global = true)]
    pub ffmpeg_path: Option<PathBuf>,

    /// Explicit path to ffprobe binary.
    /// Reserved for future commands that need media inspection capabilities.
    #[arg(long, global = true)]
    pub ffprobe_path: Option<PathBuf>,

    /// The specific media workflow to execute.
    /// Each variant maps to a dedicated command module under src/commands/.
    #[command(subcommand)]
    pub command: Command,
}

/// Available subcommands for the toolkit.
/// Adding a new tool requires adding a variant here and a dispatch arm in commands::run().
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Convert TS files to MP4 via stream copy (no re-encoding).
    Ts2mp4(Ts2mp4Args),

    /// Convert MKV files to MP3 with optional cover art extraction.
    Mkv2mp3(Mkv2mp3Args),

    /// Convert MP3 files to MP4 videos with embedded cover art as video track.
    Mp32mp4(Mp32mp4Args),

    /// Wrap a video with a companion image to create a new video with thumbnail.
    Vidwrap(VidwrapArgs),
}

/// Common conversion options shared by batch processing commands.
/// Flattened into subcommand args via #[command(flatten)].
#[derive(Debug, Clone, Args)]
pub struct BatchArgs {
    /// Directory where converted media files will be written.
    /// Created automatically if it does not exist.
    #[arg(long, default_value = "out")]
    pub output_dir: PathBuf,

    /// Overwrite existing output files instead of skipping them.
    /// By default, files with matching output paths are skipped to prevent accidental data loss.
    #[arg(long)]
    pub force: bool,

    /// Process all matching files in the current directory (or --input-dir).
    /// If omitted, but files are provided, sibling discovery may trigger.
    #[arg(long)]
    pub batch: bool,

    /// Directory to scan for input files. Implies batch processing.
    #[arg(long, value_name = "DIR")]
    pub input_dir: Option<PathBuf>,

    /// Error policy for explicit batch mode.
    /// If omitted, interactive terminals default to prompt-each, while
    /// non-interactive terminals default to skip-on-error.
    #[arg(long, value_enum)]
    pub on_error: Option<BatchOnError>,
}

/// Error policy for explicit directory batch processing.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BatchOnError {
    /// Stop and report on first error.
    Stop,

    /// Skip errors and continue.
    Skip,

    /// Prompt after each file.
    Prompt,
}
