//! Capability-based security system for hairr OS
//! 
//! This module provides the foundation for capability-based access control,
//! ensuring that components can only access resources they have explicit
//! permission to use.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Represents a unique capability token
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CapabilityToken(u64);

impl CapabilityToken {
    pub fn new(id: u64) -> Self {
        CapabilityToken(id)
    }
}

/// Types of resources that can be protected by capabilities
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resource {
    File(String),
    Network(String),
    Device(String),
    IPC(String),
    Memory(usize),
}

/// Permission levels for capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Read,
    Write,
    Execute,
    ReadWrite,
    Full,
}

/// A capability grants specific permissions to a resource
#[derive(Debug, Clone)]
pub struct Capability {
    pub token: CapabilityToken,
    pub resource: Resource,
    pub permission: Permission,
}

/// The capability manager tracks and validates capabilities
pub struct CapabilityManager {
    capabilities: Arc<Mutex<HashMap<CapabilityToken, Capability>>>,
    next_token_id: Arc<Mutex<u64>>,
}

impl CapabilityManager {
    pub fn new() -> Self {
        CapabilityManager {
            capabilities: Arc::new(Mutex::new(HashMap::new())),
            next_token_id: Arc::new(Mutex::new(1)),
        }
    }

    /// Grant a new capability for a resource
    pub fn grant(&self, resource: Resource, permission: Permission) -> CapabilityToken {
        let mut next_id = self.next_token_id.lock().unwrap();
        let token = CapabilityToken(*next_id);
        *next_id += 1;

        let capability = Capability {
            token,
            resource,
            permission,
        };

        self.capabilities.lock().unwrap().insert(token, capability);
        token
    }

    /// Revoke a capability
    pub fn revoke(&self, token: CapabilityToken) -> bool {
        self.capabilities.lock().unwrap().remove(&token).is_some()
    }

    /// Check if a capability is valid
    pub fn validate(&self, token: CapabilityToken) -> Option<Capability> {
        self.capabilities.lock().unwrap().get(&token).cloned()
    }

    /// Check if a token has permission for a specific operation
    pub fn check_permission(&self, token: CapabilityToken, required: Permission) -> bool {
        if let Some(cap) = self.validate(token) {
            match (cap.permission, required) {
                (Permission::Full, _) => true,
                (Permission::ReadWrite, Permission::Read) => true,
                (Permission::ReadWrite, Permission::Write) => true,
                (p1, p2) => p1 == p2,
            }
        } else {
            false
        }
    }
}

impl Default for CapabilityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_grant_and_validate() {
        let manager = CapabilityManager::new();
        let token = manager.grant(Resource::File("/test.txt".to_string()), Permission::Read);
        
        let cap = manager.validate(token);
        assert!(cap.is_some());
        assert_eq!(cap.unwrap().permission, Permission::Read);
    }

    #[test]
    fn test_capability_revoke() {
        let manager = CapabilityManager::new();
        let token = manager.grant(Resource::File("/test.txt".to_string()), Permission::Read);
        
        assert!(manager.revoke(token));
        assert!(manager.validate(token).is_none());
    }

    #[test]
    fn test_permission_checking() {
        let manager = CapabilityManager::new();
        let token = manager.grant(Resource::File("/test.txt".to_string()), Permission::ReadWrite);
        
        assert!(manager.check_permission(token, Permission::Read));
        assert!(manager.check_permission(token, Permission::Write));
        assert!(!manager.check_permission(token, Permission::Execute));
    }
}
