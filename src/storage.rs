use anyhow::{Context, Result};
use crate::models::{LogEntry, OperationLogs, ScheduledOperation, ScheduledOperations};
use std::fs;
use std::path::PathBuf;

const SCHEDULED_FILE: &str = "scheduled.json";
const LOGS_FILE: &str = "logs.json";
const PID_FILE: &str = "daemon.pid";

// get the storage directory, creating it if needed
// macOS: ~/Library/Application Support/git-delayed
// Linux: ~/.config/git-delayed
pub fn get_storage_dir() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("no config dir")?
        .join("git-delayed");

    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    Ok(dir)
}

pub fn get_scheduled_file_path() -> Result<PathBuf> {
    Ok(get_storage_dir()?.join(SCHEDULED_FILE))
}

pub fn get_logs_file_path() -> Result<PathBuf> {
    Ok(get_storage_dir()?.join(LOGS_FILE))
}

pub fn get_pid_file_path() -> Result<PathBuf> {
    Ok(get_storage_dir()?.join(PID_FILE))
}

use fs2::FileExt;
use std::fs::File;
use std::thread;
use std::time::Duration;

// try to get an exclusive lock on a file, with exponential backoff
// gives up after 3 attempts
pub fn with_file_lock<F, T>(file: &File, operation: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    for attempt in 0..3 {
        if file.try_lock_exclusive().is_ok() {
            let result = operation();
            let _ = FileExt::unlock(file);
            return result;
        }
        // wait a bit longer each time
        thread::sleep(Duration::from_millis(100 * (1 << (attempt + 1))));
    }
    Err(anyhow::anyhow!("couldn't acquire file lock"))
}

pub fn load_scheduled_operations() -> Result<ScheduledOperations> {
    let path = get_scheduled_file_path()?;
    if !path.exists() {
        return Ok(ScheduledOperations::default());
    }
    
    let content = fs::read_to_string(&path)?;
    if content.trim().is_empty() {
        return Ok(ScheduledOperations::default());
    }
    
    Ok(serde_json::from_str(&content)?)
}

pub fn save_scheduled_operations(operations: &ScheduledOperations) -> Result<()> {
    let path = get_scheduled_file_path()?;
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)?;
    
    with_file_lock(&file, || {
        let content = serde_json::to_string_pretty(operations)?;
        fs::write(&path, content)?;
        Ok(())
    })
}

pub fn add_scheduled_operation(operation: ScheduledOperation) -> Result<()> {
    let mut operations = load_scheduled_operations()?;
    operations.operations.push(operation);
    save_scheduled_operations(&operations)
}

pub fn remove_scheduled_operation(operation_id: &str) -> Result<bool> {
    let mut operations = load_scheduled_operations()?;
    let initial_len = operations.operations.len();
    operations.operations.retain(|op| op.id != operation_id);
    
    if operations.operations.len() < initial_len {
        save_scheduled_operations(&operations)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn load_logs() -> Result<OperationLogs> {
    let path = get_logs_file_path()?;
    if !path.exists() {
        return Ok(OperationLogs::default());
    }
    
    let content = fs::read_to_string(&path)?;
    if content.trim().is_empty() {
        return Ok(OperationLogs::default());
    }
    
    Ok(serde_json::from_str(&content)?)
}

pub fn append_log_entry(entry: LogEntry) -> Result<()> {
    let mut logs = load_logs()?;
    logs.entries.push(entry);
    
    let path = get_logs_file_path()?;
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)?;
    
    with_file_lock(&file, || {
        let content = serde_json::to_string_pretty(&logs)?;
        fs::write(&path, content)?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;
    use std::path::PathBuf;

    #[test]
    fn test_storage_dir_exists() {
        let result = get_storage_dir();
        assert!(result.is_ok());
        let dir = result.unwrap();
        assert!(dir.exists());
    }

    #[test]
    fn test_load_empty_operations() {
        let ops = load_scheduled_operations().unwrap();
        assert!(ops.operations.len() >= 0);
    }

    #[test]
    fn test_add_and_remove_operation() {
        let op = ScheduledOperation {
            id: "test-123".to_string(),
            repository_path: PathBuf::from("/tmp/test"),
            operation_type: crate::models::OperationType::Commit,
            commit_message: "test".to_string(),
            scheduled_time: Local::now(),
            created_at: Local::now(),
            retry_count: 0,
        };

        add_scheduled_operation(op).unwrap();
        let removed = remove_scheduled_operation("test-123").unwrap();
        assert!(removed);
    }

    #[test]
    fn test_remove_nonexistent() {
        let removed = remove_scheduled_operation("does-not-exist").unwrap();
        assert!(!removed);
    }
}
