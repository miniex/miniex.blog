#!/bin/bash

SESSION_NAME="miniex_blog"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DEV_PORT=3000

cleanup() {
    tmux kill-session -t "$SESSION_NAME" 2>/dev/null
    lsof -ti:"$DEV_PORT" | xargs kill -9 2>/dev/null
}

trap cleanup EXIT

# Kill existing session and port
tmux kill-session -t "$SESSION_NAME" 2>/dev/null
lsof -ti:"$DEV_PORT" | xargs kill -9 2>/dev/null
sleep 0.3

# Pane 0: Tailwind CSS watch
tmux new-session -d -s "$SESSION_NAME" -c "$PROJECT_DIR" "bun dev"

# Pane 1: Cargo watch (restart is default — kills old server before starting new one)
tmux split-window -h -t "$SESSION_NAME:0" -c "$PROJECT_DIR" \
    "cargo watch -w src -w templates -w contents -x run"

tmux select-layout -t "$SESSION_NAME:0" even-horizontal
tmux attach-session -t "$SESSION_NAME"
