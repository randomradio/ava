use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;
use uuid::Uuid;
use base64::engine::general_purpose::STANDARD as BASE64;

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    pub id: u32,
    pub start: f64,
    pub end: f64,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub image_path: String,
    pub caption: String,
}

#[tauri::command]
async fn extract_audio_from_video(video_path: String) -> Result<String, String> {
    let temp_file = NamedTempFile::new().map_err(|e| e.to_string())?;
    let audio_path = format!("{}.wav", temp_file.path().to_string_lossy());
    
    let output = Command::new("ffmpeg")
        .args([
            "-i", &video_path,
            "-vn",
            "-acodec", "pcm_s16le",
            "-ar", "16000",
            "-ac", "1",
            &audio_path,
        ])
        .output()
        .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("FFmpeg failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(audio_path)
}

#[tauri::command]
async fn transcribe_audio_openrouter(audio_path: String, api_key: String) -> Result<TranscriptionResult, String> {
    let client = reqwest::Client::new();
    
    // Read audio file
    let audio_data = std::fs::read(&audio_path).map_err(|e| e.to_string())?;
    let audio_base64 = BASE64.encode(&audio_data);
    
    // Create request payload
    let mut payload = HashMap::new();
    payload.insert("model", "openai/whisper-large-v3");
    payload.insert("audio", &audio_base64);
    payload.insert("response_format", "verbose_json");
    payload.insert("timestamp_granularities", "segment");
    
    let response = client
        .post("https://openrouter.ai/api/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    let transcription: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    
    // Parse segments
    let segments: Vec<TranscriptionSegment> = transcription["segments"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .enumerate()
        .map(|(i, segment)| TranscriptionSegment {
            id: i as u32,
            start: segment["start"].as_f64().unwrap_or(0.0),
            end: segment["end"].as_f64().unwrap_or(0.0),
            text: segment["text"].as_str().unwrap_or("").to_string(),
        })
        .collect();
    
    let full_text = transcription["text"].as_str().unwrap_or("").to_string();
    
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
    
    let mut payload = HashMap::new();
    payload.insert("model", "openai/gpt-4o-mini");
    payload.insert("messages", vec![
        HashMap::from([
            ("role", "user"),
            ("content", &prompt),
        ])
    ]);
    payload.insert("temperature", 0.3);
    payload.insert("response_format", HashMap::from([("type", "json_object")]));
    
    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let content = result["choices"][0]["message"]["content"].as_str().unwrap_or("{}");
    
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
    let temp_file = NamedTempFile::new().map_err(|e| e.to_string())?;
    let screenshot_path = format!("{}.png", temp_file.path().to_string_lossy());
    
    let output = Command::new("ffmpeg")
        .args([
            "-i", &video_path,
            "-ss", &timestamp.to_string(),
            "-vframes", "1",
            "-f", "image2",
            &screenshot_path,
        ])
        .output()
        .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("FFmpeg failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(screenshot_path)
}

#[tauri::command]
async fn caption_image_openrouter(image_path: String, api_key: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    // Read and encode image
    let image_data = std::fs::read(&image_path).map_err(|e| e.to_string())?;
    let image_base64 = BASE64.encode(&image_data);
    let mime_type = "image/png";
    
    let mut payload = HashMap::new();
    payload.insert("model", "openai/gpt-4o-mini");
    payload.insert("messages", vec![
        HashMap::from([
            ("role", "user"),
            ("content", vec![
                HashMap::from([
                    ("type", "text"),
                    ("text", "Describe this image in detail, focusing on any text, charts, code, or important visual elements visible in the screenshot."),
                ]),
                HashMap::from([
                    ("type", "image_url"),
                    ("image_url", HashMap::from([
                        ("url", format!("data:{};base64,{}", mime_type, image_base64)),
                    ])),
                ]),
            ]),
        ])
    ]);
    payload.insert("temperature", 0.3);
    
    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let caption = result["choices"][0]["message"]["content"].as_str().unwrap_or("Failed to generate caption").to_string();
    
    Ok(caption)
}

#[tauri::command]
async fn process_video_complete(video_path: String, api_key: String) -> Result<ProcessedVideo, String> {
    // Extract audio
    let audio_path = extract_audio_from_video(video_path.clone()).await?;
    
    // Transcribe audio
    let transcription = transcribe_audio_openrouter(audio_path, api_key.clone()).await?;
    
    // Analyze for screenshots
    let screenshot_moments = analyze_transcription_for_screenshots(transcription.clone(), api_key.clone()).await?;
    
    // Capture and caption screenshots
    let mut screenshots = Vec::new();
    for moment in screenshot_moments {
        if let Ok(screenshot_path) = capture_screenshot(video_path.clone(), moment.timestamp).await {
            if let Ok(caption) = caption_image_openrouter(screenshot_path.clone(), api_key.clone()).await {
                screenshots.push(ScreenshotData {
                    timestamp: moment.timestamp,
                    image_path: screenshot_path,
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
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            extract_audio_from_video,
            transcribe_audio_openrouter,
            analyze_transcription_for_screenshots,
            capture_screenshot,
            caption_image_openrouter,
            process_video_complete
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
