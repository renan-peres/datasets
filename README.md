# Datasets

### Installation

```bash
# Clone the repository
git clone https://github.com/renan-peres/mfin-quant-portfolio-options.git
cd mfin-quant-portfolio-options

# Set up environment variables
cp .env.example .env
# Edit .env with your API keys

# Create virtual environment using UV (recommended)
curl -LsSf https://astral.sh/uv/install.sh | env INSTALLER_NO_MODIFY_PATH=1 sh
uv venv
source .venv/bin/activate

# Install dependencies
uv pip install -r requirements.txt --prerelease=allow
```