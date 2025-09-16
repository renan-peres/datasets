#!/bin/bash
cd ../
# Check if virtual environment exists
if [ ! -d ".venv" ]; then
    echo "Virtual environment not found. Please run 01_install_dependencies.sh first."
    exit 1
fi

# Activate virtual environment
echo "Activating virtual environment..."
. ./.venv/bin/activate

# Verify virtual environment activation
if [ -z "$VIRTUAL_ENV" ] || [[ "$VIRTUAL_ENV" != *venv* ]]; then
    echo "Virtual environment not activated correctly. Aborting."
    exit 1
fi

# Load environment variables from .env if present (non-fatal if missing)
if [ -f .env ]; then
    echo "Sourcing .env file (whitespace-normalized)..."
    tmp_env_file="$(mktemp)"
    # Normalize VAR = value -> VAR=value and strip Windows CR plus blank/comment lines
    sed -E 's/^([A-Za-z_][A-Za-z0-9_]*)[[:space:]]*=[[:space:]]*/\1=/' .env | \
      sed -E 's/\r$//' | \
      grep -v -E '^[[:space:]]*#' | \
      grep -v -E '^[[:space:]]*$' > "$tmp_env_file"
    set -a
    . "$tmp_env_file"
    set +a
    rm -f "$tmp_env_file"
else
    echo ".env file not found (continuing)."
fi

# Ensure required API key exists
if [ -z "$FMP_API_KEY" ]; then
    echo "ERROR: FMP_API_KEY is not set."
    echo "Add a line like: FMP_API_KEY=your_key_here to the .env file (no quotes) or export it in your shell before running."
    deactivate 2>/dev/null || true
    exit 1
fi

# Quick validation of API key (lightweight endpoint)
echo "Validating FMP_API_KEY..."
validation_resp=$(curl -s "https://financialmodelingprep.com/stable/profile-bulk?part=0&apikey=${FMP_API_KEY}" | head -n 2)
if echo "$validation_resp" | grep -qi "Invalid API KEY"; then
    echo "ERROR: The provided FMP_API_KEY appears invalid. Response:"
    echo "$validation_resp"
    echo "Visit: https://site.financialmodelingprep.com/ to create/retrieve a valid key, then update your .env."
    deactivate 2>/dev/null || true
    exit 1
fi

# Install required packages if not already installed
echo "Checking required packages..."
uv pip install --upgrade -r requirements.txt --prerelease=allow

# Set Python path
export PYTHONPATH="${PYTHONPATH}:$(pwd)"

# Initialize status tracking
failed_notebooks=0
total_notebooks=0

# Define the notebooks to run in order
notebooks=(
    "data/loaders/01_fmp_fetch_americas.ipynb"
    "data/loaders/02_load_to_s3.ipynb"
)

# Run the specified notebooks in order
echo "Running specified notebooks in order..."
for notebook in "${notebooks[@]}"; do
    if [ -f "$notebook" ]; then
        ((total_notebooks++))
        echo "Executing notebook: $notebook"
        if jupyter nbconvert --to notebook --execute --inplace "$notebook"; then
            echo "Successfully executed: $notebook"
        else
            echo "Failed to execute: $notebook"
            ((failed_notebooks++))
            # Exit immediately if a notebook fails to maintain execution order
            echo "Stopping execution due to notebook failure."
            break
        fi
    else
        echo "Notebook not found: $notebook"
        ((failed_notebooks++))
        echo "Stopping execution due to missing notebook."
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