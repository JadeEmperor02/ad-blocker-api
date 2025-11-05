//! Ad Blocker API Library
//! 
//! A comprehensive ad-blocking library that can be integrated into applications
//! to block advertisements, tracking scripts, and malicious content.
//! 
//! # Quick Start
//! 
//! ```rust
//! use ad_blocker_api::SimpleAdBlocker;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let blocker = SimpleAdBlocker::new().await?;
//!     
//!     if blocker.is_blocked("https://googleads.g.doubleclick.net/ads").await {
//!         println!("Ad blocked!");
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod blocker;
pub mod config;
pub mod filters;
pub mod types;
pub mod stevenblack;

pub use blocker::{AdBlockerAPI, SimpleAdBlocker};
pub use config::AdBlockerConfig;
pub use types::{BlockResult, BlockCategory};
pub use stevenblack::StevenBlackBlocker;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{AdBlockerAPI, SimpleAdBlocker, AdBlockerConfig, BlockResult, BlockCategory, StevenBlackBlocker};
}