const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;

let selectedVideoPath = null;
let apiKey = null;

function updateProgress(text, active = false) {
  const progressText = document.querySelector("#progress-text");
  const progressBar = document.querySelector("#progress-bar");
  
  progressText.textContent = text;
  
  if (active) {
    progressBar.classList.add("active");
  } else {
    progressBar.classList.remove("active");
  }
}

function displayTranscription(transcription) {
  const container = document.querySelector("#transcription-result");
  container.innerHTML = "";
  
  if (!transcription.segments || transcription.segments.length === 0) {
    container.innerHTML = "<p>No transcription segments found.</p>";
    return;
  }
  
  transcription.segments.forEach(segment => {
    const segmentDiv = document.createElement("div");
    segmentDiv.className = "transcription-segment";
    
    const timeDiv = document.createElement("div");
    timeDiv.className = "segment-time";
    timeDiv.textContent = `${formatTime(segment.start)} - ${formatTime(segment.end)}`;
    
    const textDiv = document.createElement("div");
    textDiv.className = "segment-text";
    textDiv.textContent = segment.text;
    
    segmentDiv.appendChild(timeDiv);
    segmentDiv.appendChild(textDiv);
    container.appendChild(segmentDiv);
  });
}

function displayScreenshots(screenshots) {
  const container = document.querySelector("#screenshots-container");
  container.innerHTML = "";
  
  if (!screenshots || screenshots.length === 0) {
    container.innerHTML = "<p>No screenshots were captured.</p>";
    return;
  }
  
  screenshots.forEach(screenshot => {
    const itemDiv = document.createElement("div");
    itemDiv.className = "screenshot-item";
    
    const img = document.createElement("img");
    img.src = `file://${screenshot.image_path}`;
    img.alt = `Screenshot at ${formatTime(screenshot.timestamp)}`;
    
    const timeDiv = document.createElement("div");
    timeDiv.className = "screenshot-time";
    timeDiv.textContent = `Time: ${formatTime(screenshot.timestamp)}`;
    
    const captionDiv = document.createElement("div");
    captionDiv.className = "screenshot-caption";
    captionDiv.textContent = screenshot.caption;
    
    itemDiv.appendChild(img);
    itemDiv.appendChild(timeDiv);
    itemDiv.appendChild(captionDiv);
    container.appendChild(itemDiv);
  });
}

function formatTime(seconds) {
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = Math.floor(seconds % 60);
  return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
}

async function selectVideoFile() {
  try {
    const filePath = await open({
      multiple: false,
      filters: [
        {
          name: 'Video files',
          extensions: ['mp4', 'avi', 'mov', 'mkv', 'wmv', 'flv', 'webm']
        }
      ]
    });
    
    if (filePath) {
      selectedVideoPath = filePath;
      document.querySelector("#selected-file").innerHTML = `
        <strong>Selected file:</strong> ${filePath.split('/').pop()}
      `;
      document.querySelector("#process-btn").disabled = false;
      updateProgress("Ready to process video");
    }
  } catch (error) {
    console.error("Error selecting file:", error);
    alert("Error selecting file: " + error);
  }
}

async function processVideo() {
  if (!selectedVideoPath) {
    alert("Please select a video file first");
    return;
  }
  
  apiKey = document.querySelector("#api-key").value;
  if (!apiKey) {
    alert("Please enter your OpenRouter API key");
    return;
  }
  
  try {
    document.querySelector("#process-btn").disabled = true;
    updateProgress("Processing video...", true);
    
    const result = await invoke("process_video_complete", {
      videoPath: selectedVideoPath,
      apiKey: apiKey
    });
    
    updateProgress("Processing completed successfully!");
    
    displayTranscription(result.transcription);
    displayScreenshots(result.screenshots);
    
  } catch (error) {
    console.error("Error processing video:", error);
    alert("Error processing video: " + error);
    updateProgress("Error occurred during processing");
  } finally {
    document.querySelector("#process-btn").disabled = false;
  }
}

function showError(message) {
  const errorDiv = document.createElement("div");
  errorDiv.className = "error-message";
  errorDiv.textContent = message;
  errorDiv.style.cssText = `
    position: fixed;
    top: 20px;
    right: 20px;
    background: #ff4444;
    color: white;
    padding: 1rem;
    border-radius: 8px;
    z-index: 1000;
  `;
  
  document.body.appendChild(errorDiv);
  
  setTimeout(() => {
    document.body.removeChild(errorDiv);
  }, 5000);
}

window.addEventListener("DOMContentLoaded", () => {
  document.querySelector("#select-video-btn").addEventListener("click", selectVideoFile);
  document.querySelector("#process-btn").addEventListener("click", processVideo);
  
  // Check for FFmpeg availability
  updateProgress("Ready - Make sure FFmpeg is installed on your system");
});
