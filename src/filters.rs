use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Filter list sources
pub struct FilterSources;

impl FilterSources {
    pub const EASYLIST: &'static str = "https://easylist.to/easylist/easylist.txt";
    pub const EASYPRIVACY: &'static str = "https://easylist.to/easylist/easyprivacy.txt";
    pub const MALWARE_DOMAINS: &'static str = "https://malware-filter.gitlab.io/malware-filter/urlhaus-filter-online.txt";
    pub const SOCIAL_ANNOYANCES: &'static str = "https://easylist.to/easylist/fanboy-social.txt";
}

/// Built-in tracking patterns
pub struct TrackingPatterns;

impl TrackingPatterns {
    pub fn get_patterns() -> Result<Vec<Regex>> {
        let patterns = vec![
            // Google Analytics & Ads
            r"google-analytics\.com",
            r"googletagmanager\.com",
            r"googlesyndication\.com",
            r"doubleclick\.net",
            r"googleadservices\.com",
            
            // Facebook
            r"facebook\.com/tr",
            r"connect\.facebook\.net",
            
            // Amazon
            r"amazon-adsystem\.com",
            r"adsystem\.amazon",
            
            // Other major trackers
            r"scorecardresearch\.com",
            r"quantserve\.com",
            r"outbrain\.com",
            r"taboola\.com",
            r"adsystem\.com",
            r"ads\.yahoo\.com",
            r"advertising\.com",
            
            // Analytics
            r"hotjar\.com",
            r"mixpanel\.com",
            r"segment\.com",
            r"amplitude\.com",
        ];
        
        patterns.into_iter()
            .map(|p| Regex::new(p))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow::anyhow!("Failed to compile regex: {}", e))
    }
}

/// Social media patterns
pub struct SocialPatterns;

impl SocialPatterns {
    pub fn get_patterns() -> Result<Vec<Regex>> {
        let patterns = vec![
            r"facebook\.com/plugins",
            r"twitter\.com/widgets",
            r"linkedin\.com/widgets",
            r"instagram\.com/embed",
            r"youtube\.com/embed",
            r"tiktok\.com/embed",
            r"addthis\.com",
            r"sharethis\.com",
        ];
        
        patterns.into_iter()
            .map(|p| Regex::new(p))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow::anyhow!("Failed to compile regex: {}", e))
    }
}

/// Filter list manager
pub struct FilterManager {
    cached_filters: HashMap<String, Vec<String>>,
}

impl FilterManager {
    pub fn new() -> Self {
        Self {
            cached_filters: HashMap::new(),
        }
    }
    
    /// Load filters from URL with caching
    pub async fn load_filters(&mut self, url: &str, use_cache: bool) -> Result<Vec<String>> {
        if use_cache && self.cached_filters.contains_key(url) {
            return Ok(self.cached_filters[url].clone());
        }
        
        let response = reqwest::get(url).await?;
        let content = response.text().await?;
        
        let filters: Vec<String> = content
            .lines()
            .filter(|line| !line.starts_with('!') && !line.trim().is_empty())
            .map(|line| line.to_string())
            .collect();
        
        if use_cache {
            self.cached_filters.insert(url.to_string(), filters.clone());
        }
        
        Ok(filters)
    }
    
    /// Clear filter cache
    pub fn clear_cache(&mut self) {
        self.cached_filters.clear();
    }
}