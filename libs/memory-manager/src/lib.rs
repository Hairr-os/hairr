//! Memory Manager for hairr OS
//! 
//! Provides memory allocation, paging, and virtual memory management
//! for the hairr OS microkernel.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Memory page size (4KB)
pub const PAGE_SIZE: usize = 4096;

/// Memory address
pub type Address = usize;

/// Memory region
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryRegion {
    pub start: Address,
    pub size: usize,
}

impl MemoryRegion {
    pub fn new(start: Address, size: usize) -> Self {
        MemoryRegion { start, size }
    }

    pub fn end(&self) -> Address {
        self.start + self.size
    }

    pub fn contains(&self, addr: Address) -> bool {
        addr >= self.start && addr < self.end()
    }
}

/// Memory protection flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryProtection {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}

impl MemoryProtection {
    pub fn read_only() -> Self {
        MemoryProtection {
            readable: true,
            writable: false,
            executable: false,
        }
    }

    pub fn read_write() -> Self {
        MemoryProtection {
            readable: true,
            writable: true,
            executable: false,
        }
    }

    pub fn read_execute() -> Self {
        MemoryProtection {
            readable: true,
            writable: false,
            executable: true,
        }
    }
}

/// Virtual memory mapping
#[derive(Debug, Clone)]
pub struct VirtualMapping {
    pub virtual_addr: Address,
    pub physical_addr: Address,
    pub size: usize,
    pub protection: MemoryProtection,
}

/// Process ID for memory management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessId(pub u64);

/// Memory manager
pub struct MemoryManager {
    // Physical memory tracking
    total_memory: usize,
    free_memory: Arc<Mutex<usize>>,
    allocated_regions: Arc<Mutex<HashMap<ProcessId, Vec<MemoryRegion>>>>,
    
    // Virtual memory mappings per process
    virtual_mappings: Arc<Mutex<HashMap<ProcessId, Vec<VirtualMapping>>>>,
    
    // Page allocation tracking
    free_pages: Arc<Mutex<Vec<Address>>>,
    used_pages: Arc<Mutex<HashMap<Address, ProcessId>>>,
}

impl MemoryManager {
    /// Create a new memory manager with specified total memory
    pub fn new(total_memory_mb: usize) -> Self {
        let total_memory = total_memory_mb * 1024 * 1024;
        let num_pages = total_memory / PAGE_SIZE;
        
        // Initialize free pages
        let free_pages: Vec<Address> = (0..num_pages)
            .map(|i| i * PAGE_SIZE)
            .collect();

        MemoryManager {
            total_memory,
            free_memory: Arc::new(Mutex::new(total_memory)),
            allocated_regions: Arc::new(Mutex::new(HashMap::new())),
            virtual_mappings: Arc::new(Mutex::new(HashMap::new())),
            free_pages: Arc::new(Mutex::new(free_pages)),
            used_pages: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Allocate memory for a process
    pub fn allocate(&self, process_id: ProcessId, size: usize) -> Result<MemoryRegion, String> {
        if size == 0 {
            return Err("Cannot allocate zero bytes".to_string());
        }

        let aligned_size = (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let num_pages = aligned_size / PAGE_SIZE;

        let mut free_memory = self.free_memory.lock().unwrap();
        if *free_memory < aligned_size {
            return Err("Out of memory".to_string());
        }

        let mut free_pages = self.free_pages.lock().unwrap();
        if free_pages.len() < num_pages {
            return Err("Out of memory pages".to_string());
        }

        // Allocate consecutive pages
        let start_addr = free_pages.remove(0);
        let mut used_pages = self.used_pages.lock().unwrap();
        used_pages.insert(start_addr, process_id);

        for _ in 1..num_pages {
            let page_addr = free_pages.remove(0);
            used_pages.insert(page_addr, process_id);
        }

        *free_memory -= aligned_size;

        let region = MemoryRegion::new(start_addr, aligned_size);
        
        // Track allocation
        let mut allocated = self.allocated_regions.lock().unwrap();
        allocated.entry(process_id).or_insert_with(Vec::new).push(region);

        Ok(region)
    }

    /// Free memory for a process
    pub fn free(&self, process_id: ProcessId, region: MemoryRegion) -> Result<(), String> {
        let mut allocated = self.allocated_regions.lock().unwrap();
        let regions = allocated.get_mut(&process_id).ok_or("Process not found")?;

        // Find and remove the region
        let pos = regions.iter().position(|r| *r == region)
            .ok_or("Region not found")?;
        regions.remove(pos);

        // Return pages to free pool
        let num_pages = region.size / PAGE_SIZE;
        let mut free_pages = self.free_pages.lock().unwrap();
        let mut used_pages = self.used_pages.lock().unwrap();

        for i in 0..num_pages {
            let page_addr = region.start + (i * PAGE_SIZE);
            used_pages.remove(&page_addr);
            free_pages.push(page_addr);
        }

        *self.free_memory.lock().unwrap() += region.size;

        Ok(())
    }

    /// Free all memory for a process
    pub fn free_all(&self, process_id: ProcessId) -> Result<(), String> {
        let mut allocated = self.allocated_regions.lock().unwrap();
        let regions = allocated.remove(&process_id).ok_or("Process not found")?;

        let mut free_pages = self.free_pages.lock().unwrap();
        let mut used_pages = self.used_pages.lock().unwrap();
        let mut total_freed = 0;

        for region in regions {
            let num_pages = region.size / PAGE_SIZE;
            for i in 0..num_pages {
                let page_addr = region.start + (i * PAGE_SIZE);
                used_pages.remove(&page_addr);
                free_pages.push(page_addr);
            }
            total_freed += region.size;
        }

        *self.free_memory.lock().unwrap() += total_freed;

        // Also remove virtual mappings
        self.virtual_mappings.lock().unwrap().remove(&process_id);

        Ok(())
    }

    /// Create a virtual memory mapping
    pub fn map_virtual(
        &self,
        process_id: ProcessId,
        virtual_addr: Address,
        physical_addr: Address,
        size: usize,
        protection: MemoryProtection,
    ) -> Result<(), String> {
        let mapping = VirtualMapping {
            virtual_addr,
            physical_addr,
            size,
            protection,
        };

        let mut mappings = self.virtual_mappings.lock().unwrap();
        mappings.entry(process_id).or_insert_with(Vec::new).push(mapping);

        Ok(())
    }

    /// Translate virtual address to physical address
    pub fn translate_address(
        &self,
        process_id: ProcessId,
        virtual_addr: Address,
    ) -> Option<Address> {
        let mappings = self.virtual_mappings.lock().unwrap();
        let process_mappings = mappings.get(&process_id)?;

        for mapping in process_mappings {
            if virtual_addr >= mapping.virtual_addr
                && virtual_addr < mapping.virtual_addr + mapping.size
            {
                let offset = virtual_addr - mapping.virtual_addr;
                return Some(mapping.physical_addr + offset);
            }
        }

        None
    }

    /// Get memory statistics
    pub fn stats(&self) -> MemoryStats {
        let free = *self.free_memory.lock().unwrap();
        let used = self.total_memory - free;
        let free_pages = self.free_pages.lock().unwrap().len();
        let used_pages = self.used_pages.lock().unwrap().len();

        MemoryStats {
            total_memory: self.total_memory,
            used_memory: used,
            free_memory: free,
            total_pages: self.total_memory / PAGE_SIZE,
            used_pages,
            free_pages,
        }
    }

    /// Get allocated memory for a process
    pub fn process_memory(&self, process_id: ProcessId) -> usize {
        let allocated = self.allocated_regions.lock().unwrap();
        allocated
            .get(&process_id)
            .map(|regions| regions.iter().map(|r| r.size).sum())
            .unwrap_or(0)
    }

    /// List all processes with memory
    pub fn list_processes(&self) -> Vec<ProcessId> {
        self.allocated_regions.lock().unwrap().keys().copied().collect()
    }
}

/// Memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_memory: usize,
    pub used_memory: usize,
    pub free_memory: usize,
    pub total_pages: usize,
    pub used_pages: usize,
    pub free_pages: usize,
}

impl MemoryStats {
    pub fn usage_percent(&self) -> f32 {
        (self.used_memory as f32 / self.total_memory as f32) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_allocation() {
        let manager = MemoryManager::new(16); // 16MB
        let process_id = ProcessId(1);
        
        let region = manager.allocate(process_id, 4096).unwrap();
        assert_eq!(region.size, 4096);
        
        let stats = manager.stats();
        assert_eq!(stats.used_memory, 4096);
    }

    #[test]
    fn test_memory_free() {
        let manager = MemoryManager::new(16);
        let process_id = ProcessId(1);
        
        let region = manager.allocate(process_id, 4096).unwrap();
        assert!(manager.free(process_id, region).is_ok());
        
        let stats = manager.stats();
        assert_eq!(stats.used_memory, 0);
    }

    #[test]
    fn test_out_of_memory() {
        let manager = MemoryManager::new(1); // 1MB
        let process_id = ProcessId(1);
        
        // Try to allocate more than available
        let result = manager.allocate(process_id, 2 * 1024 * 1024);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_processes() {
        let manager = MemoryManager::new(16);
        let process1 = ProcessId(1);
        let process2 = ProcessId(2);
        
        manager.allocate(process1, 4096).unwrap();
        manager.allocate(process2, 8192).unwrap();
        
        assert_eq!(manager.process_memory(process1), 4096);
        assert_eq!(manager.process_memory(process2), 8192);
    }

    #[test]
    fn test_free_all() {
        let manager = MemoryManager::new(16);
        let process_id = ProcessId(1);
        
        manager.allocate(process_id, 4096).unwrap();
        manager.allocate(process_id, 4096).unwrap();
        
        assert!(manager.free_all(process_id).is_ok());
        
        let stats = manager.stats();
        assert_eq!(stats.used_memory, 0);
    }

    #[test]
    fn test_virtual_mapping() {
        let manager = MemoryManager::new(16);
        let process_id = ProcessId(1);
        
        let region = manager.allocate(process_id, 4096).unwrap();
        
        manager.map_virtual(
            process_id,
            0x10000,
            region.start,
            4096,
            MemoryProtection::read_write(),
        ).unwrap();
        
        let physical = manager.translate_address(process_id, 0x10000);
        assert_eq!(physical, Some(region.start));
    }

    #[test]
    fn test_memory_stats() {
        let manager = MemoryManager::new(16);
        let stats = manager.stats();
        
        assert_eq!(stats.total_memory, 16 * 1024 * 1024);
        assert_eq!(stats.free_memory, stats.total_memory);
        assert!(stats.usage_percent() < 0.01);
    }
}
