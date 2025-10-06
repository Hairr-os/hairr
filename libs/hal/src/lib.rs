//! Hardware Abstraction Layer (HAL) for hairr OS
//! 
//! Provides protocol-centric trait definitions for hardware interaction,
//! allowing hardware vendors to implement drivers independently.

/// CPU architecture types supported by hairr OS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuArch {
    X86_64,
    AArch64,
    RiscV64,
}

/// Device types that can be managed by the HAL
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceType {
    Display,
    Input,
    Network,
    Storage,
    Audio,
    GPU,
    Sensor,
    Custom(String),
}

/// Hardware device information
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub device_type: DeviceType,
    pub vendor: String,
    pub model: String,
    pub version: String,
}

/// Trait for hardware device drivers
pub trait Device: Send + Sync {
    /// Get device information
    fn info(&self) -> DeviceInfo;
    
    /// Initialize the device
    fn init(&mut self) -> Result<(), String>;
    
    /// Shutdown the device
    fn shutdown(&mut self) -> Result<(), String>;
    
    /// Read from the device
    fn read(&self, offset: usize, buffer: &mut [u8]) -> Result<usize, String>;
    
    /// Write to the device
    fn write(&mut self, offset: usize, data: &[u8]) -> Result<usize, String>;
}

/// Trait for display devices
pub trait DisplayDevice: Device {
    /// Get display resolution
    fn resolution(&self) -> (u32, u32);
    
    /// Set display resolution
    fn set_resolution(&mut self, width: u32, height: u32) -> Result<(), String>;
    
    /// Update the display framebuffer
    fn update_framebuffer(&mut self, buffer: &[u8]) -> Result<(), String>;
}

/// Trait for input devices
pub trait InputDevice: Device {
    /// Poll for input events
    fn poll_events(&self) -> Vec<InputEvent>;
}

/// Input event types
#[derive(Debug, Clone)]
pub enum InputEvent {
    KeyPress(u32),
    KeyRelease(u32),
    MouseMove { x: i32, y: i32 },
    MouseButton { button: u8, pressed: bool },
    TouchEvent { x: i32, y: i32, pressure: f32 },
    GestureEvent { gesture_type: String },
    VoiceCommand(String),
    EyeTracking { x: i32, y: i32 },
}

/// Trait for network devices
pub trait NetworkDevice: Device {
    /// Get MAC address
    fn mac_address(&self) -> [u8; 6];
    
    /// Send a network packet
    fn send_packet(&mut self, packet: &[u8]) -> Result<(), String>;
    
    /// Receive a network packet
    fn receive_packet(&self) -> Option<Vec<u8>>;
}

/// Trait for storage devices
pub trait StorageDevice: Device {
    /// Get storage capacity in bytes
    fn capacity(&self) -> u64;
    
    /// Read a block from storage
    fn read_block(&self, block: u64, buffer: &mut [u8]) -> Result<(), String>;
    
    /// Write a block to storage
    fn write_block(&mut self, block: u64, data: &[u8]) -> Result<(), String>;
}

/// Reference implementation of a basic device
pub struct ReferenceDevice {
    info: DeviceInfo,
    initialized: bool,
}

impl ReferenceDevice {
    pub fn new(device_type: DeviceType) -> Self {
        ReferenceDevice {
            info: DeviceInfo {
                device_type,
                vendor: "hairr OS".to_string(),
                model: "Reference Implementation".to_string(),
                version: "0.1.0".to_string(),
            },
            initialized: false,
        }
    }
}

impl Device for ReferenceDevice {
    fn info(&self) -> DeviceInfo {
        self.info.clone()
    }

    fn init(&mut self) -> Result<(), String> {
        self.initialized = true;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), String> {
        self.initialized = false;
        Ok(())
    }

    fn read(&self, _offset: usize, buffer: &mut [u8]) -> Result<usize, String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        Ok(buffer.len())
    }

    fn write(&mut self, _offset: usize, data: &[u8]) -> Result<usize, String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        Ok(data.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_device() {
        let mut device = ReferenceDevice::new(DeviceType::Display);
        assert!(device.init().is_ok());
        
        let mut buffer = [0u8; 10];
        assert!(device.read(0, &mut buffer).is_ok());
        assert!(device.write(0, &[1, 2, 3]).is_ok());
        
        assert!(device.shutdown().is_ok());
    }
}
