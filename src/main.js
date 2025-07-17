const { invoke } = window.__TAURI__.core;

// Global state
let selectedVideoPath = null;
let currentTaskId = null;
let isProcessing = false;
let transcriptionData = null;
let screenshotsData = null;

// Utility functions
function formatTime(seconds) {
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = Math.floor(seconds % 60);
  return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
}

function formatDuration(seconds) {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);
  
  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }
  return `${minutes}:${secs.toString().padStart(2, '0')}`;
}

function updateProgress(text, percentage = 0) {
  const progressText = document.querySelector("#progress-text");
  const progressFill = document.querySelector("#progress-fill");
  
  progressText.textContent = text;
  progressFill.style.width = `${percentage}%`;
}

function showError(message) {
  const errorDiv = document.createElement("div");
  errorDiv.className = "error-message";
  errorDiv.textContent = message;
  errorDiv.style.cssText = `
    position: fixed;
    top: 20px;
    right: 20px;
    background: #e74c3c;
    color: white;
    padding: 1rem;
    border-radius: 8px;
    z-index: 1000;
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
  `;

  document.body.appendChild(errorDiv);
  setTimeout(() => {
    if (errorDiv.parentNode) {
      document.body.removeChild(errorDiv);
    }
  }, 5000);
}

// File selection with native dialog
async function selectVideoFile() {
  try {
    // Check if Tauri dialog is available
    if (!window.__TAURI__) {
      // Fallback to basic prompt for testing
      const filePath = prompt("Enter video file path:");
      if (filePath) {
        selectedVideoPath = filePath;
        document.querySelector("#selected-file").innerHTML = `
          <strong>Selected:</strong> ${filePath.split('/').pop()}
        `;
        document.querySelector("#process-btn").disabled = false;
        updateProgress("Ready to process video");
      }
      return;
    }

    // Use Tauri's dialog API
    const { open } = window.__TAURI__.dialog;
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'Video Files',
          extensions: ['mp4', 'mov', 'avi', 'mkv', 'webm', 'm4v', '3gp', 'flv', 'wmv']
        }
      ]
    });

    if (selected) {
      const filePath = Array.isArray(selected) ? selected[0] : selected;
      selectedVideoPath = filePath;
      document.querySelector("#selected-file").innerHTML = `
        <strong>Selected:</strong> ${filePath.split('/').pop()}
      `;
      document.querySelector("#process-btn").disabled = false;
      updateProgress("Ready to process video");
    }
  } catch (error) {
    console.error("Error selecting file:", error);
    showError("Error selecting file: " + error);
    
    // Fallback to basic prompt
    const filePath = prompt("Enter video file path:");
    if (filePath) {
      selectedVideoPath = filePath;
      document.querySelector("#selected-file").innerHTML = `
        <strong>Selected:</strong> ${filePath.split('/').pop()}
      `;
      document.querySelector("#process-btn").disabled = false;
      updateProgress("Ready to process video");
    }
  }
}

// Task management
async function createTask() {
  if (!selectedVideoPath) {
    showError("Please select a video file first");
    return null;
  }

  const apiKey = document.querySelector("#api-key").value;
  if (!apiKey) {
    showError("Please enter your OpenRouter API key");
    return null;
  }

  try {
    const taskId = await invoke("create_task", {
      videoPath: selectedVideoPath,
      apiKey: apiKey
    });
    
    currentTaskId = taskId;
    await loadTasks();
    return taskId;
  } catch (error) {
    console.error("Error creating task:", error);
    showError("Error creating task: " + error);
    return null;
  }
}

async function processVideo() {
  const taskId = await createTask();
  if (!taskId) return;

  try {
    setProcessingState(true);
    updateProgress("Starting processing...", 10);

    await invoke("process_task", { taskId });
    
    updateProgress("Processing completed!", 100);
    await loadTasks();
    await loadTaskResults(taskId);
    
  } catch (error) {
    console.error("Error processing video:", error);
    showError("Error processing video: " + error);
    updateProgress("Processing failed", 0);
  } finally {
    setProcessingState(false);
  }
}

async function stopProcessing() {
  if (currentTaskId) {
    try {
      // For now, we'll just mark the task as failed
      // In a real implementation, you'd implement proper task cancellation
      updateProgress("Processing stopped", 0);
      setProcessingState(false);
    } catch (error) {
      console.error("Error stopping task:", error);
    }
  }
}

function setProcessingState(processing) {
  isProcessing = processing;
  const processBtn = document.querySelector("#process-btn");
  const pauseBtn = document.querySelector("#pause-btn");
  const stopBtn = document.querySelector("#stop-btn");
  
  processBtn.disabled = processing;
  pauseBtn.disabled = !processing;
  stopBtn.disabled = !processing;
}

// Task UI
async function loadTasks() {
  try {
    const tasks = await invoke("get_all_tasks");
    displayTasks(tasks);
  } catch (error) {
    console.error("Error loading tasks:", error);
  }
}

function displayTasks(tasks) {
  const tasksList = document.querySelector("#tasks-list");
  tasksList.innerHTML = "";

  if (!tasks || tasks.length === 0) {
    tasksList.innerHTML = `
      <div style="text-align: center; color: #666; padding: 2rem;">
        No tasks yet. Select a video and start processing!
      </div>
    `;
    return;
  }

  tasks.forEach(task => {
    const taskDiv = createTaskElement(task);
    tasksList.appendChild(taskDiv);
  });
}

function createTaskElement(task) {
  const div = document.createElement("div");
  div.className = `task-item ${task.status}`;
  
  const filename = task.video_path.split('/').pop();
  const created = new Date(task.created_at).toLocaleString();
  
  div.innerHTML = `
    <div class="task-header">
      <div class="task-title">${filename}</div>
      <div class="task-status status-${task.status}">${task.status}</div>
    </div>
    <div class="task-details">
      <div>Created: ${created}</div>
      ${task.status === 'completed' ? `<div>Duration: ${formatDuration(task.duration || 0)}</div>` : ''}
    </div>
    <div class="task-actions">
      ${task.status === 'completed' ? 
        `<button class="btn btn-primary" onclick="loadTaskResults('${task.id}')">View Results</button>` : ''}
      <button class="btn btn-secondary" onclick="removeTask('${task.id}')">Remove</button>
    </div>
  `;

  div.addEventListener('click', (e) => {
    if (!e.target.classList.contains('btn')) {
      showTaskDetails(task);
    }
  });

  return div;
}

async function removeTask(taskId) {
  try {
    await invoke("remove_task", { taskId });
    await loadTasks();
  } catch (error) {
    console.error("Error removing task:", error);
    showError("Error removing task: " + error);
  }
}

async function clearCompletedTasks() {
  try {
    await invoke("clear_completed_tasks");
    await loadTasks();
  } catch (error) {
    console.error("Error clearing tasks:", error);
    showError("Error clearing tasks: " + error);
  }
}

// Task results
async function loadTaskResults(taskId) {
  try {
    const task = await invoke("get_task", { taskId });
    if (task.status === 'completed' && task.result) {
      transcriptionData = task.result.transcription;
      screenshotsData = task.result.screenshots;
      
      displayTranscription(transcriptionData);
      displayScreenshots(screenshotsData);
      
      // Switch to transcription tab
      switchTab('transcription');
    }
  } catch (error) {
    console.error("Error loading task results:", error);
    showError("Error loading task results: " + error);
  }
}

// Transcription display and editing
function displayTranscription(transcription) {
  const container = document.querySelector("#transcription-content");
  container.innerHTML = "";

  if (!transcription || !transcription.segments || transcription.segments.length === 0) {
    container.innerHTML = `
      <div style="text-align: center; color: #666; padding: 2rem;">
        No transcription data available
      </div>
    `;
    return;
  }

  const editorDiv = document.createElement("div");
  editorDiv.className = "transcription-editor";

  transcription.segments.forEach((segment, index) => {
    const segmentDiv = document.createElement("div");
    segmentDiv.className = "transcription-segment";
    segmentDiv.innerHTML = `
      <div class="segment-header">
        <span class="segment-time">${formatTime(segment.start)} - ${formatTime(segment.end)}</span>
        <button class="btn btn-secondary" onclick="jumpToScreenshot(${segment.start})">
          📸 Find Screenshot
        </button>
      </div>
      <textarea class="segment-textarea" data-index="${index}" data-start="${segment.start}" data-end="${segment.end}">
        ${segment.text}
      </textarea>
    `;
    editorDiv.appendChild(segmentDiv);
  });

  container.appendChild(editorDiv);
}

async function saveTranscription() {
  if (!transcriptionData) return;

  const textareas = document.querySelectorAll('.segment-textarea');
  const updatedSegments = [];

  textareas.forEach((textarea, index) => {
    updatedSegments.push({
      ...transcriptionData.segments[index],
      text: textarea.value.trim()
    });
  });

  transcriptionData.segments = updatedSegments;
  transcriptionData.text = updatedSegments.map(s => s.text).join(' ');

  showError("Transcription saved successfully!");
}

// Screenshots display
function displayScreenshots(screenshots) {
  const container = document.querySelector("#screenshots-content");
  container.innerHTML = "";

  if (!screenshots || screenshots.length === 0) {
    container.innerHTML = `
      <div style="text-align: center; color: #666; padding: 2rem;">
        No screenshots available
      </div>
    `;
    return;
  }

  const gridDiv = document.createElement("div");
  gridDiv.className = "screenshot-grid";

  screenshots.forEach((screenshot, index) => {
    const cardDiv = document.createElement("div");
    cardDiv.className = "screenshot-card";
    cardDiv.innerHTML = `
      <img src="${screenshot.image_data}" alt="Screenshot at ${formatTime(screenshot.timestamp)}" class="screenshot-image" />
      <div class="screenshot-info">
        <div class="screenshot-timestamp">⏰ ${formatTime(screenshot.timestamp)}</div>
        <div class="screenshot-caption">${screenshot.caption}</div>
        <a href="#" class="screenshot-link" onclick="jumpToTranscription(${screenshot.timestamp})">
          🔗 Jump to Transcription
        </a>
      </div>
    `;
    gridDiv.appendChild(cardDiv);
  });

  container.appendChild(gridDiv);
}

// Navigation between transcription and screenshots
function jumpToScreenshot(timestamp) {
  switchTab('screenshots');
  // Highlight screenshot at this timestamp
  const screenshots = document.querySelectorAll('.screenshot-card');
  screenshots.forEach(screenshot => {
    const screenshotTime = parseFloat(screenshot.querySelector('.screenshot-timestamp').textContent.match(/[\d.]+/)[0]);
    if (Math.abs(screenshotTime - timestamp) < 1) {
      screenshot.style.border = '2px solid #3498db';
      screenshot.scrollIntoView({ behavior: 'smooth', block: 'center' });
    } else {
      screenshot.style.border = 'none';
    }
  });
}

function jumpToTranscription(timestamp) {
  switchTab('transcription');
  // Highlight transcription segment near this timestamp
  const segments = document.querySelectorAll('.transcription-segment');
  segments.forEach(segment => {
    const segmentStart = parseFloat(segment.querySelector('.segment-textarea').dataset.start);
    const segmentEnd = parseFloat(segment.querySelector('.segment-textarea').dataset.end);
    
    if (timestamp >= segmentStart && timestamp <= segmentEnd) {
      segment.style.border = '2px solid #3498db';
      segment.scrollIntoView({ behavior: 'smooth', block: 'center' });
    } else {
      segment.style.border = '1px solid #eee';
    }
  });
}

// Tab management
function switchTab(tabName) {
  // Hide all tab contents
  document.querySelectorAll('.tab-content').forEach(content => {
    content.classList.remove('active');
  });
  
  // Remove active class from all buttons
  document.querySelectorAll('.tab-btn').forEach(btn => {
    btn.classList.remove('active');
  });
  
  // Show selected tab content
  document.getElementById(`${tabName}-tab`).classList.add('active');
  
  // Add active class to button
  document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');
}

// Task details modal
function showTaskDetails(task) {
  const modal = document.getElementById('task-modal');
  const details = document.getElementById('task-details');
  
  const filename = task.video_path.split('/').pop();
  const created = new Date(task.created_at).toLocaleString();
  const updated = new Date(task.updated_at).toLocaleString();
  
  details.innerHTML = `
    <h3>${filename}</h3>
    <p><strong>Status:</strong> ${task.status}</p>
    <p><strong>Created:</strong> ${created}</p>
    <p><strong>Updated:</strong> ${updated}</p>
    ${task.error ? `<p><strong>Error:</strong> ${task.error}</p>` : ''}
    ${task.result ? `
      <h4>Results:</h4>
      <p><strong>Duration:</strong> ${formatDuration(task.duration || 0)}</p>
      <p><strong>Segments:</strong> ${task.result.transcription.segments.length}</p>
      <p><strong>Screenshots:</strong> ${task.result.screenshots.length}</p>
    ` : ''}
  `;
  
  modal.style.display = 'block';
}

// Export functionality
async function exportScreenshots() {
  if (!screenshotsData) {
    showError("No screenshots to export");
    return;
  }

  try {
    if (!window.__TAURI__) {
      showError("Export not available in fallback mode");
      return;
    }

    const { save } = window.__TAURI__.dialog;
    const selected = await save({
      defaultPath: `screenshots-${Date.now()}.zip`,
      filters: [{ name: 'ZIP Files', extensions: ['zip'] }]
    });

    if (selected) {
      // Here you would implement actual export functionality
      showError("Export functionality would be implemented here");
    }
  } catch (error) {
    console.error("Error exporting screenshots:", error);
    showError("Error exporting screenshots: " + error);
  }
}

// Event listeners
document.addEventListener('DOMContentLoaded', async () => {
  // Tab switching
  document.querySelectorAll('.tab-btn').forEach(btn => {
    btn.addEventListener('click', (e) => {
      const tabName = e.target.dataset.tab;
      switchTab(tabName);
    });
  });

  // Button event listeners
  document.querySelector("#select-video-btn").addEventListener("click", selectVideoFile);
  document.querySelector("#process-btn").addEventListener("click", processVideo);
  document.querySelector("#stop-btn").addEventListener("click", stopProcessing);
  document.querySelector("#save-transcription-btn").addEventListener("click", saveTranscription);
  document.querySelector("#clear-completed-btn").addEventListener("click", clearCompletedTasks);
  document.querySelector("#export-screenshots-btn").addEventListener("click", exportScreenshots);

  // Modal close
  document.querySelector(".close").addEventListener("click", () => {
    document.getElementById('task-modal').style.display = 'none';
  });

  window.addEventListener("click", (e) => {
    const modal = document.getElementById('task-modal');
    if (e.target === modal) {
      modal.style.display = 'none';
    }
  });

  // Initial load
  await loadTasks();
  updateProgress("Ready - Select a video file to begin");
});

// Auto-refresh tasks every 5 seconds
setInterval(async () => {
  if (!isProcessing) {
    await loadTasks();
  }
}, 5000);