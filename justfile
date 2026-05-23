#set shell := ["bash", "-c"]

# ------------------------------------------------------------------------------
# Variables
# ------------------------------------------------------------------------------

bot_path := "backend/bot"
diy_binance_mirror_path := "backend/diy_binance_mirror"
binance_diy_path := "backend/binance_diy"
frontend_dir := "frontend"
go_proxy := "GOPROXY=https://mirror-go.runflare.com"

# ------------------------------------------------------------------------------
# Default
# ------------------------------------------------------------------------------

# Default target: List available commands
default:
    @just --list

# ------------------------------------------------------------------------------
# Development
# ------------------------------------------------------------------------------

# Add go dependencies in offline mode.
[group('Development')]
gooffline:
    go -C {{ bot_path }} mod download all
    go env -w GOPROXY=file://$HOME/go/pkg/mod/cache/download
    go env -w GOSUMDB=off

# Go online
[group('Development')]
goonline:
    go env -u GOPROXY

# Dependencies.
[group('Development')]
dep: gooffline
    cd {{ bot_path }} && go mod tidy

# Compile backend without building output binary.
[group('Development')]
check:
    cd {{ bot_path }} && go build ./...

# Build backend.
[group('Development')]
build:
    cd {{ bot_path }} && go build -o bin/goprintmoney cmd/server/main.go

# Run backend.
[group('Development')]
dev: build
    {{ bot_path }}/bin/goprintmoney

# Run treeclip with default flags.
[group('Development')]
[linux]
treeclip dir="":
    treeclip run {{ dir }} -f -t -v -c --stats

# ------------------------------------------------------------------------------
# Docker
# ------------------------------------------------------------------------------

# Start db.
[group('Docker')]
[linux]
db-up:
    docker compose up db -d

# Stop db.
[group('Docker')]
[linux]
db-down:
    docker compose down db

# ------------------------------------------------------------------------------
# Code Quality
# ------------------------------------------------------------------------------

# 🚨 Run lint checks.
[group('Code Quality')]
[linux]
lint:
    cd {{ bot_path }} && golangci-lint run

# 🚀 Conduct quality checks.
[group('Code Quality')]
[linux]
audit:
    go mod verify
    go vet ./...
    go run golang.org/x/vuln/cmd/govulncheck@latest ./...

# ----------------------------------------------------------------
# Dependency
# ----------------------------------------------------------------

[group('Dependency')]
vendor:
    cd {{ binance_diy_path }} && cargo vendor vendor --versioned-dirs --no-delete

[group('Dependency')]
vendor-clean:
    cd {{ binance_diy_path }} && rm -rf ./vendor

# ------------------------------------------------------------------------------
# Git
# ------------------------------------------------------------------------------

# Commit staged changes with amend.
[group('Git')]
amend:
    git commit -a --amend

[group('Git')]
empty:
    git commit --allow-empty

# Rebase current branch to the specified number of commits (Usage: just rebase 5)
[group('Git')]
rebase n="3":
    git rebase -i HEAD~{{ n }}

[group('Git')]
[linux]
diff-cp:
    git add -N .  # Intent-to-add for untracked files
    git diff HEAD | xclip -selection clipboard
    git reset  	  # Remove the intent-to-add markers

[group('Git')]
[windows]
diff-cp:
    git add -N .  # Intent-to-add for untracked files
    git diff HEAD | /c/Windows/System32/clip.exe
    git reset  	  # Remove the intent-to-add markers

[group('Git')]
today:
    git log --since="today 00:00:00" --until="today 23:59:59" --oneline
