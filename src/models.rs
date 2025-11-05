use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum OperationType {
    Commit,
    Push,
}

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationType::Commit => write!(f, "commit"),
            OperationType::Push => write!(f, "push"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum OperationState {
    Pending,
    Failing,
}

impl fmt::Display for OperationState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationState::Pending => write!(f, "pending"),
            OperationState::Failing => write!(f, "failing"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScheduledOperation {
    pub id: String,
    pub repository_path: PathBuf,
    pub operation_type: OperationType,
    pub commit_message: String,
    pub scheduled_time: DateTime<Local>,
    pub created_at: DateTime<Local>,
    #[serde(default)]
    pub retry_count: u32,
    #[serde(default)]
    pub state: OperationState,
}

impl Default for OperationState {
    fn default() -> Self {
        OperationState::Pending
    }
}

impl fmt::Display for ScheduledOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} | {} | {} | {} | {}",
            self.id,
            self.scheduled_time.format("%Y-%m-%d %H:%M:%S"),
            self.operation_type,
            self.repository_path.display(),
            self.commit_message
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ExecutionStatus {
    Success,
    Failure,
    Cancelled,
}

impl fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionStatus::Success => write!(f, "Success"),
            ExecutionStatus::Failure => write!(f, "Failure"),
            ExecutionStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogEntry {
    pub id: String,
    pub repository_path: PathBuf,
    pub operation_type: OperationType,
    pub commit_message: String,
    pub scheduled_time: DateTime<Local>,
    pub executed_at: DateTime<Local>,
    pub status: ExecutionStatus,
    pub error_message: Option<String>,
}

impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_str = self
            .error_message
            .as_ref()
            .map(|e| format!(" | Error: {}", e))
            .unwrap_or_default();
        write!(
            f,
            "{} | {} | {} | {} | {}{}",
            self.executed_at.format("%Y-%m-%d %H:%M:%S"),
            self.status,
            self.repository_path.display(),
            self.commit_message,
            self.id,
            error_str
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScheduledOperations {
    pub operations: Vec<ScheduledOperation>,
}

impl Default for ScheduledOperations {
    fn default() -> Self {
        Self {
            operations: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OperationLogs {
    pub entries: Vec<LogEntry>,
}

impl Default for OperationLogs {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}
