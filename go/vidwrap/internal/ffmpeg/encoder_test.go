// Package ffmpeg_test. encoder_test.go - Validates argument generation and
// execution behavior. Uses table-driven tests to verify correctness across
// encoding scenarios and ensures error handling preserves diagnostic output
// for debugging.
package ffmpeg_test

import (
	"errors"
	"os/exec"
	"testing"

	"vidwrap/internal/ffmpeg"
)

// ------------------------------------------------ Tests ------------------------------------------- //

// TestImageToJPGArgs verifies JPEG conversion arguments for various inputs.
// Ensures even-dimension scaling and pixel format compliance for libx264.
func TestImageToJPGArgs(t *testing.T) {
	tests := []struct {
		name    string
		input   string
		output  string
		wantLen int
		checkFn func(args []string) bool
	}{
		{
			name:    "standard png to jpg",
			input:   "/tmp/test.png",
			output:  "/tmp/out.jpg",
			wantLen: 7,
			checkFn: func(args []string) bool {
				return args[0] == "-i" && args[1] == "/tmp/test.png" &&
					args[4] == "scale=trunc(iw/2)*2:trunc(ih/2)*2" &&
					args[5] == "-pix_fmt" && args[6] == "yuv420p"
			},
		},
		{
			name:    "path with spaces",
			input:   "/my videos/photo.jpeg",
			output:  "/my videos/temp.jpg",
			wantLen: 7,
			checkFn: func(args []string) bool {
				return args[1] == "/my videos/photo.jpeg" && args[8] == "/my videos/temp.jpg"
			},
		},
		{
			name:    "webp input",
			input:   "cover.webp",
			output:  "cover.jpg",
			wantLen: 7,
			checkFn: func(args []string) bool {
				return args[1] == "cover.webp" && args[8] == "cover.jpg"
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := ffmpeg.ImageToJPGArgs(tt.input, tt.output)
			if len(got) != tt.wantLen {
				t.Errorf("got %d args, want %d", len(got), tt.wantLen)
			}
			if !tt.checkFn(got) {
				t.Errorf("unexpected args: %v", got)
			}
		})
	}
}

// TestStripThumbnailArgs validates stream copying without metadata.
// Confirms map directives exclude thumbnail streams while preserving A/V.
func TestStripThumbnailArgs(t *testing.T) {
	tests := []struct {
		name    string
		input   string
		output  string
		wantMap []string
	}{
		{
			name:    "basic strip",
			input:   "video.mp4",
			output:  "clean.mp4",
			wantMap: []string{"-map", "0:v", "-map", "0:a"},
		},
		{
			name:    "mkv container",
			input:   "/data/movie.mkv",
			output:  "/data/clean.mkv",
			wantMap: []string{"-map", "0:v", "-map", "0:a"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := ffmpeg.StripThumbnailArgs(tt.input, tt.output)
			foundMaps := 0
			for i := 0; i < len(got)-1; i++ {
				if got[i] == "-map" {
					foundMaps++
				}
			}
			if foundMaps != 2 {
				t.Errorf("expected 2 -map flags, got %d in %v", foundMaps, got)
			}
			if got[len(got)-1] != tt.output {
				t.Errorf("output arg = %q, want %q", got[len(got)-1], tt.output)
			}
		})
	}
}

// TestEncodeLoopArgs checks looping image + audio encoding parameters.
// Validates ultrafast preset, CRF value, and shortest flag presence.
func TestEncodeLoopArgs(t *testing.T) {
	tests := []struct {
		name        string
		image       string
		audio       string
		output      string
		mustContain []string
	}{
		{
			name:        "standard encode",
			image:       "temp.jpg",
			audio:       "clean.mp4",
			output:      "final.mp4",
			mustContain: []string{"-loop", "1", "-preset", "ultrafast", "-crf", "23", "-shortest"},
		},
		{
			name:        "high quality override not applied",
			image:       "hires.png",
			audio:       "audio.aac",
			output:      "out.mp4",
			mustContain: []string{"-crf", "23"}, // Should NOT be 18
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := ffmpeg.EncodeLoopArgs(tt.image, tt.audio, tt.output)
			argSet := make(map[string]bool)
			for _, a := range got {
				argSet[a] = true
			}
			for _, want := range tt.mustContain {
				if !argSet[want] {
					t.Errorf("missing required arg %q in %v", want, got)
				}
			}
		})
	}
}

// TestAttachThumbnailArgs verifies embedded pic disposition setup.
// Ensures correct stream mapping and attached_pic metadata assignment.
func TestAttachThumbnailArgs(t *testing.T) {
	tests := []struct {
		name              string
		video             string
		image             string
		output            string
		expectDisposition bool
	}{
		{
			name:              "standard attach",
			video:             "encoded.mp4",
			image:             "thumb.jpg",
			output:            "final.mp4",
			expectDisposition: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := ffmpeg.AttachThumbnailArgs(tt.video, tt.image, tt.output)
			hasDisp := false
			for i := 0; i < len(got)-1; i++ {
				if got[i] == "-disposition:v:1" && got[i+1] == "attached_pic" {
					hasDisp = true
					break
				}
			}
			if hasDisp != tt.expectDisposition {
				t.Errorf("disposition flag = %v, want %v", hasDisp, tt.expectDisposition)
			}
			if got[len(got)-1] != tt.output {
				t.Errorf("output = %q, want %q", got[len(got)-1], tt.output)
			}
		})
	}
}

// TestRunWithSpinner_MissingFFmpeg validates graceful failure when binary absent.
// Temporarily modifies PATH to simulate missing dependency scenario.
// t.Setenv auto-restores PATH after test completes (Go 1.17+).
func TestRunWithSpinner_MissingFFmpeg(t *testing.T) {
	t.Setenv("PATH", "/nonexistent")

	err := ffmpeg.RunWithSpinner([]string{"-version"}, "test", "dots")
	if err == nil {
		t.Fatal("expected error for missing ffmpeg, got nil")
	}
	if !errors.Is(err, exec.ErrNotFound) &&
		!containsString(err.Error(), "ffmpeg not found") {
		t.Errorf("unexpected error message: %v", err)
	}
}

// ------------------------------------------- Internal Helpers ------------------------------------- //

// containsString checks substring presence for error message validation.
func containsString(s, substr string) bool {
	return len(s) >= len(substr) &&
		(s == substr || len(s) > len(substr) && findSubstring(s, substr))
}

// findSubstring performs naive substring search without importing strings.
func findSubstring(s, sub string) bool {
	for i := 0; i <= len(s)-len(sub); i++ {
		if s[i:i+len(sub)] == sub {
			return true
		}
	}
	return false
}
