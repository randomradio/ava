#!/bin/bash
# Setup script for MLX Whisper on macOS

set -e

echo "Setting up MLX Whisper for AVA..."

# Check if Python3 is available
if ! command -v python3 &> /dev/null; then
    echo "Error: Python3 is required but not installed. Please install Python3."
    exit 1
fi

# Install mlx-whisper
echo "Installing mlx-whisper..."
python3 -m pip install mlx-whisper

# Copy the MLX Whisper script to the correct location
echo "Copying MLX Whisper script..."
cp src-tauri/transcribe.py . 2>/dev/null || true
cp transcribe.py src-tauri/ 2>/dev/null || true

echo "MLX Whisper setup complete!"
echo "You can now run: npm run tauri dev"