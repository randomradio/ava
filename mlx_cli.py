#!/usr/bin/env python3
"""
Simple CLI for MLX Whisper transcription with pause/resume functionality.
"""

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path
import signal

class TranscriptionCheckpoint:
    def __init__(self, video_path, output_dir):
        self.video_path = str(Path(video_path).resolve())
        self.output_dir = str(Path(output_dir).resolve())
        self.checkpoint_file = Path(output_dir) / f"{Path(video_path).stem}_checkpoint.json"
        self.transcription = []
        self.screenshots = []
        self.current_time = 0
        self.status = "not_started"
        
    def load(self):
        """Load checkpoint if it exists"""
        if self.checkpoint_file.exists():
            try:
                with open(self.checkpoint_file, 'r') as f:
                    data = json.load(f)
                    self.transcription = data.get('transcription', [])
                    self.screenshots = data.get('screenshots', [])
                    self.current_time = data.get('current_time', 0)
                    self.status = data.get('status', 'not_started')
                    return True
            except Exception as e:
                print(f"Warning: Could not load checkpoint: {e}")
        return False
    
    def save(self):
        """Save current progress"""
        os.makedirs(self.output_dir, exist_ok=True)
        data = {
            'video_path': self.video_path,
            'transcription': self.transcription,
            'screenshots': self.screenshots,
            'current_time': self.current_time,
            'status': self.status
        }
        try:
            with open(self.checkpoint_file, 'w') as f:
                json.dump(data, f, indent=2)
            print(f"Checkpoint saved: {self.checkpoint_file}")
        except Exception as e:
            print(f"Error saving checkpoint: {e}")
    
    def mark_complete(self):
        """Mark transcription as complete"""
        self.status = "completed"
        self.save()

class MLXTranscriber:
    def __init__(self, checkpoint):
        self.checkpoint = checkpoint
        self.interrupted = False
        
    def setup_signal_handlers(self):
        """Setup graceful shutdown on Ctrl+C"""
        def signal_handler(signum, frame):
            print("\nReceived interrupt signal. Saving progress...")
            self.interrupted = True
            
        signal.signal(signal.SIGINT, signal_handler)
        signal.signal(signal.SIGTERM, signal_handler)
    
    def get_video_duration(self, video_path):
        """Get video duration in seconds"""
        try:
            result = subprocess.run([
                'ffprobe', '-v', 'error', '-show_entries', 
                'format=duration', '-of', 'default=noprint_wrappers=1:nokey=1',
                video_path
            ], capture_output=True, text=True)
            return float(result.stdout.strip())
        except Exception as e:
            print(f"Warning: Could not get video duration: {e}")
            return 3600  # Default 1 hour
    
    def extract_screenshot(self, video_path, timestamp, output_path):
        """Extract screenshot at specific timestamp"""
        try:
            cmd = [
                'ffmpeg', '-ss', str(timestamp), '-i', video_path,
                '-vframes', '1', '-q:v', '2', '-y', output_path
            ]
            subprocess.run(cmd, capture_output=True, check=True)
            return True
        except subprocess.CalledProcessError as e:
            print(f"Error extracting screenshot: {e}")
            return False
    
    def run_mlx_whisper(self, video_path, start_time=0):
        """Run MLX Whisper on video segment"""
        try:
            cmd = [
                "/Users/randomradio/src/ava/.venv/bin/mlx_whisper",
                video_path,
                "--output-format", "json"
            ]
            
            result = subprocess.run(cmd, capture_output=True, text=True)
            
            if result.returncode != 0:
                raise Exception(f"MLX Whisper failed: {result.stderr}")
            
            # Parse JSON output
            lines = result.stdout.strip().split('\n')
            json_start = None
            for i, line in enumerate(lines):
                if line.strip().startswith('{'):
                    json_start = i
                    break
            
            if json_start is None:
                # Try to read from file that mlx_whisper creates
                output_file = Path(video_path).with_suffix('.json')
                if output_file.exists():
                    with open(output_file, 'r') as f:
                        return json.load(f)
                raise Exception("No JSON found in MLX Whisper output")
            
            json_str = '\n'.join(lines[json_start:])
            return json.loads(json_str)
            
        except Exception as e:
            print(f"Error running MLX Whisper: {e}")
            return {"segments": []}
    
    def process_video(self):
        """Process video from checkpoint or start"""
        print(f"Processing: {self.checkpoint.video_path}")
        
        if self.checkpoint.load() and self.checkpoint.status == "completed":
            print("Video already processed. Use --force to reprocess.")
            return
        
        self.setup_signal_handlers()
        
        if self.checkpoint.transcription:
            print(f"Resuming from {self.checkpoint.current_time:.1f}s")
        else:
            print("Starting new transcription...")
        
        duration = self.get_video_duration(self.checkpoint.video_path)
        print(f"Video duration: {duration:.1f}s")
        
        chunk_size = 300  # 5 minute chunks
        start_time = self.checkpoint.current_time
        
        while start_time < duration and not self.interrupted:
            print(f"Processing chunk: {start_time:.1f}s - {min(start_time + chunk_size, duration):.1f}s")
            
            try:
                result = self.run_mlx_whisper(self.checkpoint.video_path, start_time)
                
                for segment in result.get('segments', []):
                    if self.interrupted:
                        break
                    
                    segment_start = segment.get('start', 0) + start_time
                    segment_end = segment.get('end', 0) + start_time
                    text = segment.get('text', '').strip()
                    
                    if text:
                        segment_data = {
                            'start': segment_start,
                            'end': segment_end,
                            'text': text
                        }
                        self.checkpoint.transcription.append(segment_data)
                        
                        # Generate screenshot for significant segments
                        if len(text) > 20 and segment_start > start_time:
                            screenshot_name = f"screenshot_{len(self.checkpoint.screenshots):04d}.jpg"
                            screenshot_path = Path(self.checkpoint.output_dir) / screenshot_name
                            
                            if self.extract_screenshot(
                                self.checkpoint.video_path, 
                                segment_start, 
                                str(screenshot_path)
                            ):
                                self.checkpoint.screenshots.append({
                                    'timestamp': segment_start,
                                    'file': str(screenshot_path),
                                    'text': text[:50] + "..." if len(text) > 50 else text
                                })
                
                start_time += chunk_size
                self.checkpoint.current_time = start_time
                self.checkpoint.status = "processing"
                self.checkpoint.save()
                
            except Exception as e:
                print(f"Error processing chunk: {e}")
                break
        
        if not self.interrupted:
            self.checkpoint.mark_complete()
            print("Transcription completed!")
        else:
            print(f"Transcription paused at {start_time:.1f}s")

def main():
    parser = argparse.ArgumentParser(description="MLX Whisper CLI with pause/resume")
    parser.add_argument("video", help="Path to video file")
    parser.add_argument("-o", "--output", default="./output", help="Output directory")
    parser.add_argument("-f", "--force", action="store_true", help="Force reprocessing")
    parser.add_argument("--resume", help="Resume from checkpoint file")
    
    args = parser.parse_args()
    
    if not os.path.exists(args.video):
        print(f"Error: Video file not found: {args.video}")
        sys.exit(1)
    
    if args.resume:
        # Resume from specific checkpoint
        checkpoint_path = Path(args.resume)
        if not checkpoint_path.exists():
            print(f"Error: Checkpoint file not found: {args.resume}")
            sys.exit(1)
        
        output_dir = checkpoint_path.parent
        with open(checkpoint_path, 'r') as f:
            data = json.load(f)
            video_path = data['video_path']
    else:
        output_dir = args.output
        video_path = args.video
    
    checkpoint = TranscriptionCheckpoint(video_path, output_dir)
    
    if args.force:
        checkpoint.status = "not_started"
        checkpoint.transcription = []
        checkpoint.screenshots = []
        checkpoint.current_time = 0
        # Remove existing checkpoint file to force reprocessing
        if checkpoint.checkpoint_file.exists():
            checkpoint.checkpoint_file.unlink()
    
    transcriber = MLXTranscriber(checkpoint)
    transcriber.process_video()

if __name__ == "__main__":
    main()
