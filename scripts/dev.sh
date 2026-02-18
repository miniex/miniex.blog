#!/bin/bash

SESSION_NAME="miniex_blog"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

tmux kill-session -t $SESSION_NAME 2>/dev/null

tmux new-session -d -s $SESSION_NAME -c "$PROJECT_DIR" "bun dev"

tmux split-window -h -t $SESSION_NAME:0 -c "$PROJECT_DIR" "cargo watch -w src -w templates -w contents -x run"

tmux select-layout -t $SESSION_NAME:0 even-horizontal

tmux attach-session -t $SESSION_NAME

tmux kill-session -t $SESSION_NAME
