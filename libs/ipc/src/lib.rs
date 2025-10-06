//! Inter-Process Communication (IPC) for hairr OS
//! 
//! Provides high-performance, capability-aware IPC mechanisms for communication
//! between userspace processes and services.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Unique identifier for IPC channels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelId(u64);

impl ChannelId {
    pub fn new(id: u64) -> Self {
        ChannelId(id)
    }
}

/// Message types that can be sent through IPC
#[derive(Debug, Clone)]
pub enum Message {
    Text(String),
    Binary(Vec<u8>),
    Request { id: u64, data: Vec<u8> },
    Response { id: u64, data: Vec<u8> },
    Error { code: u32, message: String },
}

/// Represents an IPC channel endpoint
#[derive(Debug)]
pub struct Channel {
    id: ChannelId,
    messages: Arc<Mutex<Vec<Message>>>,
}

impl Channel {
    pub fn new(id: ChannelId) -> Self {
        Channel {
            id,
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn id(&self) -> ChannelId {
        self.id
    }

    /// Send a message through this channel
    pub fn send(&self, message: Message) -> Result<(), String> {
        self.messages.lock().unwrap().push(message);
        Ok(())
    }

    /// Receive the next message from this channel
    pub fn receive(&self) -> Option<Message> {
        self.messages.lock().unwrap().pop()
    }

    /// Check if there are pending messages
    pub fn has_messages(&self) -> bool {
        !self.messages.lock().unwrap().is_empty()
    }
}

/// The IPC manager handles channel creation and routing
pub struct IPCManager {
    channels: Arc<Mutex<HashMap<ChannelId, Channel>>>,
    next_channel_id: Arc<Mutex<u64>>,
}

impl IPCManager {
    pub fn new() -> Self {
        IPCManager {
            channels: Arc::new(Mutex::new(HashMap::new())),
            next_channel_id: Arc::new(Mutex::new(1)),
        }
    }

    /// Create a new IPC channel
    pub fn create_channel(&self) -> ChannelId {
        let mut next_id = self.next_channel_id.lock().unwrap();
        let channel_id = ChannelId(*next_id);
        *next_id += 1;

        let channel = Channel::new(channel_id);
        self.channels.lock().unwrap().insert(channel_id, channel);
        channel_id
    }

    /// Get a reference to a channel
    pub fn get_channel(&self, id: ChannelId) -> Option<Channel> {
        self.channels.lock().unwrap().get(&id).map(|c| Channel {
            id: c.id,
            messages: Arc::clone(&c.messages),
        })
    }

    /// Close a channel
    pub fn close_channel(&self, id: ChannelId) -> bool {
        self.channels.lock().unwrap().remove(&id).is_some()
    }

    /// Send a message to a specific channel
    pub fn send_message(&self, channel_id: ChannelId, message: Message) -> Result<(), String> {
        if let Some(channel) = self.get_channel(channel_id) {
            channel.send(message)
        } else {
            Err("Channel not found".to_string())
        }
    }

    /// Receive a message from a specific channel
    pub fn receive_message(&self, channel_id: ChannelId) -> Result<Option<Message>, String> {
        if let Some(channel) = self.get_channel(channel_id) {
            Ok(channel.receive())
        } else {
            Err("Channel not found".to_string())
        }
    }
}

impl Default for IPCManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_creation() {
        let manager = IPCManager::new();
        let channel_id = manager.create_channel();
        assert!(manager.get_channel(channel_id).is_some());
    }

    #[test]
    fn test_message_send_receive() {
        let manager = IPCManager::new();
        let channel_id = manager.create_channel();
        
        let msg = Message::Text("Hello, hairr OS!".to_string());
        assert!(manager.send_message(channel_id, msg).is_ok());
        
        let received = manager.receive_message(channel_id).unwrap();
        assert!(received.is_some());
    }

    #[test]
    fn test_channel_close() {
        let manager = IPCManager::new();
        let channel_id = manager.create_channel();
        
        assert!(manager.close_channel(channel_id));
        assert!(manager.get_channel(channel_id).is_none());
    }
}
