//! hairr OS Microkernel
//! 
//! The core microkernel providing fundamental OS services including:
//! - Process and thread management
//! - Memory management
//! - IPC facilitation
//! - Capability-based security enforcement

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Process identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessId(u64);

impl ProcessId {
    pub fn new(id: u64) -> Self {
        ProcessId(id)
    }
}

/// Thread identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThreadId(u64);

impl ThreadId {
    pub fn new(id: u64) -> Self {
        ThreadId(id)
    }
}

/// Process state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

/// Process priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    RealTime,
    High,
    Normal,
    Low,
}

/// Process control block
#[derive(Debug, Clone)]
pub struct Process {
    pub id: ProcessId,
    pub name: String,
    pub state: ProcessState,
    pub priority: Priority,
    pub parent: Option<ProcessId>,
}

impl Process {
    pub fn new(id: ProcessId, name: String, priority: Priority) -> Self {
        Process {
            id,
            name,
            state: ProcessState::Ready,
            priority,
            parent: None,
        }
    }
}

/// The microkernel itself
pub struct Kernel {
    processes: Arc<Mutex<HashMap<ProcessId, Process>>>,
    next_process_id: Arc<Mutex<u64>>,
}

impl Kernel {
    pub fn new() -> Self {
        Kernel {
            processes: Arc::new(Mutex::new(HashMap::new())),
            next_process_id: Arc::new(Mutex::new(1)),
        }
    }

    /// Create a new process
    pub fn create_process(&self, name: String, priority: Priority) -> ProcessId {
        let mut next_id = self.next_process_id.lock().unwrap();
        let process_id = ProcessId(*next_id);
        *next_id += 1;

        let process = Process::new(process_id, name, priority);
        self.processes.lock().unwrap().insert(process_id, process);
        
        process_id
    }

    /// Get process information
    pub fn get_process(&self, id: ProcessId) -> Option<Process> {
        self.processes.lock().unwrap().get(&id).cloned()
    }

    /// Terminate a process
    pub fn terminate_process(&self, id: ProcessId) -> Result<(), String> {
        let mut processes = self.processes.lock().unwrap();
        if let Some(process) = processes.get_mut(&id) {
            process.state = ProcessState::Terminated;
            Ok(())
        } else {
            Err("Process not found".to_string())
        }
    }

    /// Update process state
    pub fn update_process_state(&self, id: ProcessId, state: ProcessState) -> Result<(), String> {
        let mut processes = self.processes.lock().unwrap();
        if let Some(process) = processes.get_mut(&id) {
            process.state = state;
            Ok(())
        } else {
            Err("Process not found".to_string())
        }
    }

    /// List all processes
    pub fn list_processes(&self) -> Vec<Process> {
        self.processes.lock().unwrap().values().cloned().collect()
    }

    /// Get process count
    pub fn process_count(&self) -> usize {
        self.processes.lock().unwrap().len()
    }
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_creation() {
        let kernel = Kernel::new();
        let pid = kernel.create_process("test_process".to_string(), Priority::Normal);
        
        let process = kernel.get_process(pid);
        assert!(process.is_some());
        assert_eq!(process.unwrap().name, "test_process");
    }

    #[test]
    fn test_process_termination() {
        let kernel = Kernel::new();
        let pid = kernel.create_process("test_process".to_string(), Priority::Normal);
        
        assert!(kernel.terminate_process(pid).is_ok());
        let process = kernel.get_process(pid).unwrap();
        assert_eq!(process.state, ProcessState::Terminated);
    }

    #[test]
    fn test_process_listing() {
        let kernel = Kernel::new();
        kernel.create_process("process1".to_string(), Priority::Normal);
        kernel.create_process("process2".to_string(), Priority::High);
        
        assert_eq!(kernel.process_count(), 2);
    }
}
