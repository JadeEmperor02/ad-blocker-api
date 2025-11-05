use serde::{Deserialize, Serialize};

/// Result of checking if a URL should be blocked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResult {
    pub should_block: bool,
    pub reason: String,
    pub filter_matched: Option<String>,
    pub category: BlockCategory,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BlockCategory {
    Advertisement,
    Tracking,
    Malware,
    Social,
    Custom,
    Whitelisted,
    Clean,
}

/// Statistics about blocked content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStats {
    pub total_requests: u64,
    pub blocked_requests: u64,
    pub ads_blocked: u64,
    pub trackers_blocked: u64,
    pub malware_blocked: u64,
    pub bytes_saved: u64,
}

impl Default for BlockStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            blocked_requests: 0,
            ads_blocked: 0,
            trackers_blocked: 0,
            malware_blocked: 0,
            bytes_saved: 0,
        }
    }
}

impl BlockStats {
    pub fn block_percentage(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.blocked_requests as f64 / self.total_requests as f64) * 100.0
        }
    }
}