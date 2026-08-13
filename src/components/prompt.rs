use std::io::{BufRead, Write};

/// Parse a yes/no response, falling back to `default` for unknown input.
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

/// Prompt with injectable streams, keeping interactive code testable.
pub fn choice<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    question: &str,
    options: &[&str],
    default: usize,
) -> std::io::Result<usize> {
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
