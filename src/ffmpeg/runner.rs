use std::{path::PathBuf, process::Command};
use anyhow::{bail, Result};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ProcessOutput {
    pub success: bool,
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Error)]
#[error("ffmpeg failed (exit {code:?}): {stderr}\ncommand: {command}")]
pub struct ProcessError {
    pub command: String,
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// Minimal seam around process execution, allowing deterministic command tests.
pub trait ProcessRunner {
    fn run(&self, binary: &str, args: &[String]) -> Result<ProcessOutput>;
}

pub struct RealRunner;
impl ProcessRunner for RealRunner {
    fn run(&self, binary: &str, args: &[String]) -> Result<ProcessOutput> {
        let output = Command::new(binary).args(args).output()?;
        Ok(ProcessOutput {
            success: output.status.success(),
            code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

/// Configured FFmpeg facade used by commands.
pub struct Ffmpeg<R> {
    binary: PathBuf,
    verbose: bool,
    runner: R,
}
impl<R: ProcessRunner> Ffmpeg<R> {
    pub fn new(binary: Option<PathBuf>, verbose: bool, runner: R) -> Self {
        Self {
            binary: binary.unwrap_or_else(|| PathBuf::from("ffmpeg")),
            verbose,
            runner,
        }
    }
    pub fn run(&self, args: Vec<String>) -> Result<()> {
        let binary = self.binary.to_string_lossy();
        if self.verbose {
            eprintln!("[ffmpeg] {binary} {}", args.join(" "));
        }
        let output = self.runner.run(&binary, &args)?;
        if output.success {
            Ok(())
        } else {
            if self.verbose && !output.stdout.is_empty() {
                eprintln!("{}", output.stdout);
            }
            bail!(ProcessError {
                command: format!("{binary} {}", args.join(" ")),
                code: output.code,
                stdout: output.stdout,
                stderr: output.stderr
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fake;
    impl ProcessRunner for Fake {
        fn run(&self, _: &str, _: &[String]) -> Result<ProcessOutput> {
            Ok(ProcessOutput {
                success: false,
                code: Some(2),
                stdout: String::new(),
                stderr: "bad input".into(),
            })
        }
    }
    #[test]
    fn error_has_stderr() {
        let error = Ffmpeg::new(None, false, Fake)
            .run(vec!["-i".into(), "x".into()])
            .unwrap_err();
        assert!(error.to_string().contains("bad input"));
    }
}
