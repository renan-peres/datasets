#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Determine virtual environment directory (.venv preferred, fallback venv)
VENV_DIR=""
if [ -d "${REPO_ROOT}/.venv" ]; then
    VENV_DIR="${REPO_ROOT}/.venv"
elif [ -d "${REPO_ROOT}/venv" ]; then
    VENV_DIR="${REPO_ROOT}/venv"
else
    echo "[ERROR] Virtual environment not found (.venv or venv). Run orchestrate/01_install_dependencies.sh first." >&2
    exit 1
fi

echo "[INFO] Using virtual environment at: ${VENV_DIR}"
. "${VENV_DIR}/bin/activate"

if [ -z "${VIRTUAL_ENV:-}" ]; then
    echo "[ERROR] Virtual environment failed to activate." >&2
    exit 1
fi

echo "[INFO] Python: $(python --version 2>&1)"
echo "[INFO] Which python: $(command -v python)"

echo "[INFO] Running Jupyter notebook execution script..."
bash "${SCRIPT_DIR}/02_run_ipynb.sh"

echo "[INFO] Deactivating virtual environment"
deactivate || true

echo "[SUCCESS] All tasks execution completed"