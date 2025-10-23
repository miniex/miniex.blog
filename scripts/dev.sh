#!/bin/bash

SESSION_NAME="miniex_blog"

tmux kill-session -t $SESSION_NAME 2>/dev/null

tmux new-session -d -s $SESSION_NAME "bun dev"

tmux split-window -h -t $SESSION_NAME:0 "cargo watch -w src -w contents -x run" 

tmux select-layout -t $SESSION_NAME:0 even-horizontal

tmux attach-session -t $SESSION_NAME

tmux kill-session -t $SESSION_NAME
