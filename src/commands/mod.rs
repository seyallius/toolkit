//! module commands - Subcommand implementations for the toolkit.

pub mod mkv2mp3;
pub mod mp32mp4;
pub mod ts2mp4;
pub mod vidwrap;

use crate::{
    cli::{Cli, Command},
    ffmpeg::{Ffmpeg, RealRunner},
};
use anyhow::Result;

// ----------------------------------------- Public API ----------------------------------------- //

/// Dispatches the selected subcommand.
///
/// New commands need one enum variant and one arm here.
pub fn run(cli: Cli) -> Result<()> {
    console::set_colors_enabled(!cli.no_color);
    let ffmpeg = Ffmpeg::new(cli.ffmpeg_path, cli.verbose, RealRunner);
    match cli.command {
        Command::Ts2mp4(args) => ts2mp4::run(args, &ffmpeg),
        Command::Mkv2mp3(args) => mkv2mp3::run(args, &ffmpeg),
        Command::Mp32mp4(args) => mp32mp4::run(args, &ffmpeg),
        Command::Vidwrap(args) => vidwrap::run(args, &ffmpeg),
    }
}
