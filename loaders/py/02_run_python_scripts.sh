#!/bin/bash

# Check if virtual environment exists
if [ ! -d "venv" ]; then
    echo "Virtual environment not found. Please run 01_install_dependencies.sh first."
    exit 1
fi

# Activate virtual environment
. ./venv/bin/activate

# Verify virtual environment activation
if ! echo "$VIRTUAL_ENV" | grep -q "venv"; then
    echo "Virtual environment not activated correctly"
    exit 1
fi

# Run all Python scripts in the scripts directory with parent directory in PYTHONPATH
echo "Running all Python scripts..."
export PYTHONPATH="${PYTHONPATH}:$(pwd)"
for script in scripts/*.py; do
    if [ "$(basename "$script")" = "make_clean_names.py" ]; then
        echo "Skipping $script..."
        continue
    fi
    if [ -f "$script" ]; then
        echo "Executing $script..."
        python3 "$script"
    fi
done

# Deactivate virtual environment
deactivate

echo "All scripts execution completed"