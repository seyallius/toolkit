// Package cmd. root.go - Implements the primary workflow orchestration for vidwrap.
// It coordinates file discovery, user interaction, and encoding steps
// without containing low-level implementation details.
package cmd

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"vidwrap/internal/ffmpeg"
	"vidwrap/internal/fileutil"
	"vidwrap/internal/ui"
)

// -------------------------------------------- Public API ------------------------------------------ //

// Execute runs the main vidwrap workflow.
// It validates inputs, processes the video/image pair, and handles
// post-processing user choices. Returns an error if any step fails.
func Execute() error {
	if len(os.Args) < 2 {
		fmt.Println("Usage: vidwrap <video_file>")
		return fmt.Errorf("missing required argument")
	}

	videoPath := os.Args[1]
	absVideo, err := filepath.Abs(videoPath)
	if err != nil {
		fmt.Printf("❌ Error resolving path: %v\n", err)
		return err
	}

	if _, err = os.Stat(absVideo); err != nil {
		fmt.Printf("❌ Video file not found: %s\n", absVideo)
		return err
	}

	imagePath, err := fileutil.FindImage(absVideo)
	if err != nil {
		fmt.Printf("❌ %v\n", err)
		return err
	}

	fmt.Printf("\n📸 Found image: %s\n", filepath.Base(imagePath))
	fmt.Printf("🎬 Found video: %s\n", filepath.Base(absVideo))

	if err = processMedia(absVideo, imagePath); err != nil {
		return err
	}

	return handlePostProcess(absVideo, imagePath)
}

// ------------------------------------------- Internal Helpers ------------------------------------- //

// processMedia executes the four-step encoding pipeline.
// It manages temporary files and ensures cleanup on failure.
func processMedia(videoPath, imagePath string) error {
	dir := filepath.Dir(videoPath)
	baseName := strings.TrimSuffix(filepath.Base(videoPath), filepath.Ext(videoPath))

	tempJpg := filepath.Join(dir, "temp_"+baseName+".jpg")
	tempClean := filepath.Join(dir, "temp_clean_"+baseName+".mp4")
	tempVideo := filepath.Join(dir, "temp_video_"+baseName+".mp4")
	finalOutput := filepath.Join(dir, baseName+"_with_image.mp4")

	success := false
	defer func() {
		ui.CleanupTempFiles(success, []string{tempJpg, tempClean, tempVideo})
	}()

	steps := []struct {
		msg   string
		style string
		args  []string
		label string
	}{
		{"Optimizing dimensions...", "bounce", ffmpeg.ImageToJPGArgs(imagePath, tempJpg), "Converting image to JPG"},
		{"Cleaning video...", "pulse", ffmpeg.StripThumbnailArgs(videoPath, tempClean), "Removing existing thumbnail"},
		{"Encoding video...", "circle", ffmpeg.EncodeLoopArgs(tempJpg, tempClean, tempVideo), "Encoding video with image"},
		{"Embedding thumbnail...", "arrow", ffmpeg.AttachThumbnailArgs(tempVideo, tempJpg, finalOutput), "Adding thumbnail"},
	}

	for i, step := range steps {
		fmt.Printf("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n")
		fmt.Printf("📍 Step %d/4: %s\n", i+1, step.label)
		fmt.Printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n")

		if err := ffmpeg.RunWithSpinner(step.args, step.msg, step.style); err != nil {
			fmt.Printf("  ❌ Failed: %v\n", err)
			return err
		}
		fmt.Println("  ✅ Success")
	}

	success = true
	fmt.Println("\n✨ Encoding complete!")
	fmt.Printf("📁 Output: %s\n", finalOutput)
	return nil
}

// handlePostProcess prompts the user for final file management actions.
func handlePostProcess(originalVideo, imagePath string) error {
	baseName := strings.TrimSuffix(filepath.Base(originalVideo), filepath.Ext(originalVideo))
	dir := filepath.Dir(originalVideo)
	newVideo := filepath.Join(dir, baseName+"_with_image.mp4")

	options := []string{
		"Replace original (delete original, rename new)",
		"Keep both files",
		"Delete original only",
	}

	choice := ui.PromptChoice("\nWhat would you like to do?", options, 1)

	switch choice {
	case 0:
		os.Remove(originalVideo)
		os.Rename(newVideo, originalVideo)
		os.Remove(imagePath)
		fmt.Println("✅ Replaced original and cleaned up")
	case 2:
		os.Remove(originalVideo)
		os.Remove(imagePath)
		fmt.Println("✅ Deleted original and source image")
	default:
		fmt.Println("✅ Kept all files")
	}

	fmt.Println("\n🎉 All done! (ﾉ◕ヮ◕)ﾉ*:･ﾟ✧")
	return nil
}
