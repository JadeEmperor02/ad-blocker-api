use serde::{Deserialize, Serialize};

/// Configuration for the ad blocker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdBlockerConfig {
    pub enable_easylist: bool,
    pub enable_easyprivacy: bool,
    pub enable_malware_protection: bool,
    pub custom_filters: Vec<String>,
    pub whitelist_domains: Vec<String>,
    pub block_tracking: bool,
    pub block_social: bool,
    pub aggressive_blocking: bool,
    pub cache_filters: bool,
}

impl Default for AdBlockerConfig {
    fn default() -> Self {
        Self {
            enable_easylist: true,
            enable_easyprivacy: true,
            enable_malware_protection: false, // Disabled by default due to potential network issues
            custom_filters: vec![],
            whitelist_domains: vec![],
            block_tracking: true,
            block_social: false,
            aggressive_blocking: false,
            cache_filters: true,
        }
    }
}

impl AdBlockerConfig {
    /// Create a minimal configuration for basic ad blocking
    pub fn minimal() -> Self {
        Self {
            enable_easylist: true,
            enable_easyprivacy: false,
            enable_malware_protection: false,
            custom_filters: vec![],
            whitelist_domains: vec![],
            block_tracking: false,
            block_social: false,
            aggressive_blocking: false,
            cache_filters: true,
        }
    }
    
    /// Create a privacy-focused configuration
    pub fn privacy_focused() -> Self {
        Self {
            enable_easylist: true,
            enable_easyprivacy: true,
            enable_malware_protection: true,
            custom_filters: vec![],
            whitelist_domains: vec![],
            block_tracking: true,
            block_social: true,
            aggressive_blocking: true,
            cache_filters: true,
        }
    }
    
    /// Create a performance-focused configuration (less blocking, faster)
    pub fn performance_focused() -> Self {
        Self {
            enable_easylist: true,
            enable_easyprivacy: false,
            enable_malware_protection: false,
            custom_filters: vec![],
            whitelist_domains: vec![],
            block_tracking: false,
            block_social: false,
            aggressive_blocking: false,
            cache_filters: true,
        }
    }
}