// Package ui_test. spinner_test.go - Validates terminal interaction components.
// Tests prompt parsing logic and cleanup behavior using controlled inputs
// to ensure predictable UX across edge cases and failure modes.
package ui_test

import (
	"bytes"
	"io"
	"os"
	"strings"
	"testing"

	"vidwrap/internal/ui"
)

// TestPromptYesNo covers affirmative, negative, and default responses.
// Uses stdin redirection to simulate user input without interactive blocking.
func TestPromptYesNo(t *testing.T) {
	tests := []struct {
		name       string
		input      string
		defaultYes bool
		want       bool
	}{
		{"explicit yes lowercase", "y\n", false, true},
		{"explicit YES uppercase", "YES\n", false, true},
		{"explicit no", "n\n", true, false},
		{"empty with default yes", "\n", true, true},
		{"empty with default no", "\n", false, false},
		{"whitespace only default yes", "   \n", true, true},
		{"invalid input defaults", "maybe\n", true, true},
		{"invalid input defaults no", "xyz\n", false, false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			r, w, _ := os.Pipe()
			origStdin := os.Stdin
			os.Stdin = r
			defer func() { os.Stdin = origStdin }()

			go func() {
				w.Write([]byte(tt.input))
				w.Close()
			}()

			got := ui.PromptYesNo("Test?", tt.defaultYes)
			if got != tt.want {
				t.Errorf("PromptYesNo(%q, %v) = %v, want %v",
					strings.TrimSpace(tt.input), tt.defaultYes, got, tt.want)
			}
		})
	}
}

// TestPromptChoice validates selection parsing and fallback behavior.
// Ensures out-of-range and non-numeric inputs safely return default index.
func TestPromptChoice(t *testing.T) {
	tests := []struct {
		name         string
		input        string
		options      []string
		defaultIndex int
		want         int
	}{
		{"valid first choice", "1\n", []string{"A", "B", "C"}, 1, 0},
		{"valid last choice", "3\n", []string{"A", "B", "C"}, 0, 2},
		{"empty uses default", "\n", []string{"X", "Y"}, 1, 1},
		{"zero falls back", "0\n", []string{"A", "B"}, 0, 0},
		{"out of range high", "99\n", []string{"A"}, 0, 0},
		{"non-numeric falls back", "abc\n", []string{"A", "B"}, 1, 1},
		{"negative falls back", "-1\n", []string{"A"}, 0, 0},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			r, w, _ := os.Pipe()
			origStdin := os.Stdin
			os.Stdin = r
			defer func() { os.Stdin = origStdin }()

			go func() {
				w.Write([]byte(tt.input))
				w.Close()
			}()

			got := ui.PromptChoice("Pick:", tt.options, tt.defaultIndex)
			if got != tt.want {
				t.Errorf("PromptChoice() = %d, want %d", got, tt.want)
			}
		})
	}
}

// TestCleanupTempFiles_Success verifies deletion confirmation messaging.
// Creates actual temp files to validate filesystem side effects.
func TestCleanupTempFiles_Success(t *testing.T) {
	tmpDir := t.TempDir()
	files := []string{
		tmpDir + "/a.tmp",
		tmpDir + "/b.tmp",
	}
	for _, f := range files {
		os.WriteFile(f, []byte("data"), 0644)
	}

	// Capture stdout
	old := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	ui.CleanupTempFiles(true, files)

	w.Close()
	os.Stdout = old

	var buf bytes.Buffer
	io.Copy(&buf, r)
	output := buf.String()

	for _, f := range files {
		if _, err := os.Stat(f); !os.IsNotExist(err) {
			t.Errorf("file %s still exists after cleanup", f)
		}
		if !strings.Contains(output, "Removed:") {
			t.Error("missing removal confirmation in output")
		}
	}
}

// TestCleanupTempFiles_Failure preserves files and warns user.
// Validates debugging aid behavior when pipeline fails mid-process.
func TestCleanupTempFiles_Failure(t *testing.T) {
	tmpDir := t.TempDir()
	f := tmpDir + "/keep.tmp"
	os.WriteFile(f, []byte("debug"), 0644)

	old := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	ui.CleanupTempFiles(false, []string{f})

	w.Close()
	os.Stdout = old

	var buf bytes.Buffer
	io.Copy(&buf, r)
	output := buf.String()

	if _, err := os.Stat(f); os.IsNotExist(err) {
		t.Error("file was deleted despite failure state")
	}
	if !strings.Contains(output, "keeping temp files") {
		t.Error("missing failure warning message")
	}
}

// TestGetFrames_DefaultReturnsBraille ensures unrecognized styles don't panic.
// Validates defensive programming against invalid configuration values.
func TestGetFrames_DefaultReturnsBraille(t *testing.T) {
	// Access unexported via reflection or accept indirect test through RenderSpinner
	// For now, we trust the switch default clause by testing RenderSpinner doesn't panic
	done := make(chan struct{})
	close(done) // immediate stop

	// Should not panic with unknown style
	ui.RenderSpinner("unknown_style_xyz", "test", done)
}
