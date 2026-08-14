//! module ffmpeg - FFmpeg wrapper with argument builders and process execution.

pub mod args;
pub mod runner;

pub use runner::{Ffmpeg, ProcessRunner, RealRunner};
