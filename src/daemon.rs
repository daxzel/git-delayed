use anyhow::{Context, Result};
use chrono::{Duration as ChronoDuration, Local};
use daemonize::Daemonize;
use std::fs;
use std::fs::File;
use std::thread;
use std::time::Duration;

use crate::executor;
use crate::models::{ExecutionStatus, LogEntry, OperationType};
use crate::storage;

pub fn write_pid_file(pid: u32) -> Result<()> {
    fs::write(storage::get_pid_file_path()?, pid.to_string())?;
    Ok(())
}

pub fn read_pid_file() -> Result<u32> {
    let content = fs::read_to_string(storage::get_pid_file_path()?)?;
    Ok(content.trim().parse()?)
}

pub fn delete_pid_file() -> Result<()> {
    let path = storage::get_pid_file_path()?;
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}

pub fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        kill(Pid::from_raw(pid as i32), Signal::SIGCONT).is_ok()
    }
    
    #[cfg(not(unix))]
    false
}

pub fn is_daemon_running() -> Result<bool> {
    match read_pid_file() {
        Ok(pid) => Ok(is_process_running(pid)),
        Err(_) => Ok(false),
    }
}

pub fn run_daemon_loop() -> Result<()> {
    loop {
        let now = Local::now();
        let mut operations = storage::load_scheduled_operations()?;
        
        // sort by scheduled time to process in order
        operations.operations.sort_by_key(|op| op.scheduled_time);
        
        // process only the first due operation, then break
        // this ensures sequential execution
        for mut operation in operations.operations {
            if operation.scheduled_time <= now {
                storage::remove_scheduled_operation(&operation.id)?;
                
                // handle push operations specially
                if operation.operation_type == OperationType::Push {
                    match executor::execute_push_with_branch(
                        &operation.repository_path,
                        operation.branch.as_deref(),
                    ) {
                        Ok(executor::PushResult::Success(_)) => {
                            storage::append_log_entry(LogEntry {
                                id: operation.id,
                                repository_path: operation.repository_path,
                                operation_type: operation.operation_type,
                                commit_message: operation.commit_message,
                                scheduled_time: operation.scheduled_time,
                                executed_at: Local::now(),
                                status: ExecutionStatus::Success,
                                error_message: None,
                            })?;
                        }
                        Ok(executor::PushResult::NothingToPush) => {
                            storage::append_log_entry(LogEntry {
                                id: operation.id,
                                repository_path: operation.repository_path,
                                operation_type: operation.operation_type,
                                commit_message: operation.commit_message,
                                scheduled_time: operation.scheduled_time,
                                executed_at: Local::now(),
                                status: ExecutionStatus::Skipped,
                                error_message: Some("nothing to push".to_string()),
                            })?;
                        }
                        Err(e) => {
                            operation.retry_count += 1;
                            operation.state = crate::models::OperationState::Failing;
                            operation.scheduled_time = Local::now() + ChronoDuration::minutes(10);
                            
                            storage::append_log_entry(LogEntry {
                                id: operation.id.clone(),
                                repository_path: operation.repository_path.clone(),
                                operation_type: operation.operation_type.clone(),
                                commit_message: format!("{} (retry {})", operation.commit_message, operation.retry_count),
                                scheduled_time: operation.scheduled_time,
                                executed_at: Local::now(),
                                status: ExecutionStatus::Failure,
                                error_message: Some(format!("retry {}: {}", operation.retry_count, e)),
                            })?;
                            
                            storage::add_scheduled_operation(operation)?;
                        }
                    }
                } else {
                    // handle commit operations
                    match executor::execute_commit(&operation.repository_path, &operation.commit_message) {
                        Ok(_) => {
                            storage::append_log_entry(LogEntry {
                                id: operation.id,
                                repository_path: operation.repository_path,
                                operation_type: operation.operation_type,
                                commit_message: operation.commit_message,
                                scheduled_time: operation.scheduled_time,
                                executed_at: Local::now(),
                                status: ExecutionStatus::Success,
                                error_message: None,
                            })?;
                        }
                        Err(e) => {
                            operation.retry_count += 1;
                            operation.state = crate::models::OperationState::Failing;
                            operation.scheduled_time = Local::now() + ChronoDuration::minutes(10);
                            
                            storage::append_log_entry(LogEntry {
                                id: operation.id.clone(),
                                repository_path: operation.repository_path.clone(),
                                operation_type: operation.operation_type.clone(),
                                commit_message: format!("{} (retry {})", operation.commit_message, operation.retry_count),
                                scheduled_time: operation.scheduled_time,
                                executed_at: Local::now(),
                                status: ExecutionStatus::Failure,
                                error_message: Some(format!("retry {}: {}", operation.retry_count, e)),
                            })?;
                            
                            storage::add_scheduled_operation(operation)?;
                        }
                    }
                }
                
                // only process one operation per loop iteration
                break;
            }
        }
        
        thread::sleep(Duration::from_secs(60));
    }
}

pub fn start_daemon() -> Result<()> {
    if is_daemon_running()? {
        return Err(anyhow::anyhow!("daemon already running (pid {})", read_pid_file()?));
    }
    
    let dir = storage::get_storage_dir()?;
    let daemonize = Daemonize::new()
        .working_directory(&dir)
        .stdout(File::create(dir.join("daemon.out"))?)
        .stderr(File::create(dir.join("daemon.err"))?);
    
    match daemonize.start() {
        Ok(_) => {
            write_pid_file(std::process::id())?;
            run_daemon_loop()
        }
        Err(e) => Err(anyhow::anyhow!("daemonize failed: {}", e)),
    }
}

pub fn stop_daemon() -> Result<()> {
    if !is_daemon_running()? {
        return Err(anyhow::anyhow!("daemon not running"));
    }
    
    let pid = read_pid_file()?;
    
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        
        kill(Pid::from_raw(pid as i32), Signal::SIGTERM)?;
        
        for _ in 0..10 {
            thread::sleep(Duration::from_millis(500));
            if !is_process_running(pid) {
                break;
            }
        }
        
        if is_process_running(pid) {
            return Err(anyhow::anyhow!("daemon didn't stop"));
        }
    }
    
    delete_pid_file()?;
    Ok(())
}
