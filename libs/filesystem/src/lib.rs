//! Filesystem Abstraction for hairr OS
//! 
//! Provides a virtual filesystem layer that supports multiple filesystem types
//! and allows for easy integration of new filesystems.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// File type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Regular,
    Directory,
    Symlink,
    Device,
    Socket,
    Pipe,
}

/// File permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilePermissions {
    pub owner_read: bool,
    pub owner_write: bool,
    pub owner_execute: bool,
    pub group_read: bool,
    pub group_write: bool,
    pub group_execute: bool,
    pub other_read: bool,
    pub other_write: bool,
    pub other_execute: bool,
}

impl FilePermissions {
    pub fn new(mode: u32) -> Self {
        FilePermissions {
            owner_read: (mode & 0o400) != 0,
            owner_write: (mode & 0o200) != 0,
            owner_execute: (mode & 0o100) != 0,
            group_read: (mode & 0o040) != 0,
            group_write: (mode & 0o020) != 0,
            group_execute: (mode & 0o010) != 0,
            other_read: (mode & 0o004) != 0,
            other_write: (mode & 0o002) != 0,
            other_execute: (mode & 0o001) != 0,
        }
    }

    pub fn default_file() -> Self {
        Self::new(0o644)
    }

    pub fn default_directory() -> Self {
        Self::new(0o755)
    }

    pub fn to_mode(&self) -> u32 {
        let mut mode = 0;
        if self.owner_read { mode |= 0o400; }
        if self.owner_write { mode |= 0o200; }
        if self.owner_execute { mode |= 0o100; }
        if self.group_read { mode |= 0o040; }
        if self.group_write { mode |= 0o020; }
        if self.group_execute { mode |= 0o010; }
        if self.other_read { mode |= 0o004; }
        if self.other_write { mode |= 0o002; }
        if self.other_execute { mode |= 0o001; }
        mode
    }
}

/// File metadata
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub file_type: FileType,
    pub size: u64,
    pub permissions: FilePermissions,
    pub created_at: u64,
    pub modified_at: u64,
    pub accessed_at: u64,
    pub owner_id: u32,
    pub group_id: u32,
}

impl FileMetadata {
    pub fn new(file_type: FileType) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let permissions = match file_type {
            FileType::Directory => FilePermissions::default_directory(),
            _ => FilePermissions::default_file(),
        };

        FileMetadata {
            file_type,
            size: 0,
            permissions,
            created_at: now,
            modified_at: now,
            accessed_at: now,
            owner_id: 0,
            group_id: 0,
        }
    }

    pub fn is_file(&self) -> bool {
        self.file_type == FileType::Regular
    }

    pub fn is_directory(&self) -> bool {
        self.file_type == FileType::Directory
    }
}

/// File handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileHandle(u64);

impl FileHandle {
    pub fn new(id: u64) -> Self {
        FileHandle(id)
    }
}

/// File open options
#[derive(Debug, Clone, Copy)]
pub struct OpenOptions {
    pub read: bool,
    pub write: bool,
    pub create: bool,
    pub truncate: bool,
    pub append: bool,
}

impl OpenOptions {
    pub fn read_only() -> Self {
        OpenOptions {
            read: true,
            write: false,
            create: false,
            truncate: false,
            append: false,
        }
    }

    pub fn write_only() -> Self {
        OpenOptions {
            read: false,
            write: true,
            create: true,
            truncate: false,
            append: false,
        }
    }

    pub fn read_write() -> Self {
        OpenOptions {
            read: true,
            write: true,
            create: true,
            truncate: false,
            append: false,
        }
    }
}

/// In-memory file node
#[derive(Debug, Clone)]
struct FileNode {
    path: PathBuf,
    metadata: FileMetadata,
    content: Vec<u8>,
    children: Vec<PathBuf>,
}

impl FileNode {
    fn new(path: PathBuf, file_type: FileType) -> Self {
        FileNode {
            path,
            metadata: FileMetadata::new(file_type),
            content: Vec::new(),
            children: Vec::new(),
        }
    }
}

/// Open file descriptor
#[derive(Debug, Clone)]
struct OpenFile {
    handle: FileHandle,
    path: PathBuf,
    options: OpenOptions,
    position: usize,
}

/// Virtual Filesystem
pub struct VirtualFileSystem {
    root: PathBuf,
    nodes: Arc<Mutex<HashMap<PathBuf, FileNode>>>,
    open_files: Arc<Mutex<HashMap<FileHandle, OpenFile>>>,
    next_handle: Arc<Mutex<u64>>,
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        let mut fs = VirtualFileSystem {
            root: PathBuf::from("/"),
            nodes: Arc::new(Mutex::new(HashMap::new())),
            open_files: Arc::new(Mutex::new(HashMap::new())),
            next_handle: Arc::new(Mutex::new(1)),
        };

        // Create root directory
        let root = FileNode::new(PathBuf::from("/"), FileType::Directory);
        fs.nodes.lock().unwrap().insert(PathBuf::from("/"), root);

        fs
    }

    /// Create a new file
    pub fn create_file(&self, path: &Path) -> Result<(), String> {
        let mut nodes = self.nodes.lock().unwrap();
        
        if nodes.contains_key(path) {
            return Err("File already exists".to_string());
        }

        // Check if parent directory exists
        if let Some(parent) = path.parent() {
            if !nodes.contains_key(parent) {
                return Err("Parent directory does not exist".to_string());
            }

            // Add to parent's children
            if let Some(parent_node) = nodes.get_mut(parent) {
                if !parent_node.metadata.is_directory() {
                    return Err("Parent is not a directory".to_string());
                }
                parent_node.children.push(path.to_path_buf());
            }
        }

        let node = FileNode::new(path.to_path_buf(), FileType::Regular);
        nodes.insert(path.to_path_buf(), node);

        Ok(())
    }

    /// Create a new directory
    pub fn create_directory(&self, path: &Path) -> Result<(), String> {
        let mut nodes = self.nodes.lock().unwrap();
        
        if nodes.contains_key(path) {
            return Err("Directory already exists".to_string());
        }

        // Check if parent directory exists
        if let Some(parent) = path.parent() {
            if !nodes.contains_key(parent) {
                return Err("Parent directory does not exist".to_string());
            }

            // Add to parent's children
            if let Some(parent_node) = nodes.get_mut(parent) {
                if !parent_node.metadata.is_directory() {
                    return Err("Parent is not a directory".to_string());
                }
                parent_node.children.push(path.to_path_buf());
            }
        }

        let node = FileNode::new(path.to_path_buf(), FileType::Directory);
        nodes.insert(path.to_path_buf(), node);

        Ok(())
    }

    /// Open a file
    pub fn open(&self, path: &Path, options: OpenOptions) -> Result<FileHandle, String> {
        let nodes = self.nodes.lock().unwrap();
        
        if !nodes.contains_key(path) {
            if options.create {
                drop(nodes);
                self.create_file(path)?;
            } else {
                return Err("File not found".to_string());
            }
        }

        let mut next_handle = self.next_handle.lock().unwrap();
        let handle = FileHandle(*next_handle);
        *next_handle += 1;

        let open_file = OpenFile {
            handle,
            path: path.to_path_buf(),
            options,
            position: 0,
        };

        self.open_files.lock().unwrap().insert(handle, open_file);

        Ok(handle)
    }

    /// Close a file
    pub fn close(&self, handle: FileHandle) -> Result<(), String> {
        self.open_files.lock().unwrap().remove(&handle)
            .ok_or("Invalid file handle".to_string())?;
        Ok(())
    }

    /// Read from a file
    pub fn read(&self, handle: FileHandle, buffer: &mut [u8]) -> Result<usize, String> {
        let mut open_files = self.open_files.lock().unwrap();
        let open_file = open_files.get_mut(&handle)
            .ok_or("Invalid file handle")?;

        if !open_file.options.read {
            return Err("File not opened for reading".to_string());
        }

        let nodes = self.nodes.lock().unwrap();
        let node = nodes.get(&open_file.path)
            .ok_or("File not found")?;

        let available = node.content.len().saturating_sub(open_file.position);
        let to_read = available.min(buffer.len());

        if to_read > 0 {
            buffer[..to_read].copy_from_slice(
                &node.content[open_file.position..open_file.position + to_read]
            );
            open_file.position += to_read;
        }

        Ok(to_read)
    }

    /// Write to a file
    pub fn write(&self, handle: FileHandle, data: &[u8]) -> Result<usize, String> {
        let mut open_files = self.open_files.lock().unwrap();
        let open_file = open_files.get_mut(&handle)
            .ok_or("Invalid file handle")?;

        if !open_file.options.write {
            return Err("File not opened for writing".to_string());
        }

        let mut nodes = self.nodes.lock().unwrap();
        let node = nodes.get_mut(&open_file.path)
            .ok_or("File not found")?;

        if open_file.options.truncate && open_file.position == 0 {
            node.content.clear();
        }

        if open_file.options.append {
            node.content.extend_from_slice(data);
            open_file.position = node.content.len();
        } else {
            // Ensure content is large enough
            if open_file.position + data.len() > node.content.len() {
                node.content.resize(open_file.position + data.len(), 0);
            }
            
            node.content[open_file.position..open_file.position + data.len()]
                .copy_from_slice(data);
            open_file.position += data.len();
        }

        node.metadata.size = node.content.len() as u64;
        node.metadata.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(data.len())
    }

    /// Get file metadata
    pub fn metadata(&self, path: &Path) -> Result<FileMetadata, String> {
        let nodes = self.nodes.lock().unwrap();
        let node = nodes.get(path).ok_or("File not found")?;
        Ok(node.metadata.clone())
    }

    /// List directory contents
    pub fn list_directory(&self, path: &Path) -> Result<Vec<PathBuf>, String> {
        let nodes = self.nodes.lock().unwrap();
        let node = nodes.get(path).ok_or("Directory not found")?;

        if !node.metadata.is_directory() {
            return Err("Not a directory".to_string());
        }

        Ok(node.children.clone())
    }

    /// Delete a file or empty directory
    pub fn delete(&self, path: &Path) -> Result<(), String> {
        let mut nodes = self.nodes.lock().unwrap();
        
        let node = nodes.get(path).ok_or("File not found")?;
        
        if node.metadata.is_directory() && !node.children.is_empty() {
            return Err("Directory not empty".to_string());
        }

        // Remove from parent's children list
        if let Some(parent) = path.parent() {
            if let Some(parent_node) = nodes.get_mut(parent) {
                parent_node.children.retain(|p| p != path);
            }
        }

        nodes.remove(path);
        Ok(())
    }

    /// Check if a path exists
    pub fn exists(&self, path: &Path) -> bool {
        self.nodes.lock().unwrap().contains_key(path)
    }

    /// Get filesystem statistics
    pub fn stats(&self) -> FilesystemStats {
        let nodes = self.nodes.lock().unwrap();
        let total_files = nodes.values().filter(|n| n.metadata.is_file()).count();
        let total_dirs = nodes.values().filter(|n| n.metadata.is_directory()).count();
        let total_size: u64 = nodes.values().map(|n| n.metadata.size).sum();

        FilesystemStats {
            total_files,
            total_directories: total_dirs,
            total_size,
        }
    }
}

impl Default for VirtualFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Filesystem statistics
#[derive(Debug, Clone)]
pub struct FilesystemStats {
    pub total_files: usize,
    pub total_directories: usize,
    pub total_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_file() {
        let fs = VirtualFileSystem::new();
        assert!(fs.create_file(Path::new("/test.txt")).is_ok());
        assert!(fs.exists(Path::new("/test.txt")));
    }

    #[test]
    fn test_create_directory() {
        let fs = VirtualFileSystem::new();
        assert!(fs.create_directory(Path::new("/dir")).is_ok());
        assert!(fs.exists(Path::new("/dir")));
        
        let metadata = fs.metadata(Path::new("/dir")).unwrap();
        assert!(metadata.is_directory());
    }

    #[test]
    fn test_file_operations() {
        let fs = VirtualFileSystem::new();
        fs.create_file(Path::new("/test.txt")).unwrap();
        
        let handle = fs.open(Path::new("/test.txt"), OpenOptions::read_write()).unwrap();
        
        let data = b"Hello, hairr OS!";
        let written = fs.write(handle, data).unwrap();
        assert_eq!(written, data.len());
        
        fs.close(handle).unwrap();
        
        let handle = fs.open(Path::new("/test.txt"), OpenOptions::read_only()).unwrap();
        let mut buffer = vec![0u8; data.len()];
        let read = fs.read(handle, &mut buffer).unwrap();
        assert_eq!(read, data.len());
        assert_eq!(&buffer, data);
        
        fs.close(handle).unwrap();
    }

    #[test]
    fn test_list_directory() {
        let fs = VirtualFileSystem::new();
        fs.create_directory(Path::new("/dir")).unwrap();
        fs.create_file(Path::new("/dir/file1.txt")).unwrap();
        fs.create_file(Path::new("/dir/file2.txt")).unwrap();
        
        let entries = fs.list_directory(Path::new("/dir")).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_delete_file() {
        let fs = VirtualFileSystem::new();
        fs.create_file(Path::new("/test.txt")).unwrap();
        
        assert!(fs.delete(Path::new("/test.txt")).is_ok());
        assert!(!fs.exists(Path::new("/test.txt")));
    }

    #[test]
    fn test_filesystem_stats() {
        let fs = VirtualFileSystem::new();
        fs.create_directory(Path::new("/dir")).unwrap();
        fs.create_file(Path::new("/dir/file.txt")).unwrap();
        
        let stats = fs.stats();
        assert_eq!(stats.total_files, 1);
        assert_eq!(stats.total_directories, 2); // root + /dir
    }
}
