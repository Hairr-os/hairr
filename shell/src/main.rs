//! hairr OS Desktop Shell
//! 
//! Provides the basic desktop environment and windowing system for hairr OS.

use std::collections::HashMap;
use std::io::{self, Write};

/// Window identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(u64);

impl WindowId {
    pub fn new(id: u64) -> Self {
        WindowId(id)
    }
}

/// Window state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
    Fullscreen,
}

/// Window information
#[derive(Debug, Clone)]
pub struct Window {
    pub id: WindowId,
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub state: WindowState,
    pub process_id: u64,
}

impl Window {
    pub fn new(id: WindowId, title: String, process_id: u64) -> Self {
        Window {
            id,
            title,
            x: 100,
            y: 100,
            width: 800,
            height: 600,
            state: WindowState::Normal,
            process_id,
        }
    }
}

/// Desktop shell manager
pub struct Shell {
    windows: HashMap<WindowId, Window>,
    next_window_id: u64,
    focused_window: Option<WindowId>,
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            windows: HashMap::new(),
            next_window_id: 1,
            focused_window: None,
        }
    }

    /// Create a new window
    pub fn create_window(&mut self, title: String, process_id: u64) -> WindowId {
        let window_id = WindowId(self.next_window_id);
        self.next_window_id += 1;

        let window = Window::new(window_id, title, process_id);
        self.windows.insert(window_id, window);
        self.focused_window = Some(window_id);
        
        window_id
    }

    /// Close a window
    pub fn close_window(&mut self, id: WindowId) -> Result<(), String> {
        if self.windows.remove(&id).is_some() {
            if self.focused_window == Some(id) {
                self.focused_window = None;
            }
            Ok(())
        } else {
            Err("Window not found".to_string())
        }
    }

    /// Get window information
    pub fn get_window(&self, id: WindowId) -> Option<&Window> {
        self.windows.get(&id)
    }

    /// Set window state
    pub fn set_window_state(&mut self, id: WindowId, state: WindowState) -> Result<(), String> {
        if let Some(window) = self.windows.get_mut(&id) {
            window.state = state;
            Ok(())
        } else {
            Err("Window not found".to_string())
        }
    }

    /// Move window
    pub fn move_window(&mut self, id: WindowId, x: i32, y: i32) -> Result<(), String> {
        if let Some(window) = self.windows.get_mut(&id) {
            window.x = x;
            window.y = y;
            Ok(())
        } else {
            Err("Window not found".to_string())
        }
    }

    /// Resize window
    pub fn resize_window(&mut self, id: WindowId, width: u32, height: u32) -> Result<(), String> {
        if let Some(window) = self.windows.get_mut(&id) {
            window.width = width;
            window.height = height;
            Ok(())
        } else {
            Err("Window not found".to_string())
        }
    }

    /// Focus a window
    pub fn focus_window(&mut self, id: WindowId) -> Result<(), String> {
        if self.windows.contains_key(&id) {
            self.focused_window = Some(id);
            Ok(())
        } else {
            Err("Window not found".to_string())
        }
    }

    /// Get focused window
    pub fn get_focused_window(&self) -> Option<WindowId> {
        self.focused_window
    }

    /// List all windows
    pub fn list_windows(&self) -> Vec<&Window> {
        self.windows.values().collect()
    }

    /// Run the shell's main loop
    pub fn run(&mut self) {
        println!("hairr OS Desktop Shell v0.1.0");
        println!("Type 'help' for available commands\n");

        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }

            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            match self.handle_command(input) {
                Ok(should_exit) => {
                    if should_exit {
                        break;
                    }
                }
                Err(e) => println!("Error: {}", e),
            }
        }
    }

    fn handle_command(&mut self, input: &str) -> Result<bool, String> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(false);
        }

        match parts[0] {
            "help" => {
                self.show_help();
                Ok(false)
            }
            "exit" | "quit" => {
                println!("Shutting down hairr OS...");
                Ok(true)
            }
            "version" => {
                println!("hairr OS v0.1.0 - Desktop Shell");
                Ok(false)
            }
            "windows" => {
                self.list_all_windows();
                Ok(false)
            }
            "create" => {
                if parts.len() < 2 {
                    println!("Usage: create <window_title>");
                } else {
                    let title = parts[1..].join(" ");
                    let window_id = self.create_window(title.clone(), 0);
                    println!("Created window '{}' with ID {:?}", title, window_id);
                }
                Ok(false)
            }
            "close" => {
                if parts.len() < 2 {
                    println!("Usage: close <window_id>");
                } else {
                    if let Ok(id) = parts[1].parse::<u64>() {
                        match self.close_window(WindowId(id)) {
                            Ok(_) => println!("Window closed"),
                            Err(e) => println!("Error: {}", e),
                        }
                    } else {
                        println!("Invalid window ID");
                    }
                }
                Ok(false)
            }
            "focus" => {
                if parts.len() < 2 {
                    println!("Usage: focus <window_id>");
                } else {
                    if let Ok(id) = parts[1].parse::<u64>() {
                        match self.focus_window(WindowId(id)) {
                            Ok(_) => println!("Window focused"),
                            Err(e) => println!("Error: {}", e),
                        }
                    } else {
                        println!("Invalid window ID");
                    }
                }
                Ok(false)
            }
            _ => {
                println!("Unknown command: {}", parts[0]);
                println!("Type 'help' for available commands");
                Ok(false)
            }
        }
    }

    fn show_help(&self) {
        println!("Available commands:");
        println!("  help                    - Show this help message");
        println!("  version                 - Show version information");
        println!("  windows                 - List all windows");
        println!("  create <title>          - Create a new window");
        println!("  close <window_id>       - Close a window");
        println!("  focus <window_id>       - Focus a window");
        println!("  exit/quit               - Exit the shell");
    }

    fn list_all_windows(&self) {
        if self.windows.is_empty() {
            println!("No windows open");
            return;
        }

        println!("\nOpen Windows:");
        println!("{:<10} {:<30} {:<12} {:<20}", "ID", "Title", "State", "Position/Size");
        println!("{:-<80}", "");
        
        for window in self.windows.values() {
            let focused = if Some(window.id) == self.focused_window { "*" } else { " " };
            println!(
                "{}{:<9} {:<30} {:<12} {}x{} at ({}, {})",
                focused,
                format!("{:?}", window.id.0),
                window.title,
                format!("{:?}", window.state),
                window.width,
                window.height,
                window.x,
                window.y
            );
        }
        println!();
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    let mut shell = Shell::new();
    shell.run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_creation() {
        let mut shell = Shell::new();
        let window_id = shell.create_window("Test Window".to_string(), 1);
        assert!(shell.get_window(window_id).is_some());
    }

    #[test]
    fn test_window_focus() {
        let mut shell = Shell::new();
        let window_id = shell.create_window("Test Window".to_string(), 1);
        assert_eq!(shell.get_focused_window(), Some(window_id));
    }

    #[test]
    fn test_window_state_change() {
        let mut shell = Shell::new();
        let window_id = shell.create_window("Test Window".to_string(), 1);
        
        assert!(shell.set_window_state(window_id, WindowState::Maximized).is_ok());
        let window = shell.get_window(window_id).unwrap();
        assert_eq!(window.state, WindowState::Maximized);
    }

    #[test]
    fn test_window_close() {
        let mut shell = Shell::new();
        let window_id = shell.create_window("Test Window".to_string(), 1);
        
        assert!(shell.close_window(window_id).is_ok());
        assert!(shell.get_window(window_id).is_none());
    }
}
