// Package main. main.go - It's the entry point for the vidwrap tool.
// It delegates all execution to the cmd package to maintain
// a clean separation between bootstrapping and business logic.
package main

import (
	"os"

	"vidwrap/cmd"
)

// main initializes the application and handles top-level exit codes.
func main() {
	if err := cmd.Execute(); err != nil {
		os.Exit(1)
	}
}
