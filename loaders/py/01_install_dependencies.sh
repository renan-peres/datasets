#!/bin/bash

# Update PATH for this session
export PATH="$PATH:$HOME/.local/bin"

# Create virtual environment only if it doesn't exist
if [ ! -d "venv" ]; then
    echo "Creating new virtual environment..."
    python3 -m venv venv
fi

# Activate virtual environment
. ./venv/bin/activate

# Verify virtual environment activation
if ! echo "$VIRTUAL_ENV" | grep -q "venv"; then
    echo "Virtual environment not activated correctly"
    exit 1
fi

# Install requirements if they exist
if [ -f "requirements.txt" ]; then
    if ! pip install -r requirements.txt; then
        echo "Failed to install requirements"
        exit 1
    fi
fi

echo "All dependencies installed successfully"