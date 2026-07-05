//! Download scheduler - Manages download queue and concurrency
//! 
//! This module provides the scheduling logic for downloads.
//! It ensures only a configurable number of downloads run simultaneously.

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::download::errors::Result;
use crate::download::models::DownloadTask;

/// Download state for UI display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadState {
    Pending,
    Queued,
    Downloading,
    Completed,
    Cancelled,
    Failed,
}

/// Queued download entry
#[derive(Debug, Clone)]
pub struct QueuedDownload {
    pub task: DownloadTask,
    pub position: usize,
}

/// Download scheduler for managing queue and concurrency
/// 
/// Uses VecDeque for O(1) FIFO queue operations.
/// HashMap for O(1) task lookup.
pub struct DownloadScheduler {
    /// Queue of download IDs waiting to run
    queue: Arc<RwLock<VecDeque<String>>>,
    /// Tasks indexed by ID for quick access
    queued_tasks: Arc<RwLock<std::collections::HashMap<String, DownloadTask>>>,
    /// Maximum concurrent downloads
    max_concurrent: usize,
}

impl DownloadScheduler {
    /// Create a new scheduler with default max concurrent downloads
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            queued_tasks: Arc::new(RwLock::new(std::collections::HashMap::new())),
            max_concurrent: 3, // Default
        }
    }

    /// Create a new scheduler with custom max concurrent downloads
    pub fn with_max_concurrent(max: usize) -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            queued_tasks: Arc::new(RwLock::new(std::collections::HashMap::new())),
            max_concurrent: max,
        }
    }

    /// Enqueue a download task
    /// 
    /// Returns the position in the queue.
    pub async fn enqueue(&self, task: DownloadTask) -> Result<usize> {
        let position = {
            let mut queue = self.queue.write().await;
            let mut tasks = self.queued_tasks.write().await;
            
            queue.push_back(task.id.clone());
            tasks.insert(task.id.clone(), task);
            
            queue.len()
        };

        Ok(position)
    }

    /// Dequeue a download task by ID
    /// 
    /// Returns the task if found, None otherwise.
    pub async fn dequeue(&self, id: &str) -> Option<DownloadTask> {
        let mut queue = self.queue.write().await;
        let mut tasks = self.queued_tasks.write().await;
        
        // Remove from queue
        queue.retain(|x| x != id);
        
        // Remove and return task
        tasks.remove(id)
    }

    /// Get the position of a download in the queue
    pub async fn get_position(&self, id: &str) -> Option<usize> {
        let queue = self.queue.read().await;
        queue.iter().position(|x| x == id).map(|p| p + 1) // 1-indexed
    }

    /// Get all queued download IDs
    pub async fn get_queued_ids(&self) -> Vec<String> {
        self.queue.read().await.iter().cloned().collect()
    }

    /// Get a queued task by ID
    pub async fn get_task(&self, id: &str) -> Option<DownloadTask> {
        self.queued_tasks.read().await.get(id).cloned()
    }

    /// Check if queue is empty
    pub async fn is_empty(&self) -> bool {
        self.queue.read().await.is_empty()
    }

    /// Get queue length
    pub async fn len(&self) -> usize {
        self.queue.read().await.len()
    }

    /// Get max concurrent downloads
    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    /// Check if we can start a new download
    pub async fn can_start(&self, active_count: usize) -> bool {
        active_count < self.max_concurrent
    }

    /// Check if a task is in the queue
    pub async fn contains(&self, id: &str) -> bool {
        let queue = self.queue.read().await;
        queue.contains(&id.to_string())
    }

    /// Get next task from queue (without removing)
    pub async fn peek_next(&self) -> Option<(String, DownloadTask)> {
        let queue = self.queue.read().await;
        let tasks = self.queued_tasks.read().await;
        
        queue.front().and_then(|id| {
            tasks.get(id).map(|task| (id.clone(), task.clone()))
        })
    }

    /// Remove and return the next task from queue
    pub async fn pop_next(&self) -> Option<(String, DownloadTask)> {
        let mut queue = self.queue.write().await;
        let mut tasks = self.queued_tasks.write().await;
        
        queue.pop_front().and_then(|id| {
            tasks.remove(&id).map(|task| (id, task))
        })
    }
}

impl Default for DownloadScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for DownloadScheduler {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
            queued_tasks: self.queued_tasks.clone(),
            max_concurrent: self.max_concurrent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enqueue_dequeue() {
        let scheduler = DownloadScheduler::new();
        
        let task = DownloadTask {
            id: "test-1".to_string(),
            url: "https://example.com/file1".to_string(),
            filename: "file1.zip".to_string(),
            save_location: "/downloads".to_string(),
        };
        
        let position = scheduler.enqueue(task.clone()).await.unwrap();
        assert_eq!(position, 1);
        
        let dequeued = scheduler.dequeue("test-1").await;
        assert!(dequeued.is_some());
        assert_eq!(dequeued.unwrap().id, "test-1");
    }

    #[tokio::test]
    async fn test_fifo_ordering() {
        let scheduler = DownloadScheduler::new();
        
        let task1 = DownloadTask {
            id: "task-1".to_string(),
            url: "https://example.com/1".to_string(),
            filename: "1.zip".to_string(),
            save_location: "/downloads".to_string(),
        };
        
        let task2 = DownloadTask {
            id: "task-2".to_string(),
            url: "https://example.com/2".to_string(),
            filename: "2.zip".to_string(),
            save_location: "/downloads".to_string(),
        };
        
        let task3 = DownloadTask {
            id: "task-3".to_string(),
            url: "https://example.com/3".to_string(),
            filename: "3.zip".to_string(),
            save_location: "/downloads".to_string(),
        };
        
        scheduler.enqueue(task1).await.unwrap();
        scheduler.enqueue(task2).await.unwrap();
        scheduler.enqueue(task3).await.unwrap();
        
        // Should be FIFO: task-1, task-2, task-3
        let next = scheduler.peek_next().await;
        assert_eq!(next.unwrap().0, "task-1");
        
        let popped = scheduler.pop_next().await;
        assert_eq!(popped.unwrap().0, "task-1");
        
        let next = scheduler.peek_next().await;
        assert_eq!(next.unwrap().0, "task-2");
    }

    #[tokio::test]
    async fn test_queue_position() {
        let scheduler = DownloadScheduler::new();
        
        let task = DownloadTask {
            id: "pos-test".to_string(),
            url: "https://example.com".to_string(),
            filename: "test.zip".to_string(),
            save_location: "/downloads".to_string(),
        };
        
        scheduler.enqueue(task).await.unwrap();
        
        let pos = scheduler.get_position("pos-test").await;
        assert_eq!(pos, Some(1));
    }

    #[tokio::test]
    async fn test_cancel_queued_download() {
        let scheduler = DownloadScheduler::new();
        
        let task = DownloadTask {
            id: "cancel-test".to_string(),
            url: "https://example.com".to_string(),
            filename: "test.zip".to_string(),
            save_location: "/downloads".to_string(),
        };
        
        scheduler.enqueue(task).await.unwrap();
        
        // Cancel should remove from queue
        let cancelled = scheduler.dequeue("cancel-test").await;
        assert!(cancelled.is_some());
        
        // Should be empty now
        assert!(scheduler.is_empty().await);
    }

    #[tokio::test]
    async fn test_max_concurrent_limit() {
        let scheduler = DownloadScheduler::with_max_concurrent(2);
        
        assert_eq!(scheduler.max_concurrent(), 2);
        assert!(scheduler.can_start(0).await);
        assert!(scheduler.can_start(1).await);
        assert!(!scheduler.can_start(2).await);
    }
}