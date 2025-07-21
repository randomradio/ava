import streamlit as st
import os
import json
import subprocess
import threading
import time
from pathlib import Path
import pandas as pd
from PIL import Image
import sys

# Add current directory to path to import mlx_cli
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from mlx_cli import TranscriptionCheckpoint, MLXTranscriber

# Page configuration
st.set_page_config(
    page_title="MLX Transcription Tool",
    page_icon="🎬",
    layout="wide"
)

# Session state initialization
if 'processing_status' not in st.session_state:
    st.session_state.processing_status = "idle"
if 'current_video' not in st.session_state:
    st.session_state.current_video = None
if 'last_output' not in st.session_state:
    st.session_state.last_output = None

# Use Streamlit's native session state for thread communication
if 'progress_data' not in st.session_state:
    st.session_state.progress_data = {"status": "idle", "error": None, "output_dir": None}

def process_video_sync(video_path, output_dir, progress_callback):
    """Process video synchronously with progress callback"""
    try:
        checkpoint = TranscriptionCheckpoint(video_path, output_dir)
        checkpoint.status = "not_started"
        checkpoint.transcription = []
        checkpoint.screenshots = []
        checkpoint.current_time = 0
        
        # Remove existing checkpoint if forcing reprocess
        if checkpoint.checkpoint_file.exists():
            checkpoint.checkpoint_file.unlink()
        
        # Simple synchronous processing
        transcriber = MLXTranscriber(checkpoint)
        
        # Update progress
        progress_callback("processing", None)
        
        # Process the video
        transcriber.process_video()
        
        progress_callback("completed", output_dir)
        
    except Exception as e:
        progress_callback("error", str(e))

def get_output_directories():
    """Get list of processed videos"""
    base_dir = Path("./output")
    if not base_dir.exists():
        return []
    
    directories = []
    # Find all directories with checkpoint files
    for item in base_dir.rglob("*_checkpoint.json"):
        directories.append(str(item.parent))
    
    # Also include directories that exist
    for item in base_dir.iterdir():
        if item.is_dir():
            directories.append(str(item))
    
    return sorted(list(set(directories)))

def load_transcription_results(output_path):
    """Load transcription results from checkpoint"""
    output_dir = Path(output_path)
    video_name = output_dir.name
    checkpoint_file = output_dir / f"{video_name}_checkpoint.json"
    
    if checkpoint_file.exists():
        with open(checkpoint_file, 'r') as f:
            return json.load(f)
    return None

def main():
    st.title("🎬 MLX Transcription Tool")
    st.markdown("Simple web interface for MLX Whisper transcription with screenshots")
    
    # Sidebar for navigation
    with st.sidebar:
        st.header("Navigation")
        page = st.selectbox("Choose a page:", ["Process New Video", "View Results", "Batch Processing"])
    
    if page == "Process New Video":
        process_new_video_page()
    elif page == "View Results":
        view_results_page()
    elif page == "Batch Processing":
        batch_processing_page()

def process_new_video_page():
    """Page for processing new videos"""
    st.header("Process New Video")
    
    col1, col2 = st.columns([2, 1])
    
    with col1:
        # File upload
        uploaded_file = st.file_uploader("Choose a video file", type=['mp4', 'mov', 'avi', 'mkv'])
        
        if uploaded_file is not None:
            # Save uploaded file
            temp_dir = Path("./temp_uploads")
            temp_dir.mkdir(exist_ok=True)
            video_path = temp_dir / uploaded_file.name
            
            with open(video_path, "wb") as f:
                f.write(uploaded_file.getbuffer())
            
            st.success(f"Uploaded: {uploaded_file.name}")
            
            # Output directory
            output_dir = st.text_input("Output directory", value=f"./output/{Path(uploaded_file.name).stem}")
            
            # Process button
            if st.button("Start Transcription", type="primary"):
                progress_bar = st.progress(0)
                status_text = st.empty()
                
                # Update UI immediately
                status_text.text("Starting transcription...")
                progress_bar.progress(10)
                
                # Process synchronously to avoid threading issues
                def progress_callback(status, data):
                    if status == "processing":
                        status_text.text("Processing video...")
                        progress_bar.progress(50)
                    elif status == "completed":
                        status_text.text("Transcription completed!")
                        progress_bar.progress(100)
                        st.session_state.last_output = data
                        st.balloons()
                    elif status == "error":
                        status_text.error(f"Error: {data}")
                        progress_bar.empty()
                
                process_video_sync(str(video_path), output_dir, progress_callback)
                
                # Refresh to show results
                st.rerun()
                
    with col2:
        st.info("**Supported formats:**\n- MP4\n- MOV\n- AVI\n- MKV")
        st.warning("**Processing time:**\n- ~3-5 minutes for 10 min video\n- Depends on video length")

def view_results_page():
    """Page for viewing existing results"""
    st.header("View Transcription Results")
    
    output_dirs = get_output_directories()
    
    if not output_dirs:
        st.info("No processed videos found. Process a video first!")
        return
    
    selected_output = st.selectbox("Select a processed video:", output_dirs)
    
    if selected_output:
        results = load_transcription_results(selected_output)
        if results:
            display_transcription_results(results)

def batch_processing_page():
    """Page for batch processing multiple videos"""
    st.header("Batch Processing")
    
    st.info("Batch processing feature coming soon!")
    
    # File upload for multiple videos
    uploaded_files = st.file_uploader(
        "Choose multiple video files", 
        type=['mp4', 'mov', 'avi', 'mkv'],
        accept_multiple_files=True
    )
    
    if uploaded_files:
        st.write(f"Selected {len(uploaded_files)} videos:")
        for file in uploaded_files:
            st.write(f"- {file.name}")
        
        if st.button("Process All Videos"):
            st.info("Batch processing will be implemented soon!")

def display_transcription_results(results):
    """Display transcription results in organized format"""
    st.header("Transcription Results")
    
    # Video info
    with st.expander("Video Information", expanded=True):
        st.json({
            "video_path": results.get("video_path"),
            "status": results.get("status"),
            "total_segments": len(results.get("transcription", [])),
            "total_screenshots": len(results.get("screenshots", []))
        })
    
    # Transcription segments
    st.subheader("Transcription Segments")
    transcription_df = pd.DataFrame(results.get("transcription", []))
    
    if not transcription_df.empty:
        # Convert time to readable format
        transcription_df['start_time'] = transcription_df['start'].apply(lambda x: f"{x:.1f}s")
        transcription_df['end_time'] = transcription_df['end'].apply(lambda x: f"{x:.1f}s")
        
        # Display as table
        st.dataframe(transcription_df[['start_time', 'end_time', 'text']], use_container_width=True)
        
        # Download button
        csv = transcription_df.to_csv(index=False)
        st.download_button(
            label="Download Transcription CSV",
            data=csv,
            file_name="transcription.csv",
            mime="text/csv"
        )
    
    # Screenshots
    if results.get("screenshots"):
        st.subheader("Screenshots")
        screenshots = results["screenshots"]
        
        # Display in grid
        cols = st.columns(3)
        for idx, screenshot in enumerate(screenshots):
            with cols[idx % 3]:
                if os.path.exists(screenshot["file"]):
                    image = Image.open(screenshot["file"])
                    st.image(image, caption=f"{screenshot['timestamp']:.1f}s: {screenshot['text']}", use_column_width=True)
    
    # Raw JSON download
    st.subheader("Export Data")
    json_str = json.dumps(results, indent=2)
    st.download_button(
        label="Download Complete JSON",
        data=json_str,
        file_name="transcription_complete.json",
        mime="application/json"
    )

if __name__ == "__main__":
    main()