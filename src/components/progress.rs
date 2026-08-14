//! module progress - Renders a multistep status line.

// ----------------------------------------- Public API ----------------------------------------- //

/// Renders a progress label with step number and total.
///
/// # Arguments
/// * `step` - Current step index (1-based).
/// * `total` - Total number of steps.
/// * `label` - Description of the current step.
///
/// # Returns
/// A formatted string like "Step 2/4: Cleaning video...".
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
