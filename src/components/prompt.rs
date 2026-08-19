//! module prompt - Interactive user prompts with injectable streams.

use std::io::{self, BufRead, IsTerminal, Write};

// ----------------------------------------- Public API ----------------------------------------- //

/// Parse a yes/no response, falling back to `default` for unknown input.
#[allow(dead_code)]
pub fn parse_yes_no(value: &str, default: bool) -> bool {
    match value.trim().to_ascii_lowercase().as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => default,
    }
}

/// Parse a one-based choice, falling back to the one-based default.
pub fn parse_choice(value: &str, choices: usize, default: usize) -> usize {
    value
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|n| (1..=choices).contains(n))
        .unwrap_or(default)
}

/// Prompt for a choice from a list, using injectable input/output streams.
///
/// It is written with injectable streams, keeping interactive code testable.
/// Automatically falls back to the default choice if stdin is not a terminal (CI/CD safety).
///
/// # Arguments
/// * `input` - Readable stream for user input.
/// * `output` - Writable stream for the prompt.
/// * `question` - The question to display.
/// * `options` - List of option labels.
/// * `default` - Default option index (1-based).
///
/// # Returns
/// The chosen index (1-based) or the default on error.
pub fn choice<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    question: &str,
    options: &[&str],
    default: usize,
) -> io::Result<usize> {
    // CI/CD Safety: If stdin isn't a terminal, we can't wait for user input.
    // Auto-select the default to prevent the pipeline from hanging indefinitely.
    if !io::stdin().is_terminal() {
        writeln!(output, "{question}")?;
        writeln!(
            output,
            "  ⚙️  Non-interactive mode detected. Auto-selecting default: {default}"
        )?;
        return Ok(default);
    }

    writeln!(output, "{question}")?;
    for (index, option) in options.iter().enumerate() {
        writeln!(output, "  {}. {option}", index + 1)?;
    }
    write!(output, "Choice [{default}]: ")?;
    output.flush()?;

    let mut line = String::new();
    input.read_line(&mut line)?;
    Ok(parse_choice(&line, options.len(), default))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parsing_is_safe() {
        assert!(parse_yes_no("yes", false));
        assert!(!parse_yes_no("?", false));
        assert_eq!(parse_choice("9", 3, 2), 2);
    }
}
