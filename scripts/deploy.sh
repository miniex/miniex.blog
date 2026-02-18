#!/bin/bash
set -e

REPO_PATH="/opt/daminstudio/apps/m0000-blog"
COMPOSE_FILE="docker-compose.yml"

# Start SSH agent and add the private key
eval $(ssh-agent -s)
ssh-add ~/.ssh/gitlab

# Add GitLab host key to known_hosts
ssh-keyscan gitlab.daminstudio.com >> ~/.ssh/known_hosts

cd $REPO_PATH
# Save current HEAD before updating
OLD_HEAD=$(git rev-parse HEAD)

# Fetch latest changes and reset to match remote main branch
git fetch origin main
git reset --hard origin/main

NEW_HEAD=$(git rev-parse HEAD)

# Ensure data directory exists with proper permissions
mkdir -p data
chmod 777 data

# Copy resume.mdx from /tmp if it exists
if [ -f "/tmp/resume.mdx" ]; then
    mkdir -p contents
    cp /tmp/resume.mdx contents/resume.mdx
fi

# Set default RESUME_TAG if not provided
: ${RESUME_TAG:=default-secret-tag}
: ${RESUME_TITLE:=miniex::resume}

# Check if any files outside content/assets/templates/docs changed
CODE_CHANGED=$(git diff --name-only "$OLD_HEAD" "$NEW_HEAD" -- \
  ':!contents/' ':!assets/' ':!*.md' ':!LICENSE' | head -1)

if [ -n "$CODE_CHANGED" ]; then
  # Code changed: full rebuild
  echo "Code files changed, rebuilding Docker images..."
  RESUME_TAG=$RESUME_TAG RESUME_TITLE=$RESUME_TITLE docker compose -f $COMPOSE_FILE build --no-cache
  RESUME_TAG=$RESUME_TAG RESUME_TITLE=$RESUME_TITLE docker compose -f $COMPOSE_FILE up -d
  docker image prune -f
else
  # Content-only change: restart to pick up volume-mounted files
  echo "Content-only change, restarting containers..."
  RESUME_TAG=$RESUME_TAG RESUME_TITLE=$RESUME_TITLE docker compose -f $COMPOSE_FILE restart
fi

# Remove the key from the agent and kill the agent
ssh-add -D
eval $(ssh-agent -k)
