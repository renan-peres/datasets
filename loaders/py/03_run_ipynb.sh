#!/bin/sh

# Check if virtual environment exists
if [ ! -d "venv" ]; then
    echo "Virtual environment not found. Please run 01_install_dependencies.sh first."
    exit 1
fi

# Activate virtual environment
echo "Activating virtual environment..."
. ./venv/bin/activate

# Verify virtual environment activation
if [ -z "$VIRTUAL_ENV" ] || [[ "$VIRTUAL_ENV" != *venv* ]]; then
    echo "Virtual environment not activated correctly. Aborting."
    exit 1
fi

# Install required packages if not already installed
echo "Checking required packages..."
python -m pip install --quiet jupyter nbconvert ipykernel

# Create necessary directories
echo "Creating directories..."
mkdir -p ipynb/output

# Set Python path
export PYTHONPATH="${PYTHONPATH}:$(pwd)"

# Initialize status tracking
failed_notebooks=0
total_notebooks=0

# Run all Jupyter notebooks in the ipynb directory
echo "Running all Jupyter notebooks in the 'ipynb' directory..."
for notebook in ipynb/*.ipynb; do
    if [ -f "$notebook" ]; then
        ((total_notebooks++))
        echo "Executing notebook: $notebook"
        if jupyter nbconvert --to notebook --execute --inplace "$notebook"; then
            echo "Successfully executed: $notebook"
        else
            echo "Failed to execute: $notebook"
            ((failed_notebooks++))
        fi
    else
        echo "No notebooks found in 'ipynb' directory."
        break
    fi
done

# Report final status
echo "Execution complete."
echo "Total notebooks processed: $total_notebooks"
echo "Failed notebooks: $failed_notebooks"

# Deactivate virtual environment
echo "Deactivating virtual environment..."
deactivate

# Exit with failure if any notebook failed
[ $failed_notebooks -eq 0 ] || exit 1