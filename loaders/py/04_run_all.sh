#!/bin/bash

# Activate virtual environment
if [ ! -d "venv" ]; then
    echo "Virtual environment not found. Please run 01_install_dependencies.sh first."
    exit 1
fi

. ./venv/bin/activate

# Verify virtual environment activation
if ! echo "$VIRTUAL_ENV" | grep -q "venv"; then
    echo "Virtual environment not activated correctly"
    exit 1
fi

# Run Python scripts
echo "Running Python scripts..."
sh 02_run_python_scripts.sh

# Run Jupyter notebooks
echo "Running Jupyter notebooks..."
sh 03_run_ipynb.sh

# Deactivate virtual environment
deactivate

echo "All tasks execution completed"