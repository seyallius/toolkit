// Package fileutil. image.go - Provides filesystem helpers for media file discovery.
// Encapsulates extension matching and path resolution logic to support
// flexible asset pairing without hardcoding formats in business logic.
package fileutil

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// FindImage locates an image companion for given video path.
// Searches same directory with common extensions; returns first match.
// Error includes tried extensions for user troubleshooting guidance.
func FindImage(videoPath string) (string, error) {
	extensions := []string{".png", ".jpg", ".jpeg", ".bmp", ".gif", ".webp"}
	base := strings.TrimSuffix(videoPath, filepath.Ext(videoPath))
	//dir := filepath.Dir(videoPath)

	for _, ext := range extensions {
		candidate := filepath.Join(base + ext)
		if _, err := os.Stat(candidate); err == nil {
			return candidate, nil
		}
	}
	return "", fmt.Errorf("no image found for %s (tried: %s)",
		filepath.Base(base), strings.Join(extensions, ", "))
}
