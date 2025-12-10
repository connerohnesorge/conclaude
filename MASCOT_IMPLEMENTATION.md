# Mascot Image Implementation

## Status: Partially Complete

### ✅ Completed Tasks

1. **Documentation Site (index.mdx)**
   - Added mascot image to the hero section of the documentation splash page
   - File: `docs/src/content/docs/index.mdx`
   - Location in frontmatter: `hero.image.file: ../../assets/mascot.svg`

2. **README.md**
   - Added centered mascot image at the top of the README
   - File: `README.md`
   - Image is centered with width="200"

3. **Asset Files**
   - Created SVG placeholder: `docs/src/assets/mascot.svg`
   - Created empty PNG file: `docs/src/assets/mascot.png`
   - Added README with instructions: `docs/src/assets/README.md`

4. **Download Script**
   - Created automated download script: `scripts/download-mascot.sh`
   - Script will download, verify, and update file references automatically

### ⚠️ Remaining Tasks

1. **Download Actual Mascot Image**
   - The current implementation uses an SVG placeholder
   - The actual PNG image needs to be downloaded from: https://github.com/user-attachments/assets/2e1cc33a-5e49-4e1c-b575-f6ab5e16374f
   - **Action Required**: Run `./scripts/download-mascot.sh` from the repository root

2. **Verify Documentation Build**
   - Test that the documentation site builds successfully with the mascot
   - **Action**: Run `cd docs && npm run build`
   - **Note**: Currently blocked by GitHub API rate limit for changelog plugin (unrelated issue)

## Quick Start

To complete the implementation:

```bash
# Step 1: Download the actual mascot image
./scripts/download-mascot.sh

# Step 2: Verify the changes
git status
git diff

# Step 3: Test the documentation site
cd docs
npm install --legacy-peer-deps
npm run dev

# Step 4: Commit the changes
git add docs/src/assets/mascot.png docs/src/content/docs/index.mdx README.md
git commit -m "Replace mascot placeholder with actual image"
```

## Technical Details

### Why SVG Placeholder?

The actual mascot image could not be downloaded during implementation due to network restrictions in the sandbox environment:
- GitHub asset URLs redirect to AWS S3
- The S3 hostname `github-production-user-asset-6210df.s3.amazonaws.com` could not be resolved
- A placeholder SVG was created to allow development to continue

### File Structure

```
docs/src/assets/
├── README.md          # Instructions for downloading mascot
├── mascot.svg         # Placeholder (to be replaced)
├── mascot.png         # Empty (to be replaced by download script)
└── houston.webp       # Original Starlight mascot

docs/src/content/docs/
└── index.mdx          # Updated with mascot in hero section

README.md              # Updated with mascot at top

scripts/
└── download-mascot.sh # Automated download and update script
```

### Image Usage

**In Documentation (Astro/Starlight):**
```yaml
hero:
  image:
    file: ../../assets/mascot.svg  # Will be changed to mascot.png by script
```

**In README.md:**
```html
<p align="center">
  <img src="./docs/src/assets/mascot.svg" alt="conclaude mascot" width="200"/>
</p>
```

## Troubleshooting

### If download script fails

Try manual download:
```bash
cd docs/src/assets
curl -L "https://github.com/user-attachments/assets/2e1cc33a-5e49-4e1c-b575-f6ab5e16374f" -o mascot.png

# Verify it's a valid PNG
file mascot.png

# Update references manually
sed -i 's/mascot\.svg/mascot.png/' docs/src/content/docs/index.mdx
sed -i 's/mascot\.svg/mascot.png/' README.md
```

### If documentation build fails

The build may fail due to:
1. GitHub API rate limits (changelog plugin) - This is a separate issue
2. Peer dependency conflicts - Use `npm install --legacy-peer-deps`

## Next Steps

After downloading the actual mascot:
1. Remove the SVG placeholder file
2. Verify the image displays correctly in both README and docs site
3. Take screenshots for PR description
4. Complete the PR and merge
