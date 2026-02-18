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

# Ping search engines for indexing
echo "Waiting for container to be ready..."
sleep 5

INDEXNOW_BODY='{
  "host": "miniex.blog",
  "key": "0de3222f-6b9a-4e62-9d8e-5d98d960963e",
  "keyLocation": "https://miniex.blog/assets/indexnow-key.txt",
  "urlList": ["https://miniex.blog/sitemap.xml"]
}'

echo "Pinging Google Sitemap..."
curl -s "https://www.google.com/ping?sitemap=https://miniex.blog/sitemap.xml" || true

echo "Pinging IndexNow (Bing)..."
curl -s -X POST "https://www.bing.com/indexnow" \
  -H "Content-Type: application/json" -d "$INDEXNOW_BODY" || true

echo "Pinging IndexNow (Naver)..."
curl -s -X POST "https://searchadvisor.naver.com/indexnow" \
  -H "Content-Type: application/json" -d "$INDEXNOW_BODY" || true

echo "Pinging IndexNow (Yandex)..."
curl -s -X POST "https://yandex.com/indexnow" \
  -H "Content-Type: application/json" -d "$INDEXNOW_BODY" || true

echo "Pinging IndexNow (Seznam)..."
curl -s -X POST "https://search.seznam.cz/indexnow" \
  -H "Content-Type: application/json" -d "$INDEXNOW_BODY" || true

echo "Search engine ping complete."

# Remove the key from the agent and kill the agent
ssh-add -D
eval $(ssh-agent -k)
