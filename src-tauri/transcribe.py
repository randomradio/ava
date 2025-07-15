"""
MLX Whisper transcription script for macOS
Uses Apple's MLX framework for fast, efficient transcription.
"""

import json
import os
import sys
from typing import Dict, List, Union
import io
from contextlib import redirect_stdout, redirect_stderr

import mlx_whisper


def transcribe_audio(audio_path: str) -> int:
    """Transcribes an audio file using MLX Whisper and prints the result as JSON.

    Args:
        audio_path: The path to the audio file.

    Returns:
        An exit code, 0 for success and 1 for failure.
    """
    try:
        # Capture stdout/stderr to suppress "Detected language" messages
        stdout_capture = io.StringIO()
        stderr_capture = io.StringIO()
        
        with redirect_stdout(stdout_capture), redirect_stderr(stderr_capture):
            result = mlx_whisper.transcribe(
                audio_path,
                path_or_hf_repo="mlx-community/whisper-large-v3-turbo",
                verbose=False,
                word_timestamps=True,
            )

        segments: List[Dict[str, Union[int, float, str]]] = []
        if "segments" in result and result["segments"]:
            for i, segment in enumerate(result["segments"]):
                segments.append(
                    {
                        "id": i,
                        "start": float(segment.get("start", 0.0)),
                        "end": float(segment.get("end", 0.0)),
                        "text": str(segment.get("text", "")).strip(),
                    }
                )
        else:
            segments = [
                {
                    "id": 0,
                    "start": 0.0,
                    "end": 0.0,
                    "text": str(result.get("text", "")).strip(),
                }
            ]

        output = {"segments": segments, "text": str(result.get("text", "")).strip()}
        print(json.dumps(output))
        return 0

    except Exception as e:
        error_result = {"error": str(e), "segments": [], "text": ""}
        print(json.dumps(error_result), file=sys.stderr)
        return 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(f"Usage: python3 {sys.argv[0]} <audio_file>", file=sys.stderr)
        sys.exit(1)

    audio_file = sys.argv[1]
    if not os.path.exists(audio_file):
        print(f"Audio file not found: {audio_file}", file=sys.stderr)
        sys.exit(1)

    sys.exit(transcribe_audio(audio_file)) 
