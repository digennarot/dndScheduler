#!/bin/bash
set -e

echo "🚀 Deploying D&D Scheduler..."

# Ensure data directory exists
mkdir -p data

# Check if .env exists
if [ ! -f .env ]; then
    echo "⚠️  .env file not found! Copying from .env.example..."
    cp .env.example .env
    echo "Please edit .env with your secrets."
fi

# Determine container runtime
if command -v podman-compose &> /dev/null; then
    COMPOSE="podman-compose"
    echo "🐳 Using Podman..."
elif command -v docker-compose &> /dev/null; then
    COMPOSE="docker-compose"
    echo "🐳 Using Docker..."
else
    COMPOSE="docker compose"
fi

# Build and Start
echo "📦 Building and starting containers..."
$COMPOSE up -d --build

echo "✅ Deployment complete!"
echo "🌍 App available at http://localhost"
