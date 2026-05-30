use std::collections::HashMap;

/// Database statistics.
#[derive(Debug, Clone)]
pub struct StoreStats {
    pub total_tiles: u64,
    pub total_rooms: u64,
    pub by_type: HashMap<String, u64>,
    pub by_status: HashMap<String, u64>,
    pub db_size_bytes: u64,
    pub oldest_tile: Option<i64>,
    pub newest_tile: Option<i64>,
}
