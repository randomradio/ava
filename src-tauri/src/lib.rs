mod task_manager;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Command;
use tempfile::NamedTempFile;
use task_manager::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranscriptionSegment {
    pub id: u32,
    pub start: f64,
    pub end: f64,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranscriptionResult {
    pub segments: Vec<TranscriptionSegment>,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenshotMoment {
    pub timestamp: f64,
    pub reason: String,
    pub confidence: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessedVideo {
    pub transcription: TranscriptionResult,
    pub screenshots: Vec<ScreenshotData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenshotData {
    pub timestamp: f64,
    pub image_data: String, // Base64 encoded image
    pub caption: String,
}

#[tauri::command]
async fn extract_audio_from_video(video_path: String) -> Result<String, String> {
    let temp_file = NamedTempFile::new().map_err(|e| e.to_string())?;
    let audio_path = format!("{}.wav", temp_file.path().to_string_lossy());

    let output = Command::new("ffmpeg")
        .args([
            "-i",
            &video_path,
            "-vn",
            "-acodec",
            "pcm_s16le",
            "-ar",
            "16000",
            "-ac",
            "1",
            &audio_path,
        ])
        .output()
        .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "FFmpeg failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(audio_path)
}

#[tauri::command]
async fn transcribe_audio_mlx(audio_path: String) -> Result<TranscriptionResult, String> {
    use std::process::Command;

    // Check if Python is available
    let python_check = Command::new("python3")
        .arg("--version")
        .output()
        .map_err(|e| format!("Python3 not available: {}", e))?;

    if !python_check.status.success() {
        return Err("Python3 is required for MLX Whisper transcription".to_string());
    }

    // Try multiple locations for the script
    let mut whisper_script = None;
    let possible_paths = vec![
        std::path::Path::new("src-tauri/transcribe.py"),
        std::path::Path::new("../src-tauri/transcribe.py"),
        std::path::Path::new("transcribe.py"),
        std::path::Path::new("./transcribe.py"),
    ];

    for path in possible_paths {
        if path.exists() {
            whisper_script = Some(path.to_path_buf());
            break;
        }
    }

    let whisper_script = whisper_script.ok_or("transcribe.py script not found".to_string())?;

    // Check if virtual environment exists and use it
    let mut python_cmd = "python3";
    let venv_python = std::path::Path::new(".venv/bin/python3");

    if venv_python.exists() {
        python_cmd = ".venv/bin/python3";
    }

    let check_venv = Command::new(python_cmd)
        .args(&["-c", "import mlx_whisper"])
        .output()
        .map_err(|e| format!("Failed to check mlx-whisper in venv: {}", e))?;

    if !check_venv.status.success() {
        return Err("mlx-whisper not found. Please run: python3 -m venv .venv && source .venv/bin/activate && pip install mlx-whisper".to_string());
    }

    // Run MLX Whisper using the venv Python
    let output = Command::new(python_cmd)
        .arg(&whisper_script)
        .arg(&audio_path)
        .output()
        .map_err(|e| format!("Failed to run MLX Whisper: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("MLX Whisper failed: {}", error));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);

    // MLX Whisper outputs "Detected language: ..." text before JSON, so we need to find the JSON
    let json_start = stdout_str.find('{').unwrap_or(0);
    let json_str = &stdout_str[json_start..];

    // Try to parse the JSON part
    let result: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
        format!(
            "Failed to parse MLX Whisper output: {}\nRaw output: {}",
            e, stdout_str
        )
    })?;

    // Check for error in response
    if let Some(error) = result.get("error") {
        return Err(error
            .as_str()
            .unwrap_or("Unknown MLX Whisper error")
            .to_string());
    }

    // Parse segments
    let segments: Vec<TranscriptionSegment> = result["segments"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|segment| TranscriptionSegment {
            id: segment["id"].as_u64().unwrap_or(0) as u32,
            start: segment["start"].as_f64().unwrap_or(0.0),
            end: segment["end"].as_f64().unwrap_or(0.0),
            text: segment["text"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    let full_text = result["text"].as_str().unwrap_or("").to_string();

    Ok(TranscriptionResult {
        segments,
        text: full_text,
    })
}

#[tauri::command]
async fn analyze_transcription_for_screenshots(
    transcription: TranscriptionResult,
    api_key: String,
) -> Result<Vec<ScreenshotMoment>, String> {
    let client = reqwest::Client::new();

    let prompt = format!(
        "Analyze this video transcription and identify moments that would benefit from a screenshot to capture important visual content. \
        Look for mentions of charts, graphs, code, demonstrations, important visual elements, or key concepts that would be better understood with an image.\n\n\
        Transcription segments:\n{}\n\n\
        Return a JSON array of objects with 'timestamp' (float), 'reason' (string), and 'confidence' (float 0-1) fields. \
        Only include moments with confidence > 0.7. Limit to maximum 10 screenshots.",
        transcription.segments.iter()
            .map(|s| format!("[{:.1}s-{:.1}s]: {}", s.start, s.end, s.text))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let payload = json!({
        "model": "openai/gpt-4o-mini",
        "messages": [{
            "role": "user",
            "content": prompt
        }],
        "temperature": 0.3,
        "response_format": {"type": "json_object"}
    });

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let content = result["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("{}");

    let parsed: serde_json::Value = serde_json::from_str(content).map_err(|e| e.to_string())?;
    let moments: Vec<ScreenshotMoment> = parsed["screenshots"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|moment| ScreenshotMoment {
            timestamp: moment["timestamp"].as_f64().unwrap_or(0.0),
            reason: moment["reason"].as_str().unwrap_or("").to_string(),
            confidence: moment["confidence"].as_f64().unwrap_or(0.0),
        })
        .collect();

    Ok(moments)
}

#[tauri::command]
async fn capture_screenshot(video_path: String, timestamp: f64) -> Result<String, String> {
    // Use temporary directory for screenshots
    let temp_dir = std::env::temp_dir();
    let screenshots_dir = temp_dir.join("ava_screenshots");
    std::fs::create_dir_all(&screenshots_dir)
        .map_err(|e| format!("Failed to create screenshots dir: {}", e))?;

    // Generate unique filename
    let filename = format!("screenshot_{}.png", uuid::Uuid::new_v4());
    let screenshot_path = screenshots_dir.join(filename);
    let screenshot_path_str = screenshot_path.to_string_lossy().to_string();

    // Capture screenshot with FFmpeg
    let output = Command::new("ffmpeg")
        .args([
            "-i",
            &video_path,
            "-ss",
            &timestamp.to_string(),
            "-vframes",
            "1",
            "-f",
            "image2",
            &screenshot_path_str,
        ])
        .output()
        .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "FFmpeg failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Read and encode image as base64
    let image_data =
        std::fs::read(&screenshot_path).map_err(|e| format!("Failed to read screenshot: {}", e))?;
    let base64_image = BASE64.encode(&image_data);

    // Return base64 encoded image with proper data URI prefix
    Ok(format!("data:image/png;base64, {}", base64_image))
}

#[tauri::command]
async fn caption_image_openrouter(image_data: String, api_key: String) -> Result<String, String> {
    let client = reqwest::Client::new();

    // Image is already base64 encoded data URL from capture_screenshot

    let payload = json!({
        "model": "openai/gpt-4o-mini",
        "messages": [{
            "role": "user",
            "content": [{
                "type": "text",
                "text": "Describe this image in detail, focusing on any text, charts, code, or important visual elements visible in the screenshot."
            }, {
                "type": "image_url",
                "image_url": {
                    "url": image_data
                }
            }]
        }],
        "temperature": 0.3
    });

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let caption = result["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("Failed to generate caption")
        .to_string();

    Ok(caption)
}

#[tauri::command]
async fn process_video_complete(
    video_path: String,
    api_key: String,
) -> Result<ProcessedVideo, String> {
    // Extract audio
    let audio_path = extract_audio_from_video(video_path.clone()).await?;

    // Transcribe audio using MLX Whisper (no API key needed)
    let transcription = transcribe_audio_mlx(audio_path).await?;

    // Analyze for screenshots using OpenRouter
    let screenshot_moments =
        analyze_transcription_for_screenshots(transcription.clone(), api_key.clone()).await?;

    // Capture and caption screenshots
    let mut screenshots = Vec::new();
    for moment in screenshot_moments {
        if let Ok(image_data) = capture_screenshot(video_path.clone(), moment.timestamp).await {
            if let Ok(caption) = caption_image_openrouter(image_data.clone(), api_key.clone()).await
            {
                screenshots.push(ScreenshotData {
                    timestamp: moment.timestamp,
                    image_data,
                    caption,
                });
            }
        }
    }

    Ok(ProcessedVideo {
        transcription,
        screenshots,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            extract_audio_from_video,
            transcribe_audio_mlx,
            analyze_transcription_for_screenshots,
            capture_screenshot,
            caption_image_openrouter,
            process_video_complete,
            create_task,
            get_task,
            get_all_tasks,
            process_task,
            get_queued_tasks,
            get_processing_tasks,
            get_completed_tasks,
            get_failed_tasks,
            remove_task,
            clear_completed_tasks,
            queue_next_task
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
