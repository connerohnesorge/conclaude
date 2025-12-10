#!/bin/bash
# Script to download the mascot image from GitHub

set -e

MASCOT_URL="https://github.com/user-attachments/assets/2e1cc33a-5e49-4e1c-b575-f6ab5e16374f"
OUTPUT_DIR="docs/src/assets"
OUTPUT_FILE="$OUTPUT_DIR/mascot.png"

echo "Downloading mascot image..."
echo "Source: $MASCOT_URL"
echo "Destination: $OUTPUT_FILE"

# Download the image
curl -L "$MASCOT_URL" -o "$OUTPUT_FILE"

# Verify the download
if [ -f "$OUTPUT_FILE" ]; then
    FILE_SIZE=$(wc -c < "$OUTPUT_FILE")
    echo "✓ Successfully downloaded mascot image ($FILE_SIZE bytes)"
    
    # Check if it's actually a PNG
    if file "$OUTPUT_FILE" | grep -q "PNG image data"; then
        echo "✓ File is a valid PNG image"
        
        # Remove the SVG placeholder if download was successful
        if [ -f "$OUTPUT_DIR/mascot.svg" ]; then
            echo "Removing SVG placeholder..."
            rm "$OUTPUT_DIR/mascot.svg"
        fi
        
        # Update the index.mdx to use PNG instead of SVG
        if [ -f "docs/src/content/docs/index.mdx" ]; then
            echo "Updating index.mdx to use PNG image..."
            sed -i.bak 's|../../assets/mascot\.svg|../../assets/mascot.png|' docs/src/content/docs/index.mdx
        fi
        
        # Update README.md to use PNG instead of SVG
        if [ -f "README.md" ]; then
            echo "Updating README.md to use PNG image..."
            sed -i.bak 's|./docs/src/assets/mascot\.svg|./docs/src/assets/mascot.png|' README.md
        fi
        
        echo ""
        echo "✓ Mascot image successfully installed!"
        echo "  You can now commit the changes:"
        echo "  git add docs/src/assets/mascot.png docs/src/content/docs/index.mdx README.md"
        echo "  git commit -m 'Replace mascot placeholder with actual image'"
    else
        echo "✗ Downloaded file is not a valid PNG image"
        exit 1
    fi
else
    echo "✗ Failed to download mascot image"
    exit 1
fi
