// Package fileutil_test. image_test.go - Validates media companion file discovery logic.
// Uses temporary directories with controlled file layouts to verify
// extension matching priority and error message completeness.
package fileutil_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"vidwrap/internal/fileutil"
)

// TestFindImage_MatchPriority verifies extension search order.
// Ensures .png is preferred over .jpg when multiple candidates exist.
func TestFindImage_MatchPriority(t *testing.T) {
	dir := t.TempDir()
	video := filepath.Join(dir, "clip.mp4")
	os.WriteFile(video, []byte{}, 0644)

	// Create multiple candidates
	candidates := []string{"clip.png", "clip.jpg", "clip.webp"}
	for _, c := range candidates {
		os.WriteFile(filepath.Join(dir, c), []byte{}, 0644)
	}

	got, err := fileutil.FindImage(video)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if filepath.Base(got) != "clip.png" {
		t.Errorf("got %q, want clip.png (first in priority list)", filepath.Base(got))
	}
}

// TestFindImage_NoMatch returns descriptive error with tried extensions.
// Validates user-facing diagnostics include actionable troubleshooting info.
func TestFindImage_NoMatch(t *testing.T) {
	dir := t.TempDir()
	video := filepath.Join(dir, "orphan.mp4")
	os.WriteFile(video, []byte{}, 0644)

	_, err := fileutil.FindImage(video)
	if err == nil {
		t.Fatal("expected error for missing image, got nil")
	}

	requiredExts := []string{".png", ".jpg", ".jpeg", ".webp"}
	errMsg := err.Error()
	for _, ext := range requiredExts {
		if !strings.Contains(errMsg, ext) {
			t.Errorf("error message missing extension %q: %s", ext, errMsg)
		}
	}
	if !strings.Contains(errMsg, "orphan") {
		t.Errorf("error message should reference base name 'orphan': %s", errMsg)
	}
}

// TestFindImage_CaseSensitivity documents platform-dependent behavior.
// Notes that matching is case-sensitive per os.Stat semantics.
func TestFindImage_CaseSensitivity(t *testing.T) {
	dir := t.TempDir()
	video := filepath.Join(dir, "test.MP4")
	os.WriteFile(video, []byte{}, 0644)

	// Lowercase image won't match uppercase video base on Linux/macOS
	os.WriteFile(filepath.Join(dir, "test.png"), []byte{}, 0644)

	got, err := fileutil.FindImage(video)
	// On case-insensitive FS (Windows/macOS default), this may succeed
	// On Linux, it should fail since "test" != "TEST"
	if err != nil {
		t.Logf("Case-sensitive FS detected: %v (expected on Linux)", err)
	} else {
		t.Logf("Case-insensitive match: %s (expected on Windows/macOS)", got)
	}
}

// TestFindImage_SubdirectoryIsolation prevents false matches in nested dirs.
// Ensures search is strictly limited to video's immediate directory.
func TestFindImage_SubdirectoryIsolation(t *testing.T) {
	dir := t.TempDir()
	subdir := filepath.Join(dir, "sub")
	os.MkdirAll(subdir, 0755)

	video := filepath.Join(dir, "movie.mp4")
	os.WriteFile(video, []byte{}, 0644)

	// Image in subdir should NOT match
	os.WriteFile(filepath.Join(subdir, "movie.png"), []byte{}, 0644)

	_, err := fileutil.FindImage(video)
	if err == nil {
		t.Error("should not find image in subdirectory")
	}
}

// TestFindImage_SpecialCharacters handles paths with spaces and unicode.
// Validates robustness for real-world filenames from diverse sources.
func TestFindImage_SpecialCharacters(t *testing.T) {
	tests := []struct {
		name      string
		videoName string
		imgName   string
	}{
		{"spaces", "my vacation.mp4", "my vacation.jpg"},
		{"unicode", "動画テスト.mp4", "動画テスト.png"},
		{"parens", "clip (final).mp4", "clip (final).webp"},
		{"multiple dots", "v1.2.3.backup.mp4", "v1.2.3.backup.jpeg"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			dir := t.TempDir()
			video := filepath.Join(dir, tt.videoName)
			img := filepath.Join(dir, tt.imgName)

			os.WriteFile(video, []byte{}, 0644)
			os.WriteFile(img, []byte{}, 0644)

			got, err := fileutil.FindImage(video)
			if err != nil {
				t.Fatalf("failed to find image: %v", err)
			}
			if filepath.Base(got) != tt.imgName {
				t.Errorf("got %q, want %q", filepath.Base(got), tt.imgName)
			}
		})
	}
}
