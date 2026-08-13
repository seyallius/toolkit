use console::Style;

/// Renders a boxed heading suitable for any command.
pub fn render(title: &str, subtitle: Option<&str>, color: bool) -> String {
    let content = match subtitle {
        Some(value) => format!("{title} — {value}"),
        None => title.to_owned(),
    };
    let border = "═".repeat(content.chars().count() + 2);
    let title = if color {
        Style::new().cyan().bold().apply_to(content).to_string()
    } else {
        content
    };
    format!("╔{border}╗\n║ {title} ║\n╚{border}╝")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renders_box() {
        assert!(render("Hello", None, false).contains("║ Hello ║"));
    }
}
