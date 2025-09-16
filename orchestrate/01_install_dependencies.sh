#!/bin/bash

set -euo pipefail

echo "[INFO] Starting dependency installation with uv"

# Ensure local bin is on PATH (uv installer places binary here by default)
export PATH="$HOME/.local/bin:$PATH"

# Install uv if not present
if ! command -v uv >/dev/null 2>&1; then
    echo "[INFO] uv not found. Installing..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
    # Re-export PATH in case installer added a new location
    export PATH="$HOME/.local/bin:$PATH"
fi

echo "[INFO] uv version: $(uv --version)"

# Create virtual environment if it doesn't exist
cd ../
if [ ! -d "venv" ]; then
    echo "[INFO] Creating new virtual environment with uv"
    uv venv
else
    echo "[INFO] Reusing existing virtual environment"
fi

# Activate virtual environment
. ./.venv/bin/activate

if [ -z "${VIRTUAL_ENV:-}" ]; then
    echo "[ERROR] Virtual environment not activated"
    exit 1
fi
echo "[INFO] Virtual environment: $VIRTUAL_ENV"

# Use uv's pip wrapper for faster installs inside the venv
if [ -f "requirements.txt" ]; then
    echo "[INFO] Installing Python dependencies from requirements.txt"
    # "uv pip install" will build/download wheels efficiently; add --upgrade to refresh existing pins
    uv pip install --upgrade -r requirements.txt --prerelease=allow
else
    echo "[WARN] requirements.txt not found, skipping Python dependency installation"
fi

echo "[SUCCESS] All dependencies installed successfully via uv"