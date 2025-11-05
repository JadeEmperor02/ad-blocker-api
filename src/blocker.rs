use crate::config::AdBlockerConfig;
use crate::filters::{FilterManager, FilterSources, SocialPatterns, TrackingPatterns};
use crate::types::{BlockCategory, BlockResult, BlockStats};

use adblock::{Engine, FilterSet, request::Request};
use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

/// Main ad blocker API
pub struct AdBlockerAPI {
    engine: Arc<RwLock<Engine>>,
    config: AdBlockerConfig,
    whitelist_domains: HashSet<String>,
    tracking_patterns: Vec<Regex>,
    social_patterns: Vec<Regex>,
    stats: Arc<RwLock<BlockStats>>,
    _filter_manager: FilterManager,
}

impl AdBlockerAPI {
    /// Create a new ad blocker instance
    pub async fn new(config: AdBlockerConfig) -> Result<Self> {
        let mut filter_manager = FilterManager::new();
        let mut filter_set = FilterSet::new(true);
        
        // Load EasyList filters
        if config.enable_easylist {
            let easylist_rules = filter_manager
                .load_filters(FilterSources::EASYLIST, config.cache_filters)
                .await?;
            filter_set.add_filters(&easylist_rules, Default::default());
        }
        
        // Load EasyPrivacy filters
        if config.enable_easyprivacy {
            let easyprivacy_rules = filter_manager
                .load_filters(FilterSources::EASYPRIVACY, config.cache_filters)
                .await?;
            filter_set.add_filters(&easyprivacy_rules, Default::default());
        }
        
        // Load malware protection filters (optional, may fail due to network)
        if config.enable_malware_protection {
            if let Ok(malware_rules) = filter_manager
                .load_filters(FilterSources::MALWARE_DOMAINS, config.cache_filters)
                .await
            {
                filter_set.add_filters(&malware_rules, Default::default());
            } else {
                eprintln!("Warning: Could not load malware protection filters");
            }
        }
        
        // Load social annoyances filters
        if config.block_social {
            let social_rules = filter_manager
                .load_filters(FilterSources::SOCIAL_ANNOYANCES, config.cache_filters)
                .await?;
            filter_set.add_filters(&social_rules, Default::default());
        }
        
        // Add custom filters
        if !config.custom_filters.is_empty() {
            filter_set.add_filters(&config.custom_filters, Default::default());
        }
        
        let engine = Engine::from_filter_set(filter_set, true);
        
        // Compile patterns
        let tracking_patterns = if config.block_tracking {
            TrackingPatterns::get_patterns()?
        } else {
            vec![]
        };
        
        let social_patterns = if config.block_social {
            SocialPatterns::get_patterns()?
        } else {
            vec![]
        };
        
        let whitelist_domains: HashSet<String> = config.whitelist_domains.iter().cloned().collect();
        
        Ok(Self {
            engine: Arc::new(RwLock::new(engine)),
            config,
            whitelist_domains,
            tracking_patterns,
            social_patterns,
            stats: Arc::new(RwLock::new(BlockStats::default())),
            _filter_manager: filter_manager,
        })
    }
    
    /// Check if a URL should be blocked
    pub async fn should_block(&self, url: &str, source_url: Option<&str>) -> Result<BlockResult> {
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
        }
        
        // Parse URL
        let parsed_url = match Url::parse(url) {
            Ok(url) => url,
            Err(_) => {
                return Ok(BlockResult {
                    should_block: false,
                    reason: "Invalid URL format".to_string(),
                    filter_matched: None,
                    category: BlockCategory::Clean,
                });
            }
        };
        
        // Check whitelist first
        if let Some(domain) = parsed_url.domain() {
            if self.whitelist_domains.contains(domain) {
                return Ok(BlockResult {
                    should_block: false,
                    reason: "Domain is whitelisted".to_string(),
                    filter_matched: None,
                    category: BlockCategory::Whitelisted,
                });
            }
        }
        
        // Check against adblock engine
        let engine = self.engine.read().await;
        let request = Request::new(
            url,
            source_url.unwrap_or(""),
            "other"
        )?;
        let blocker_result = engine.check_network_request(&request);
        
        if blocker_result.matched {
            self.update_block_stats(BlockCategory::Advertisement).await;
            return Ok(BlockResult {
                should_block: true,
                reason: "Matched ad filter".to_string(),
                filter_matched: blocker_result.filter.map(|f| f.to_string()),
                category: BlockCategory::Advertisement,
            });
        }
        
        // Check tracking patterns
        if self.config.block_tracking {
            for pattern in &self.tracking_patterns {
                if pattern.is_match(url) {
                    self.update_block_stats(BlockCategory::Tracking).await;
                    return Ok(BlockResult {
                        should_block: true,
                        reason: "Matched tracking pattern".to_string(),
                        filter_matched: Some(pattern.as_str().to_string()),
                        category: BlockCategory::Tracking,
                    });
                }
            }
        }
        
        // Check social patterns
        if self.config.block_social {
            for pattern in &self.social_patterns {
                if pattern.is_match(url) {
                    self.update_block_stats(BlockCategory::Social).await;
                    return Ok(BlockResult {
                        should_block: true,
                        reason: "Matched social media pattern".to_string(),
                        filter_matched: Some(pattern.as_str().to_string()),
                        category: BlockCategory::Social,
                    });
                }
            }
        }
        
        Ok(BlockResult {
            should_block: false,
            reason: "URL is clean".to_string(),
            filter_matched: None,
            category: BlockCategory::Clean,
        })
    }
    
    /// Batch check multiple URLs
    pub async fn batch_check(&self, urls: Vec<String>, source_url: Option<&str>) -> Result<Vec<(String, BlockResult)>> {
        let mut results = Vec::new();
        
        for url in urls {
            let result = self.should_block(&url, source_url).await?;
            results.push((url, result));
        }
        
        Ok(results)
    }
    
    /// Add custom filter rule
    pub async fn add_custom_filter(&mut self, filter: String) -> Result<()> {
        let mut engine = self.engine.write().await;
        let mut filter_set = FilterSet::new(true);
        filter_set.add_filters(&[filter.clone()], Default::default());
        *engine = Engine::from_filter_set(filter_set, true);
        
        self.config.custom_filters.push(filter);
        Ok(())
    }
    
    /// Add domain to whitelist
    pub fn add_whitelist_domain(&mut self, domain: String) {
        self.whitelist_domains.insert(domain.clone());
        self.config.whitelist_domains.push(domain);
    }
    
    /// Remove domain from whitelist
    pub fn remove_whitelist_domain(&mut self, domain: &str) {
        self.whitelist_domains.remove(domain);
        self.config.whitelist_domains.retain(|d| d != domain);
    }
    
    /// Get current configuration
    pub fn get_config(&self) -> &AdBlockerConfig {
        &self.config
    }
    
    /// Get blocking statistics
    pub async fn get_stats(&self) -> BlockStats {
        self.stats.read().await.clone()
    }
    
    /// Reset statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = BlockStats::default();
    }
    
    /// Update configuration
    pub async fn update_config(&mut self, new_config: AdBlockerConfig) -> Result<()> {
        *self = Self::new(new_config).await?;
        Ok(())
    }
    
    async fn update_block_stats(&self, category: BlockCategory) {
        let mut stats = self.stats.write().await;
        stats.blocked_requests += 1;
        
        match category {
            BlockCategory::Advertisement => stats.ads_blocked += 1,
            BlockCategory::Tracking => stats.trackers_blocked += 1,
            BlockCategory::Malware => stats.malware_blocked += 1,
            _ => {}
        }
    }
}

/// Simple API wrapper for easy integration
pub struct SimpleAdBlocker {
    blocker: AdBlockerAPI,
}

impl SimpleAdBlocker {
    /// Create a new simple ad blocker with default settings
    pub async fn new() -> Result<Self> {
        let config = AdBlockerConfig::default();
        let blocker = AdBlockerAPI::new(config).await?;
        
        Ok(Self { blocker })
    }
    
    /// Create with custom configuration
    pub async fn with_config(config: AdBlockerConfig) -> Result<Self> {
        let blocker = AdBlockerAPI::new(config).await?;
        Ok(Self { blocker })
    }
    
    /// Simple check if URL should be blocked
    pub async fn is_blocked(&self, url: &str) -> bool {
        match self.blocker.should_block(url, None).await {
            Ok(result) => result.should_block,
            Err(_) => false,
        }
    }
    
    /// Get detailed block information
    pub async fn check_url(&self, url: &str) -> Result<BlockResult> {
        self.blocker.should_block(url, None).await
    }
    
    /// Get blocking statistics
    pub async fn get_stats(&self) -> BlockStats {
        self.blocker.get_stats().await
    }
}