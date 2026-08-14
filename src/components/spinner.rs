//! module spinner - A non-blocking spinner for stderr output.

use std::{
    io::{self, IsTerminal, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

// --------------------------------- Types, Constants & Variables ------------------------------- //

/// Delay between spinner frame updates in milliseconds.
const SPINNER_DELAY_MS: u64 = 100;

/// Supported spinner frame sequences.
#[derive(Debug, Clone, Copy)]
pub enum SpinnerStyle {
    Dots,
    Arrow,
    Bounce,
    Pulse,
    Bar,
    Spin,
    Circle,
}
impl SpinnerStyle {
    /// Returns the frame string for the given index.
    pub fn frame(self, index: usize) -> &'static str {
        let frames: &[&str] = match self {
            Self::Dots => &["⠋", "⠙", "⠹", "⠸"],
            Self::Arrow => &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
            Self::Bounce => &["⠁", "⠂", "⠄", "⠂"],
            Self::Pulse => &["◐", "◓", "◑", "◒"],
            Self::Bar => &["▏", "▎", "▍", "▌", "▋", "▊", "▉", "█"],
            Self::Spin => &["-", "\\", "|", "/"],
            Self::Circle => &["◴", "◷", "◶", "◵"],
        };
        frames[index % frames.len()]
    }
}

/// A lightweight non-blocking stderr spinner. It is disabled on non-terminals.
pub struct Spinner {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    enabled: bool,
}
impl Spinner {
    /// Starts a new spinner with the given style and message.
    ///
    /// # Arguments
    /// * `style` - The spinner frame style.
    /// * `message` - The message to display next to the spinner.
    /// * `force` - If true, enable spinner even on non-terminals.
    ///
    /// # Returns
    /// A `Spinner` instance that must be stopped to clean up the thread.
    pub fn start(style: SpinnerStyle, message: String, force: bool) -> Self {
        let enabled = force || io::stderr().is_terminal();
        let stop = Arc::new(AtomicBool::new(false));
        let handle = if enabled {
            let signal = Arc::clone(&stop);
            Some(thread::spawn(move || {
                let mut index = 0;
                while !signal.load(Ordering::Relaxed) {
                    eprint!("\r{} {message}", style.frame(index));
                    let _ = io::stderr().flush();
                    index += 1;
                    thread::sleep(Duration::from_millis(SPINNER_DELAY_MS));
                }
                eprint!("\r{}\r", " ".repeat(message.len() + 4));
                let _ = io::stderr().flush();
            }))
        } else {
            None
        };
        Self {
            stop,
            handle,
            enabled,
        }
    }

    /// Returns whether the spinner is enabled (i.e., outputting frames).
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Stops the spinner and waits for the thread to finish.
    pub fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
impl Drop for Spinner {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cycles_frames() {
        assert_eq!(SpinnerStyle::Spin.frame(0), "-");
        assert_eq!(SpinnerStyle::Spin.frame(4), "-");
    }
}
