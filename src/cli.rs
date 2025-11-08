use anyhow::Result;
use chrono::Local;
use clap::{Parser, Subcommand};
use uuid::Uuid;

use crate::daemon;
use crate::git;
use crate::models::{ExecutionStatus, LogEntry, OperationType, ScheduledOperation};
use crate::schedule;
use crate::storage;

#[derive(Parser)]
#[command(name = "git-delayed")]
#[command(about = "Schedule git commits and pushes for future execution")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Schedule a commit and push for future execution")]
    Schedule {
        #[arg(help = "Time specification (e.g., '+10 hours', 'Monday', '2025-11-04 09:00')")]
        time_spec: String,
        
        #[command(subcommand)]
        action: ScheduleAction,
    },
    
    #[command(about = "List all scheduled operations")]
    List,
    
    #[command(about = "Show execution logs")]
    Logs,
    
    #[command(about = "Cancel a scheduled operation")]
    Cancel {
        #[arg(help = "Operation ID to cancel")]
        operation_id: String,
    },
    
    #[command(about = "Manage the daemon process")]
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
}

#[derive(Subcommand)]
enum ScheduleAction {
    #[command(about = "Schedule a commit (no push)")]
    Commit {
        #[arg(short, long, help = "Commit message")]
        message: String,
    },
    
    #[command(about = "Schedule a push only")]
    Push,
}

#[derive(Subcommand)]
enum DaemonAction {
    #[command(about = "Start the daemon")]
    Start,
    
    #[command(about = "Stop the daemon")]
    Stop,
    
    #[command(about = "Check daemon status")]
    Status,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Schedule { time_spec, action } => match action {
            ScheduleAction::Commit { message } => {
                handle_schedule(&time_spec, OperationType::Commit, &message)
            }
            ScheduleAction::Push => handle_schedule(&time_spec, OperationType::Push, "push"),
        }
        Commands::List => {
            handle_list()
        }
        Commands::Logs => {
            handle_logs()
        }
        Commands::Cancel { operation_id } => {
            handle_cancel(&operation_id)
        }
        Commands::Daemon { action } => match action {
            DaemonAction::Start => handle_daemon_start(),
            DaemonAction::Stop => handle_daemon_stop(),
            DaemonAction::Status => handle_daemon_status(),
        },
    }
}

fn handle_schedule(time_spec: &str, operation_type: OperationType, message: &str) -> Result<()> {
    let repo_path = git::get_repository_path()?;
    let scheduled_time = schedule::parse_time_spec(time_spec)?;
    
    // capture current branch for push operations
    let branch = if operation_type == OperationType::Push {
        Some(git::get_current_branch(&repo_path)?)
    } else {
        None
    };
    
    let operation = ScheduledOperation {
        id: Uuid::new_v4().to_string(),
        repository_path: repo_path.clone(),
        operation_type: operation_type.clone(),
        commit_message: message.to_string(),
        scheduled_time,
        created_at: Local::now(),
        retry_count: 0,
        state: crate::models::OperationState::Pending,
        branch,
    };
    
    storage::add_scheduled_operation(operation.clone())?;
    
    println!("✓ Operation scheduled successfully");
    println!("  ID: {}", operation.id);
    println!("  Type: {}", operation_type);
    println!("  Repository: {}", repo_path.display());
    println!("  Scheduled for: {}", scheduled_time.format("%Y-%m-%d %H:%M:%S"));
    if operation_type == OperationType::Commit {
        println!("  Message: {}", message);
    }
    
    Ok(())
}

fn handle_list() -> Result<()> {
    let mut operations = storage::load_scheduled_operations()?;
    
    if operations.operations.is_empty() {
        println!("No scheduled operations");
        return Ok(());
    }
    
    operations.operations.sort_by_key(|op| op.scheduled_time);
    
    println!("\nScheduled Operations:");
    println!("{:-<130}", "");
    println!(
        "{:<38} | {:<19} | {:<8} | {:<8} | {:<20} | {:<15} | {}",
        "ID", "Scheduled Time", "Type", "State", "Repository", "Branch", "Message"
    );
    println!("{:-<130}", "");
    
    for op in operations.operations {
        let repo_name = op
            .repository_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        let branch_display = op.branch.as_deref().unwrap_or("-");
        
        println!(
            "{:<38} | {} | {:<8} | {:<8} | {:<20} | {:<15} | {}",
            op.id,
            op.scheduled_time.format("%Y-%m-%d %H:%M:%S"),
            op.operation_type,
            op.state,
            repo_name,
            branch_display,
            op.commit_message
        );
    }
    
    println!("{:-<130}", "");
    
    Ok(())
}

fn handle_logs() -> Result<()> {
    let mut logs = storage::load_logs()?;
    
    if logs.entries.is_empty() {
        println!("No execution logs");
        return Ok(());
    }
    
    logs.entries.sort_by(|a, b| b.executed_at.cmp(&a.executed_at));
    
    println!("\nExecution Logs:");
    println!("{:-<120}", "");
    println!("{:<19} | {:<10} | {:<20} | {:<30} | {}", "Executed At", "Status", "Repository", "Message", "ID");
    println!("{:-<120}", "");
    
    for entry in logs.entries {
        let repo_name = entry.repository_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        let status_colored = match entry.status {
            ExecutionStatus::Success => format!("\x1b[32m{}\x1b[0m", entry.status),
            ExecutionStatus::Failure => format!("\x1b[31m{}\x1b[0m", entry.status),
            ExecutionStatus::Cancelled => format!("\x1b[33m{}\x1b[0m", entry.status),
            ExecutionStatus::Skipped => format!("\x1b[36m{}\x1b[0m", entry.status),
        };
        
        println!(
            "{} | {:<10} | {:<20} | {:<30} | {}",
            entry.executed_at.format("%Y-%m-%d %H:%M:%S"),
            status_colored,
            repo_name,
            if entry.commit_message.len() > 30 {
                format!("{}...", &entry.commit_message[..27])
            } else {
                entry.commit_message.clone()
            },
            entry.id
        );
        
        if let Some(error) = entry.error_message {
            println!("  └─ Error: {}", error);
        }
    }
    
    println!("{:-<120}", "");
    
    Ok(())
}

fn handle_cancel(operation_id: &str) -> Result<()> {
    let operations = storage::load_scheduled_operations()?;
    
    let operation = operations.operations.iter()
        .find(|op| op.id == operation_id)
        .ok_or_else(|| anyhow::anyhow!("Operation not found: {}", operation_id))?;
    
    let log_entry = LogEntry {
        id: operation.id.clone(),
        repository_path: operation.repository_path.clone(),
        operation_type: operation.operation_type.clone(),
        commit_message: operation.commit_message.clone(),
        scheduled_time: operation.scheduled_time,
        executed_at: Local::now(),
        status: ExecutionStatus::Cancelled,
        error_message: None,
    };
    
    let removed = storage::remove_scheduled_operation(operation_id)?;
    
    if removed {
        storage::append_log_entry(log_entry)?;
        println!("✓ Operation cancelled: {}", operation_id);
    } else {
        return Err(anyhow::anyhow!("Failed to remove operation: {}", operation_id));
    }
    
    Ok(())
}

fn handle_daemon_start() -> Result<()> {
    daemon::start_daemon()?;
    Ok(())
}

fn handle_daemon_stop() -> Result<()> {
    daemon::stop_daemon()?;
    println!("✓ Daemon stopped successfully");
    Ok(())
}

fn handle_daemon_status() -> Result<()> {
    if daemon::is_daemon_running()? {
        let pid = daemon::read_pid_file()?;
        let operations = storage::load_scheduled_operations()?;
        println!("✓ Daemon is running");
        println!("  PID: {}", pid);
        println!("  Scheduled operations: {}", operations.operations.len());
    } else {
        println!("✗ Daemon is not running");
    }
    Ok(())
}
