// Package ui. spinner.go - Provides terminal user interface components for vidwrap.
// Includes animated spinners, interactive prompts, and cleanup notifications
// to enhance CLI UX without coupling to business logic.
package ui

import (
	"bufio"
	"fmt"
	"os"
	"strings"
	"time"
)

// -------------------------------------------- Public API ------------------------------------------ //

// RenderSpinner displays an animated indicator until signaled via done channel.
// Supports multiple animation styles for contextual feedback differentiation.
func RenderSpinner(style, message string, done chan struct{}) {
	frames := getFrames(style)
	ticker := time.NewTicker(100 * time.Millisecond)
	defer ticker.Stop()

	i := 0
	for {
		select {
		case <-done:
			return
		case <-ticker.C:
			fmt.Printf("\r  %s %s", frames[i%len(frames)], message)
			i++
		}
	}
}

// PromptYesNo asks a binary question with configurable default.
// Returns true for affirmative responses or empty input when defaultYes is set.
func PromptYesNo(prompt string, defaultYes bool) bool {
	suffix := "[Y/n]"
	if !defaultYes {
		suffix = "[y/N]"
	}
	fmt.Printf("%s %s ", prompt, suffix)

	reader := bufio.NewReader(os.Stdin)
	line, _ := reader.ReadString('\n')
	line = strings.TrimSpace(strings.ToLower(line))

	if line == "" {
		return defaultYes
	}
	return line == "y" || line == "yes"
}

// PromptChoice presents numbered options and returns selected index.
// Falls back to defaultIndex on invalid or empty input for safe degradation.
func PromptChoice(prompt string, options []string, defaultIndex int) int {
	fmt.Println(prompt)
	for i, opt := range options {
		marker := " "
		if i == defaultIndex {
			marker = "→"
		}
		fmt.Printf("  %s %d. %s\n", marker, i+1, opt)
	}
	fmt.Printf("Enter choice (1-%d) [default %d]: ", len(options), defaultIndex+1)

	reader := bufio.NewReader(os.Stdin)
	line, _ := reader.ReadString('\n')
	line = strings.TrimSpace(line)

	if line == "" {
		return defaultIndex
	}
	var choice int
	if _, err := fmt.Sscan(line, &choice); err == nil && choice >= 1 && choice <= len(options) {
		return choice - 1
	}
	return defaultIndex
}

// CleanupTempFiles removes intermediate artifacts based on success state.
// Preserves files on failure for debugging; confirms deletion on success.
func CleanupTempFiles(success bool, files []string) {
	if !success {
		fmt.Println("\n⚠️  Process failed – keeping temp files for debugging.")
		return
	}
	fmt.Println("\n🧹 Cleaning up temporary files...")
	for _, f := range files {
		if _, err := os.Stat(f); err == nil {
			os.Remove(f)
			fmt.Printf("  ✓ Removed: %s\n", f)
		}
	}
	fmt.Println("✨ Cleanup complete!")
}

// ------------------------------------------- Internal Helpers ------------------------------------- //

// getFrames returns animation sequence for specified spinner style.
// Defaults to braille dots for unrecognized styles to prevent panics.
func getFrames(style string) []string {
	switch style {
	case "dots":
		return []string{"⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"}
	case "arrow":
		return []string{"▸", "▹", "▸", "▹"}
	case "bounce":
		return []string{"⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"}
	case "pulse":
		return []string{"◐", "◓", "◑", "◒"}
	case "bar":
		return []string{"▰", "▱", "▰", "▱"}
	case "spin":
		return []string{"|", "/", "-", "\\"}
	case "circle":
		return []string{"◴", "◷", "◶", "◵"}
	default:
		return []string{"⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"}
	}
}
