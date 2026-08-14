// Package ffmpeg. encoder.go - Provides abstractions for media processing commands.
// It encapsulates FFmpeg argument construction and execution with
// progress feedback, isolating external dependency details from core logic.
package ffmpeg

import (
	"fmt"
	"io"
	"os/exec"
	"strings"
	"vidwrap/internal/ui"
)

// ImageToJPGArgs returns arguments to convert an image to even-dimensioned JPEG.
// Ensures compatibility with libx264 encoding requirements.
func ImageToJPGArgs(input, output string) []string {
	return []string{
		"-i", input,
		"-vf", "scale=trunc(iw/2)*2:trunc(ih/2)*2",
		"-pix_fmt", "yuv420p",
		"-y", output,
	}
}

// StripThumbnailArgs returns arguments to copy streams without metadata.
// Removes existing embedded thumbnails while preserving A/V content.
func StripThumbnailArgs(input, output string) []string {
	return []string{
		"-i", input,
		"-map", "0:v", "-map", "0:a",
		"-c", "copy",
		"-y", output,
	}
}

// EncodeLoopArgs returns arguments to create a video from a still image.
// Loops the image for the duration of the audio track using ultrafast preset.
func EncodeLoopArgs(image, audio, output string) []string {
	return []string{
		"-loop", "1", "-i", image,
		"-i", audio,
		"-c:v", "libx264", "-preset", "ultrafast", "-crf", "23",
		"-c:a", "copy", "-shortest",
		"-y", output,
	}
}

// AttachThumbnailArgs returns arguments to embed an image as attached_pic.
// Sets proper disposition for media player thumbnail recognition.
func AttachThumbnailArgs(video, image, output string) []string {
	return []string{
		"-i", video, "-i", image,
		"-map", "0:v", "-map", "0:a", "-map", "1",
		"-c", "copy",
		"-disposition:v:1", "attached_pic",
		"-y", output,
	}
}

// RunWithSpinner executes an FFmpeg command with animated progress feedback.
// Combines stdout/stderr capture with non-blocking spinner rendering.
func RunWithSpinner(args []string, message, style string) error {
	ffmpegBin, err := exec.LookPath("ffmpeg")
	if err != nil {
		return fmt.Errorf("ffmpeg not found in PATH: %w", err)
	}

	cmd := exec.Command(ffmpegBin, args...)
	stdoutPipe, _ := cmd.StdoutPipe()
	stderrPipe, _ := cmd.StderrPipe()

	if err := cmd.Start(); err != nil {
		return fmt.Errorf("failed to start ffmpeg: %w", err)
	}

	done := make(chan struct{})
	go ui.RenderSpinner(style, message, done)

	var outBuf, errBuf strings.Builder
	go io.Copy(&outBuf, stdoutPipe)
	go io.Copy(&errBuf, stderrPipe)

	err = cmd.Wait()
	close(done)
	fmt.Printf("\r%-80s\r", "")

	if err != nil {
		output := errBuf.String()
		if output == "" {
			output = outBuf.String()
		}
		if output != "" {
			fmt.Println(output)
		}
		return fmt.Errorf("ffmpeg exited with error: %w", err)
	}
	return nil
}
