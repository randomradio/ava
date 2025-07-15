# AVA - Video Transcription Tool Setup

## Overview
AVA is a Tauri-based desktop application that processes video files to:
1. Extract and transcribe audio using OpenRouter's Whisper API
2. Analyze transcriptions to identify screenshot-worthy moments
3. Capture screenshots at identified timestamps
4. Generate captions for screenshots using OpenRouter's vision models

## Prerequisites

### 1. Install FFmpeg
The application requires FFmpeg to be installed and available in your system PATH.

**macOS:**
```bash
brew install ffmpeg
```

**Windows:**
Download from https://ffmpeg.org/download.html and add to PATH

**Linux:**
```bash
sudo apt install ffmpeg  # Ubuntu/Debian
sudo yum install ffmpeg  # CentOS/RHEL
```

### 2. OpenRouter API Key
Get your API key from https://openrouter.ai/

### 3. Rust and Node.js
Make sure you have Rust and Node.js installed (required for Tauri development).

## Running the Application

### Development Mode
```bash
npm run tauri dev
```

### Building for Production
```bash
npm run tauri build
```

## Usage

1. **Configure API Key**: Enter your OpenRouter API key in the configuration section
2. **Select Video**: Click "Select Video File" to choose a video file
3. **Process Video**: Click "Process Video" to start the transcription and screenshot process
4. **View Results**: The transcription will appear with timestamps, followed by screenshots with captions

## Features

### Core Functionality
- ✅ Video file selection with support for multiple formats (MP4, AVI, MOV, MKV, WMV, FLV, WebM)
- ✅ Audio extraction from video files using FFmpeg
- ✅ OpenRouter integration for Whisper transcription with timestamps
- ✅ LLM analysis to determine screenshot-worthy moments
- ✅ Automated screenshot capture at identified timestamps
- ✅ Image captioning using OpenRouter's vision models
- ✅ Modern UI with shadcn/ui design system
- ✅ Progress tracking and error handling

### Technical Implementation
- **Backend**: Rust with Tauri framework
- **Frontend**: Vanilla JavaScript with modern CSS
- **API Integration**: OpenRouter for Whisper (audio transcription) and GPT-4 Vision (image captioning)
- **Video Processing**: FFmpeg for audio extraction and screenshot capture
- **UI Design**: shadcn/ui design system with CSS custom properties

## Testing

### Test Data
Place test video files in the `test_data/` directory. Ideal test videos should have:
- Clear spoken content
- Visual elements worth capturing (charts, code, presentations)
- Duration of 1-5 minutes for quick testing

### Test Cases
1. **Educational Video**: Tutorial with visual demonstrations
2. **Presentation**: Slides or charts that benefit from screenshots
3. **Code Tutorial**: Screen recordings with code explanations

## Architecture

### Rust Backend (`src-tauri/`)
- `extract_audio_from_video`: Uses FFmpeg to extract audio as WAV
- `transcribe_audio_openrouter`: Sends audio to OpenRouter Whisper API
- `analyze_transcription_for_screenshots`: Uses GPT-4 to identify screenshot moments
- `capture_screenshot`: Uses FFmpeg to capture frames at specific timestamps
- `caption_image_openrouter`: Uses GPT-4 Vision to caption screenshots
- `process_video_complete`: Orchestrates the entire workflow

### Frontend (`src/`)
- Modern UI with file selection, progress tracking, and results display
- Real-time progress updates during processing
- Responsive design with light/dark mode support
- Error handling and user feedback

## Troubleshooting

### Common Issues
1. **FFmpeg not found**: Ensure FFmpeg is installed and in PATH
2. **API errors**: Verify OpenRouter API key is correct and has sufficient credits
3. **File access**: Ensure video files are accessible and not corrupted
4. **Build errors**: Run `cargo clean` and try building again

### Development
- Use `cargo check` to verify Rust code compilation
- Check browser console for JavaScript errors
- Test with different video formats and sizes
- Monitor API usage and costs

## Contributing

The application is structured for easy extension:
- Add new video formats by updating file filters
- Extend screenshot analysis with different LLM prompts
- Add export functionality for results
- Implement batch processing for multiple videos