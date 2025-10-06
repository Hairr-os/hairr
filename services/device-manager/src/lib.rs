//! Device Manager Service
//! 
//! Manages hardware devices and driver registration in hairr OS.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Device identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceId(u64);

impl DeviceId {
    pub fn new(id: u64) -> Self {
        DeviceId(id)
    }
}

/// Device status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceStatus {
    Uninitialized,
    Ready,
    Active,
    Error,
    Offline,
}

/// Managed device information
#[derive(Debug, Clone)]
pub struct ManagedDevice {
    pub id: DeviceId,
    pub name: String,
    pub device_type: String,
    pub status: DeviceStatus,
    pub driver_name: String,
}

impl ManagedDevice {
    pub fn new(id: DeviceId, name: String, device_type: String, driver_name: String) -> Self {
        ManagedDevice {
            id,
            name,
            device_type,
            status: DeviceStatus::Uninitialized,
            driver_name,
        }
    }
}

/// Device Manager handles device registration and lifecycle
pub struct DeviceManager {
    devices: Arc<Mutex<HashMap<DeviceId, ManagedDevice>>>,
    next_device_id: Arc<Mutex<u64>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        DeviceManager {
            devices: Arc::new(Mutex::new(HashMap::new())),
            next_device_id: Arc::new(Mutex::new(1)),
        }
    }

    /// Register a new device
    pub fn register_device(
        &self,
        name: String,
        device_type: String,
        driver_name: String,
    ) -> DeviceId {
        let mut next_id = self.next_device_id.lock().unwrap();
        let device_id = DeviceId(*next_id);
        *next_id += 1;

        let device = ManagedDevice::new(device_id, name, device_type, driver_name);
        self.devices.lock().unwrap().insert(device_id, device);
        
        device_id
    }

    /// Unregister a device
    pub fn unregister_device(&self, id: DeviceId) -> Result<(), String> {
        if self.devices.lock().unwrap().remove(&id).is_some() {
            Ok(())
        } else {
            Err("Device not found".to_string())
        }
    }

    /// Get device information
    pub fn get_device(&self, id: DeviceId) -> Option<ManagedDevice> {
        self.devices.lock().unwrap().get(&id).cloned()
    }

    /// Update device status
    pub fn update_status(&self, id: DeviceId, status: DeviceStatus) -> Result<(), String> {
        let mut devices = self.devices.lock().unwrap();
        if let Some(device) = devices.get_mut(&id) {
            device.status = status;
            Ok(())
        } else {
            Err("Device not found".to_string())
        }
    }

    /// List all devices
    pub fn list_devices(&self) -> Vec<ManagedDevice> {
        self.devices.lock().unwrap().values().cloned().collect()
    }

    /// Find devices by type
    pub fn find_by_type(&self, device_type: &str) -> Vec<ManagedDevice> {
        self.devices
            .lock()
            .unwrap()
            .values()
            .filter(|d| d.device_type == device_type)
            .cloned()
            .collect()
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_registration() {
        let manager = DeviceManager::new();
        let device_id = manager.register_device(
            "Display0".to_string(),
            "display".to_string(),
            "reference_driver".to_string(),
        );
        
        let device = manager.get_device(device_id);
        assert!(device.is_some());
        assert_eq!(device.unwrap().name, "Display0");
    }

    #[test]
    fn test_device_status_update() {
        let manager = DeviceManager::new();
        let device_id = manager.register_device(
            "Display0".to_string(),
            "display".to_string(),
            "reference_driver".to_string(),
        );
        
        assert!(manager.update_status(device_id, DeviceStatus::Ready).is_ok());
        let device = manager.get_device(device_id).unwrap();
        assert_eq!(device.status, DeviceStatus::Ready);
    }

    #[test]
    fn test_find_by_type() {
        let manager = DeviceManager::new();
        manager.register_device("Display0".to_string(), "display".to_string(), "driver1".to_string());
        manager.register_device("Display1".to_string(), "display".to_string(), "driver1".to_string());
        manager.register_device("Keyboard0".to_string(), "input".to_string(), "driver2".to_string());
        
        let displays = manager.find_by_type("display");
        assert_eq!(displays.len(), 2);
    }
}
