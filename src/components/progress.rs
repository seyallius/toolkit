/// A reusable, pure multi-step status line.
pub fn render(step: usize, total: usize, label: &str) -> String {
    format!("Step {step}/{total}: {label}")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renders_step() {
        assert_eq!(
            render(2, 4, "Cleaning video..."),
            "Step 2/4: Cleaning video..."
        );
    }
}
