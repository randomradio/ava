#!/usr/bin/env python3
"""
Unit tests for transcribe.py MLX Whisper script
"""

import json
import os
import sys
import tempfile
import unittest
from unittest.mock import patch, MagicMock
from io import StringIO

# Add the directory containing transcribe.py to the path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

try:
    import transcribe
except ImportError as e:
    print(f"Error importing transcribe: {e}")
    sys.exit(1)


class TestTranscribe(unittest.TestCase):
    def setUp(self):
        """Set up test fixtures."""
        self.test_audio_path = "test_audio.wav"
        self.temp_dir = tempfile.mkdtemp()
        self.test_audio_file = os.path.join(self.temp_dir, "test.wav")
        
        # Create a dummy audio file for testing file existence
        with open(self.test_audio_file, "wb") as f:
            f.write(b"dummy audio data")

    def tearDown(self):
        """Clean up test fixtures."""
        if os.path.exists(self.test_audio_file):
            os.remove(self.test_audio_file)
        if os.path.exists(self.temp_dir):
            os.rmdir(self.temp_dir)

    def test_transcribe_audio_success_with_segments(self):
        """Test successful transcription with segments."""
        mock_result = {
            "segments": [
                {"start": 0.0, "end": 2.5, "text": "Hello world"},
                {"start": 2.5, "end": 5.0, "text": "This is a test"},
            ],
            "text": "Hello world This is a test"
        }
        
        with patch('mlx_whisper.transcribe') as mock_transcribe:
            mock_transcribe.return_value = mock_result
            
            # Capture stdout
            captured_output = StringIO()
            with patch('sys.stdout', captured_output):
                result = transcribe.transcribe_audio(self.test_audio_file)
            
            self.assertEqual(result, 0)
            
            # Parse the JSON output
            output_data = json.loads(captured_output.getvalue())
            self.assertIn("segments", output_data)
            self.assertIn("text", output_data)
            self.assertEqual(output_data["text"], "Hello world This is a test")
            self.assertEqual(len(output_data["segments"]), 2)
            self.assertEqual(output_data["segments"][0]["id"], 0)
            self.assertEqual(output_data["segments"][0]["start"], 0.0)
            self.assertEqual(output_data["segments"][0]["text"], "Hello world")

    def test_transcribe_audio_success_no_segments(self):
        """Test successful transcription without segments (fallback)."""
        mock_result = {
            "text": "Hello world without segments"
        }
        
        with patch('mlx_whisper.transcribe') as mock_transcribe:
            mock_transcribe.return_value = mock_result
            
            # Capture stdout
            captured_output = StringIO()
            with patch('sys.stdout', captured_output):
                result = transcribe.transcribe_audio(self.test_audio_file)
            
            self.assertEqual(result, 0)
            
            # Parse the JSON output
            output_data = json.loads(captured_output.getvalue())
            self.assertIn("segments", output_data)
            self.assertIn("text", output_data)
            self.assertEqual(output_data["text"], "Hello world without segments")
            self.assertEqual(len(output_data["segments"]), 1)
            self.assertEqual(output_data["segments"][0]["text"], "Hello world without segments")

    def test_transcribe_audio_error(self):
        """Test error handling."""
        with patch('mlx_whisper.transcribe') as mock_transcribe:
            mock_transcribe.side_effect = Exception("Transcription failed")
            
            # Capture both stdout and stderr
            captured_stdout = StringIO()
            captured_stderr = StringIO()
            with patch('sys.stdout', captured_stdout), patch('sys.stderr', captured_stderr):
                result = transcribe.transcribe_audio(self.test_audio_file)
            
            self.assertEqual(result, 1)
            
            # Error should be in stderr - check for the JSON error output
            stderr_content = captured_stderr.getvalue()
            self.assertIn("Transcription failed", stderr_content)

    def test_transcribe_audio_empty_segments(self):
        """Test transcription with empty segments array."""
        mock_result = {
            "segments": [],
            "text": "Some text with no segments"
        }
        
        with patch('mlx_whisper.transcribe') as mock_transcribe:
            mock_transcribe.return_value = mock_result
            
            # Capture stdout
            captured_output = StringIO()
            with patch('sys.stdout', captured_output):
                result = transcribe.transcribe_audio(self.test_audio_file)
            
            self.assertEqual(result, 0)
            
            # Parse the JSON output
            output_data = json.loads(captured_output.getvalue())
            self.assertEqual(len(output_data["segments"]), 1)  # Fallback to 1 segment
            self.assertEqual(output_data["segments"][0]["text"], "Some text with no segments")

    def test_json_output_format(self):
        """Test that output is valid JSON."""
        mock_result = {
            "segments": [
                {"start": 1.0, "end": 3.0, "text": "Test segment"}
            ],
            "text": "Test segment"
        }
        
        with patch('mlx_whisper.transcribe') as mock_transcribe:
            mock_transcribe.return_value = mock_result
            
            # Capture stdout
            captured_output = StringIO()
            with patch('sys.stdout', captured_output):
                result = transcribe.transcribe_audio(self.test_audio_file)
            
            self.assertEqual(result, 0)
            
            # Verify JSON is parseable
            try:
                json.loads(captured_output.getvalue())
            except json.JSONDecodeError:
                self.fail("Output is not valid JSON")

    def test_numeric_type_conversion(self):
        """Test that numeric types are properly converted."""
        mock_result = {
            "segments": [
                {"start": "1.5", "end": "3.7", "text": "Test"}  # String numbers
            ],
            "text": "Test"
        }
        
        with patch('mlx_whisper.transcribe') as mock_transcribe:
            mock_transcribe.return_value = mock_result
            
            # Capture stdout
            captured_output = StringIO()
            with patch('sys.stdout', captured_output):
                result = transcribe.transcribe_audio(self.test_audio_file)
            
            self.assertEqual(result, 0)
            
            output_data = json.loads(captured_output.getvalue())
            self.assertIsInstance(output_data["segments"][0]["start"], float)
            self.assertEqual(output_data["segments"][0]["start"], 1.5)


def create_test_audio():
    """Create a small test audio file for integration testing."""
    try:
        import numpy as np
        
        # Create a simple sine wave for testing
        sample_rate = 16000
        duration = 1.0
        frequency = 440.0
        
        t = np.linspace(0, duration, int(sample_rate * duration))
        audio = np.sin(2 * np.pi * frequency * t)
        
        # Save as WAV file
        import scipy.io.wavfile as wavfile
        wavfile.write("test_sample.wav", sample_rate, (audio * 32767).astype(np.int16))
        print("Created test_sample.wav for integration testing")
        
    except ImportError:
        print("NumPy/SciPy not available, skipping test audio creation")


if __name__ == "__main__":
    # Run the unit tests
    print("Running unit tests for transcribe.py...")
    unittest.main(verbosity=2)
    
    # Optionally create test audio
    create_test_audio()