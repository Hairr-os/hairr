//! AI-Aware Scheduler Service
//! 
//! Provides intelligent workload scheduling optimized for AI/ML tasks,
//! with support for mixed-criticality real-time and batch workloads.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

/// Process identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessId(u64);

impl ProcessId {
    pub fn new(id: u64) -> Self {
        ProcessId(id)
    }
}

/// Workload types that the scheduler can handle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadType {
    /// Real-time critical workload with deadline guarantees
    RealTime,
    /// AI/ML inference workload
    AIInference,
    /// AI/ML training workload
    AITraining,
    /// Interactive user-facing workload
    Interactive,
    /// Batch processing workload
    Batch,
}

/// Scheduling priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SchedulingPriority {
    Critical = 4,
    High = 3,
    Normal = 2,
    Low = 1,
    Background = 0,
}

/// Task information for scheduling
#[derive(Debug, Clone)]
pub struct Task {
    pub id: ProcessId,
    pub workload_type: WorkloadType,
    pub priority: SchedulingPriority,
    pub cpu_time_used: u64,
    pub deadline: Option<u64>,
    pub ai_accelerator_required: bool,
}

impl Task {
    pub fn new(id: ProcessId, workload_type: WorkloadType) -> Self {
        let priority = match workload_type {
            WorkloadType::RealTime => SchedulingPriority::Critical,
            WorkloadType::Interactive => SchedulingPriority::High,
            WorkloadType::AIInference => SchedulingPriority::High,
            WorkloadType::AITraining => SchedulingPriority::Normal,
            WorkloadType::Batch => SchedulingPriority::Low,
        };

        Task {
            id,
            workload_type,
            priority,
            cpu_time_used: 0,
            deadline: None,
            ai_accelerator_required: matches!(workload_type, WorkloadType::AIInference | WorkloadType::AITraining),
        }
    }

    pub fn with_deadline(mut self, deadline: u64) -> Self {
        self.deadline = Some(deadline);
        self
    }
}

/// AI-aware scheduler
pub struct AIScheduler {
    ready_queue: Arc<Mutex<VecDeque<Task>>>,
    tasks: Arc<Mutex<HashMap<ProcessId, Task>>>,
    ai_accelerator_available: Arc<Mutex<bool>>,
}

impl AIScheduler {
    pub fn new() -> Self {
        AIScheduler {
            ready_queue: Arc::new(Mutex::new(VecDeque::new())),
            tasks: Arc::new(Mutex::new(HashMap::new())),
            ai_accelerator_available: Arc::new(Mutex::new(true)),
        }
    }

    /// Add a task to the scheduler
    pub fn add_task(&self, task: Task) {
        let task_id = task.id;
        self.tasks.lock().unwrap().insert(task_id, task.clone());
        
        // Insert into ready queue based on priority
        let mut queue = self.ready_queue.lock().unwrap();
        
        // Find the correct position based on priority
        let pos = queue.iter().position(|t| t.priority < task.priority).unwrap_or(queue.len());
        queue.insert(pos, task);
    }

    /// Remove a task from the scheduler
    pub fn remove_task(&self, id: ProcessId) -> Option<Task> {
        self.tasks.lock().unwrap().remove(&id)
    }

    /// Get the next task to execute
    pub fn next_task(&self) -> Option<Task> {
        let mut queue = self.ready_queue.lock().unwrap();
        
        // Check for real-time tasks first
        if let Some(pos) = queue.iter().position(|t| matches!(t.workload_type, WorkloadType::RealTime)) {
            return Some(queue.remove(pos).unwrap());
        }

        // Check for AI tasks if accelerator is available
        let accelerator_available = *self.ai_accelerator_available.lock().unwrap();
        if accelerator_available {
            if let Some(pos) = queue.iter().position(|t| t.ai_accelerator_required) {
                *self.ai_accelerator_available.lock().unwrap() = false;
                return Some(queue.remove(pos).unwrap());
            }
        }

        // Otherwise, return highest priority task
        queue.pop_front()
    }

    /// Mark a task as completed
    pub fn complete_task(&self, id: ProcessId) {
        if let Some(task) = self.tasks.lock().unwrap().get(&id) {
            if task.ai_accelerator_required {
                *self.ai_accelerator_available.lock().unwrap() = true;
            }
        }
        self.remove_task(id);
    }

    /// Get task information
    pub fn get_task(&self, id: ProcessId) -> Option<Task> {
        self.tasks.lock().unwrap().get(&id).cloned()
    }

    /// List all tasks
    pub fn list_tasks(&self) -> Vec<Task> {
        self.tasks.lock().unwrap().values().cloned().collect()
    }

    /// Update task CPU time
    pub fn update_cpu_time(&self, id: ProcessId, time: u64) {
        if let Some(task) = self.tasks.lock().unwrap().get_mut(&id) {
            task.cpu_time_used += time;
        }
    }

    /// Check for deadline violations
    pub fn check_deadlines(&self, current_time: u64) -> Vec<ProcessId> {
        self.tasks
            .lock()
            .unwrap()
            .values()
            .filter(|t| {
                if let Some(deadline) = t.deadline {
                    current_time > deadline
                } else {
                    false
                }
            })
            .map(|t| t.id)
            .collect()
    }
}

impl Default for AIScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(ProcessId::new(1), WorkloadType::AIInference);
        assert_eq!(task.priority, SchedulingPriority::High);
        assert!(task.ai_accelerator_required);
    }

    #[test]
    fn test_scheduler_add_and_next() {
        let scheduler = AIScheduler::new();
        let task1 = Task::new(ProcessId::new(1), WorkloadType::Interactive);
        let task2 = Task::new(ProcessId::new(2), WorkloadType::Batch);
        
        scheduler.add_task(task1);
        scheduler.add_task(task2);
        
        let next = scheduler.next_task();
        assert!(next.is_some());
        assert_eq!(next.unwrap().id, ProcessId::new(1)); // Interactive has higher priority
    }

    #[test]
    fn test_realtime_priority() {
        let scheduler = AIScheduler::new();
        let task1 = Task::new(ProcessId::new(1), WorkloadType::Batch);
        let task2 = Task::new(ProcessId::new(2), WorkloadType::RealTime);
        
        scheduler.add_task(task1);
        scheduler.add_task(task2);
        
        let next = scheduler.next_task();
        assert_eq!(next.unwrap().id, ProcessId::new(2)); // Real-time always first
    }

    #[test]
    fn test_deadline_checking() {
        let scheduler = AIScheduler::new();
        let task = Task::new(ProcessId::new(1), WorkloadType::RealTime).with_deadline(100);
        scheduler.add_task(task);
        
        let violations = scheduler.check_deadlines(150);
        assert_eq!(violations.len(), 1);
    }
}
