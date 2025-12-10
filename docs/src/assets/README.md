# Assets Directory

## Mascot Image

The `mascot.svg` file is currently a placeholder. To download and install the actual mascot image, run:

```bash
# From the repository root
./scripts/download-mascot.sh
```

This script will:
1. Download the mascot image from: https://github.com/user-attachments/assets/2e1cc33a-5e49-4e1c-b575-f6ab5e16374f
2. Save it as `mascot.png`
3. Remove the SVG placeholder
4. Update references in `index.mdx` and `README.md`

The mascot is an orange/coral colored cartoon crab character that represents the conclaude project.

### Manual Download (alternative)

If the script doesn't work, you can manually download:

```bash
cd docs/src/assets
curl -L "https://github.com/user-attachments/assets/2e1cc33a-5e49-4e1c-b575-f6ab5e16374f" -o mascot.png
```

Then update the file references:
- In `docs/src/content/docs/index.mdx`: Change `mascot.svg` to `mascot.png`
- In `README.md`: Change `mascot.svg` to `mascot.png`
