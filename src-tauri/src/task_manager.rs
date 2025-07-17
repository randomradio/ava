use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Queued,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub video_path: String,
    pub api_key: String,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<String>,
    pub error: Option<String>,
}

impl Task {
    pub fn new(video_path: String, api_key: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            video_path,
            api_key,
            status: TaskStatus::Queued,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
        }
    }

    pub fn start_processing(&mut self) {
        self.status = TaskStatus::Processing;
        self.started_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn complete(&mut self, result: String) {
        self.status = TaskStatus::Completed;
        self.result = Some(result);
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn fail(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone)]
pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_task(&self, video_path: String, api_key: String) -> String {
        let task = Task::new(video_path, api_key);
        let task_id = task.id.clone();
        
        let mut tasks = self.tasks.write().await;
        tasks.insert(task_id.clone(), task);
        
        task_id
    }

    pub async fn get_task(&self, task_id: &str) -> Option<Task> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    pub async fn get_all_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    pub async fn get_tasks_by_status(&self, status: TaskStatus) -> Vec<Task> {
        let tasks = self.tasks.read().await;
        tasks.values()
            .filter(|task| task.status == status)
            .cloned()
            .collect()
    }

    pub async fn process_task(&self, task_id: &str) -> Result<String, String> {
        // Get the task
        let mut task = {
            let tasks = self.tasks.read().await;
            tasks.get(task_id)
                .cloned()
                .ok_or_else(|| "Task not found".to_string())?
        };

        // Update task to processing
        task.start_processing();
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id.to_string(), task.clone());
        }

        // Simulate processing - in real implementation, this would call the transcription service
        // For now, we'll just simulate some work
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Update task with result or error
        let mut task = {
            let tasks = self.tasks.read().await;
            tasks.get(task_id)
                .cloned()
                .ok_or_else(|| "Task not found".to_string())?
        };

        // Simulate success/failure
        let success = true; // In real implementation, check actual transcription result
        
        if success {
            let result = format!("Transcription completed for video: {}", task.video_path);
            task.complete(result.clone());
            {
                let mut tasks = self.tasks.write().await;
                tasks.insert(task_id.to_string(), task);
            }
            Ok(result)
        } else {
            let error = "Transcription failed".to_string();
            task.fail(error.clone());
            {
                let mut tasks = self.tasks.write().await;
                tasks.insert(task_id.to_string(), task);
            }
            Err(error)
        }
    }

    pub async fn queue_next_task(&self) -> Option<String> {
        let tasks = self.tasks.read().await;
        tasks.values()
            .find(|task| task.status == TaskStatus::Queued)
            .map(|task| task.id.clone())
    }

    pub async fn remove_task(&self, task_id: &str) -> bool {
        let mut tasks = self.tasks.write().await;
        tasks.remove(task_id).is_some()
    }

    pub async fn clear_completed_tasks(&self) -> usize {
        let mut tasks = self.tasks.write().await;
        let completed_ids: Vec<String> = tasks.iter()
            .filter(|(_, task)| task.status == TaskStatus::Completed || task.status == TaskStatus::Failed)
            .map(|(id, _)| id.clone())
            .collect();
        
        let removed_count = completed_ids.len();
        for id in completed_ids {
            tasks.remove(&id);
        }
        
        removed_count
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

// Tauri commands
#[tauri::command]
pub async fn create_task(
    task_manager: tauri::State<'_, TaskManager>,
    video_path: String,
    api_key: String,
) -> Result<String, String> {
    let task_id = task_manager.create_task(video_path, api_key).await;
    Ok(task_id)
}

#[tauri::command]
pub async fn get_task(
    task_manager: tauri::State<'_, TaskManager>,
    task_id: String,
) -> Result<Option<Task>, String> {
    let task = task_manager.get_task(&task_id).await;
    Ok(task)
}

#[tauri::command]
pub async fn get_all_tasks(
    task_manager: tauri::State<'_, TaskManager>,
) -> Result<Vec<Task>, String> {
    let tasks = task_manager.get_all_tasks().await;
    Ok(tasks)
}

#[tauri::command]
pub async fn process_task(
    task_manager: tauri::State<'_, TaskManager>,
    task_id: String,
) -> Result<String, String> {
    task_manager.process_task(&task_id).await
}

#[tauri::command]
pub async fn get_queued_tasks(
    task_manager: tauri::State<'_, TaskManager>,
) -> Result<Vec<Task>, String> {
    let tasks = task_manager.get_tasks_by_status(TaskStatus::Queued).await;
    Ok(tasks)
}

#[tauri::command]
pub async fn get_processing_tasks(
    task_manager: tauri::State<'_, TaskManager>,
) -> Result<Vec<Task>, String> {
    let tasks = task_manager.get_tasks_by_status(TaskStatus::Processing).await;
    Ok(tasks)
}

#[tauri::command]
pub async fn get_completed_tasks(
    task_manager: tauri::State<'_, TaskManager>,
) -> Result<Vec<Task>, String> {
    let tasks = task_manager.get_tasks_by_status(TaskStatus::Completed).await;
    Ok(tasks)
}

#[tauri::command]
pub async fn get_failed_tasks(
    task_manager: tauri::State<'_, TaskManager>,
) -> Result<Vec<Task>, String> {
    let tasks = task_manager.get_tasks_by_status(TaskStatus::Failed).await;
    Ok(tasks)
}

#[tauri::command]
pub async fn remove_task(
    task_manager: tauri::State<'_, TaskManager>,
    task_id: String,
) -> Result<bool, String> {
    let removed = task_manager.remove_task(&task_id).await;
    Ok(removed)
}

#[tauri::command]
pub async fn clear_completed_tasks(
    task_manager: tauri::State<'_, TaskManager>,
) -> Result<usize, String> {
    let count = task_manager.clear_completed_tasks().await;
    Ok(count)
}

#[tauri::command]
pub async fn queue_next_task(
    task_manager: tauri::State<'_, TaskManager>,
) -> Result<Option<String>, String> {
    let task_id = task_manager.queue_next_task().await;
    Ok(task_id)
}