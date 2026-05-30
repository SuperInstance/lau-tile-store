/// Store configuration.
#[derive(Debug, Clone)]
pub struct StoreConfig {
    pub wal_mode: bool,
    pub busy_timeout_ms: u32,
    pub journal_size_limit: Option<u64>,
    pub cache_size: Option<i64>,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            wal_mode: true,
            busy_timeout_ms: 5000,
            journal_size_limit: None,
            cache_size: None,
        }
    }
}

impl StoreConfig {
    pub fn high_performance() -> Self {
        Self {
            wal_mode: true,
            busy_timeout_ms: 10000,
            journal_size_limit: Some(100 * 1024 * 1024), // 100MB
            cache_size: Some(-8192),                      // 8GB page cache
        }
    }

    pub fn minimal() -> Self {
        Self {
            wal_mode: false,
            busy_timeout_ms: 1000,
            journal_size_limit: Some(5 * 1024 * 1024), // 5MB
            cache_size: None,
        }
    }
}
