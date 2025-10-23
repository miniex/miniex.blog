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
git pull origin main

# Ensure data directory exists with proper permissions
mkdir -p data
chmod 777 data

# Set default RESUME_TAG if not provided
: ${RESUME_TAG:=default-secret-tag}

# Force rebuild of Docker images
RESUME_TAG=$RESUME_TAG docker compose -f $COMPOSE_FILE build --no-cache

# Start the services
RESUME_TAG=$RESUME_TAG docker compose -f $COMPOSE_FILE up -d

# Remove old, unused images
docker image prune -f

# Remove the key from the agent and kill the agent
ssh-add -D
eval $(ssh-agent -k)
