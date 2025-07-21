# MLX Transcription Tool

A simple, efficient tool for transcribing videos using MLX Whisper with automatic screenshot generation.

## Quick Start

### Option 1: Web Interface (Recommended)
```bash
streamlit run streamlit_app.py
```
Then open http://localhost:8501 in your browser.

### Option 2: Command Line
```bash
# Basic usage
python mlx_cli.py video.mp4

# Force reprocessing
python mlx_cli.py video.mp4 --force

# Custom output directory
python mlx_cli.py video.mp4 -o ./my-output
```

## Features

- **🎯 MLX Whisper**: Native macOS transcription using Apple Silicon
- **📸 Screenshots**: Auto-generated at key moments
- **⏸️ Pause/Resume**: Interrupt anytime, continue later
- **📊 Progress Tracking**: Real-time status in web UI
- **📁 File Management**: Easy upload/download
- **🔄 JSON Export**: Structured data output

## Output

Creates `output/[video_name]/` with:
- `video_checkpoint.json`: Complete transcription + metadata
- `screenshot_*.jpg`: Screenshots at significant moments
- **CSV/JSON Export**: Available in web interface

## Requirements

- macOS with Apple Silicon
- Python 3.11+
- MLX Whisper (installed via pip: `mlx-whisper`)
- FFmpeg (for screenshots)

## Installation

```bash
pip install -r requirements.txt
```

## Usage Examples

### Web Interface
1. Run `streamlit run streamlit_app.py`
2. Upload video via drag-and-drop
3. Click "Start Transcription"
4. Download results when complete

### Command Line
```bash
# Process single video
python mlx_cli.py presentation.mp4

# Resume from checkpoint
python mlx_cli.py --resume output/presentation_checkpoint.json
```

## Architecture

- **Core**: `mlx_cli.py` - CLI transcription engine
- **Web**: `streamlit_app.py` - Browser interface
- **Shared**: JSON checkpoint system
