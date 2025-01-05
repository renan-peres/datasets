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

# Run all Jupyter notebooks in the ipynb directory
echo "Running all Jupyter notebooks in the 'ipynb' directory..."
for notebook in ipynb/*.ipynb; do
    if [ -f "$notebook" ]; then
        echo "Executing notebook: $notebook"
        jupyter nbconvert --to notebook --execute "$notebook" --output-dir ./ipynb/output/
        if [ $? -ne 0 ]; then
            echo "Error occurred while executing notebook: $notebook"
            deactivate
            exit 1
        fi
    else
        echo "No notebooks found in 'ipynb' directory."
        break
    fi
done

# Deactivate virtual environment
echo "Execution complete. Deactivating virtual environment..."
deactivate