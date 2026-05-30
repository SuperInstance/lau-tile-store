use std::collections::HashMap;
use std::fs;
use std::path::Path;

use rusqlite::{params, Connection};

use crate::config::StoreConfig;
use crate::error::StoreError;
use crate::query::TileQuery;
use crate::stats::StoreStats;
use crate::tile::StorableTile;
use crate::types::{TileStatus, TileType};

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS tiles (
    id TEXT PRIMARY KEY,
    room_id TEXT,
    tile_type TEXT NOT NULL,
    parent_id TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    content TEXT NOT NULL,
    content_type TEXT,
    deadband_lower REAL, deadband_upper REAL, deadband_current REAL,
    deadband_trend TEXT,
    ensign_id TEXT,
    model_used TEXT,
    tokens_used INTEGER NOT NULL DEFAULT 0,
    conservation_delta REAL NOT NULL DEFAULT 0.0,
    metadata TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_tiles_room ON tiles(room_id);
CREATE INDEX IF NOT EXISTS idx_tiles_type ON tiles(tile_type);
CREATE INDEX IF NOT EXISTS idx_tiles_status ON tiles(status);
CREATE INDEX IF NOT EXISTS idx_tiles_parent ON tiles(parent_id);
CREATE INDEX IF NOT EXISTS idx_tiles_ensign ON tiles(ensign_id);
CREATE INDEX IF NOT EXISTS idx_tiles_created ON tiles(created_at);
CREATE INDEX IF NOT EXISTS idx_tiles_model ON tiles(model_used);

CREATE TABLE IF NOT EXISTS tile_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

/// The main tile store interface.
pub struct TileStore {
    pub db: Connection,
    pub config: StoreConfig,
}

impl TileStore {
    /// Open or create a SQLite-backed tile store.
    pub fn open(path: &str) -> Result<Self, StoreError> {
        Self::open_with_config(path, StoreConfig::default())
    }

    /// Open with custom config.
    pub fn open_with_config(path: &str, config: StoreConfig) -> Result<Self, StoreError> {
        // Ensure parent dir exists
        if path != ":memory:" {
            if let Some(parent) = Path::new(path).parent() {
                fs::create_dir_all(parent).map_err(|e| StoreError::OpenFailed(e.to_string()))?;
            }
        }

        let db = Connection::open(path).map_err(|e| StoreError::OpenFailed(e.to_string()))?;

        // Apply config
        if config.wal_mode {
            db.execute_batch("PRAGMA journal_mode=WAL;")
                .map_err(|e| StoreError::OpenFailed(e.to_string()))?;
        }
        db.execute_batch(&format!(
            "PRAGMA busy_timeout={};",
            config.busy_timeout_ms
        ))
        .map_err(|e| StoreError::OpenFailed(e.to_string()))?;

        if let Some(limit) = config.journal_size_limit {
            db.execute_batch(&format!("PRAGMA journal_size_limit={limit};"))
                .map_err(|e| StoreError::OpenFailed(e.to_string()))?;
        }
        if let Some(cache) = config.cache_size {
            db.execute_batch(&format!("PRAGMA cache_size={cache};"))
                .map_err(|e| StoreError::OpenFailed(e.to_string()))?;
        }

        // Create schema
        db.execute_batch(SCHEMA)
            .map_err(|e| StoreError::OpenFailed(e.to_string()))?;

        Ok(Self { db, config })
    }

    /// Open an in-memory store (for testing).
    pub fn open_memory() -> Result<Self, StoreError> {
        Self::open_with_config(":memory:", StoreConfig::default())
    }

    /// Store (insert or update) a tile.
    pub fn store(&self, tile: &StorableTile) -> Result<(), StoreError> {
        self.db.execute(
            "INSERT OR REPLACE INTO tiles
             (id, room_id, tile_type, parent_id, status, content, content_type,
              deadband_lower, deadband_upper, deadband_current, deadband_trend,
              ensign_id, model_used, tokens_used, conservation_delta, metadata,
              created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
            params![
                tile.id,
                tile.room_id,
                tile.tile_type.as_str(),
                tile.parent_id,
                tile.status.as_str(),
                tile.content,
                tile.content_type,
                tile.deadband_lower,
                tile.deadband_upper,
                tile.deadband_current,
                tile.deadband_trend,
                tile.ensign_id,
                tile.model_used,
                tile.tokens_used,
                tile.conservation_delta,
                tile.metadata_json(),
                tile.created_at,
                tile.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Retrieve a tile by ID.
    pub fn get(&self, id: &str) -> Result<Option<StorableTile>, StoreError> {
        let mut stmt = self
            .db
            .prepare("SELECT * FROM tiles WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(row_to_tile(row)?)),
            None => Ok(None),
        }
    }

    /// Query tiles with a filter.
    pub fn query(&self, filter: TileQuery) -> Result<Vec<StorableTile>, StoreError> {
        let (where_clause, params) = filter.to_sql();
        let order = filter.order_sql();

        let limit_clause = match filter.limit {
            Some(n) => format!("LIMIT {n}"),
            None => String::new(),
        };
        let offset_clause = match filter.offset {
            Some(n) => format!("OFFSET {n}"),
            None => String::new(),
        };

        let sql = format!(
            "SELECT * FROM tiles {where_clause} {order} {limit_clause} {offset_clause}"
        );

        let mut stmt = self.db.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        let mut rows = stmt.query(param_refs.as_slice())?;

        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            result.push(row_to_tile(row)?);
        }
        Ok(result)
    }

    /// Delete a tile by ID.
    pub fn delete(&self, id: &str) -> Result<(), StoreError> {
        let changes = self.db.execute("DELETE FROM tiles WHERE id = ?1", params![id])?;
        if changes == 0 {
            return Err(StoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    /// Count tiles matching a filter.
    pub fn count(&self, filter: &TileQuery) -> Result<u64, StoreError> {
        let (where_clause, params) = filter.to_sql();
        let sql = format!("SELECT COUNT(*) FROM tiles {where_clause}");

        let mut stmt = self.db.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        let count: u64 = stmt.query_row(param_refs.as_slice(), |row| row.get(0))?;
        Ok(count)
    }

    /// Get direct children of a tile.
    pub fn children_of(&self, parent_id: &str) -> Result<Vec<StorableTile>, StoreError> {
        self.query(TileQuery::new().with_parent(parent_id).limit(1000))
    }

    /// Get tiles in a room.
    pub fn room_tiles(&self, room_id: &str, limit: usize) -> Result<Vec<StorableTile>, StoreError> {
        self.query(TileQuery::new().in_room(room_id).limit(limit))
    }

    /// Get most recent tiles across all rooms.
    pub fn recent(&self, limit: usize) -> Result<Vec<StorableTile>, StoreError> {
        self.query(TileQuery::new().limit(limit))
    }

    /// Walk up the parent chain to get all ancestors.
    pub fn ancestors(&self, tile_id: &str) -> Result<Vec<StorableTile>, StoreError> {
        let mut result = Vec::new();
        let mut current_id = tile_id.to_string();

        while let Some(tile) = self.get(&current_id)? {
            if let Some(ref pid) = tile.parent_id {
                current_id = pid.clone();
                result.push(tile);
            } else {
                break;
            }
        }

        Ok(result)
    }

    /// Walk down children recursively.
    pub fn descendants(&self, tile_id: &str) -> Result<Vec<StorableTile>, StoreError> {
        let mut result = Vec::new();
        let mut stack = vec![tile_id.to_string()];

        while let Some(pid) = stack.pop() {
            let children = self.children_of(&pid)?;
            for child in children {
                stack.push(child.id.clone());
                result.push(child);
            }
        }

        Ok(result)
    }

    /// Get database statistics.
    pub fn stats(&self) -> Result<StoreStats, StoreError> {
        let total_tiles: u64 = self
            .db
            .query_row("SELECT COUNT(*) FROM tiles", [], |row| row.get(0))?;

        let total_rooms: u64 = self
            .db
            .query_row(
                "SELECT COUNT(DISTINCT room_id) FROM tiles WHERE room_id IS NOT NULL",
                [],
                |row| row.get(0),
            )?;

        let mut by_type = HashMap::new();
        {
            let mut stmt = self
                .db
                .prepare("SELECT tile_type, COUNT(*) FROM tiles GROUP BY tile_type")?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let t: String = row.get(0)?;
                let c: u64 = row.get(1)?;
                by_type.insert(t, c);
            }
        }

        let mut by_status = HashMap::new();
        {
            let mut stmt = self
                .db
                .prepare("SELECT status, COUNT(*) FROM tiles GROUP BY status")?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let s: String = row.get(0)?;
                let c: u64 = row.get(1)?;
                by_status.insert(s, c);
            }
        }

        let oldest_tile: Option<i64> = self
            .db
            .query_row(
                "SELECT MIN(created_at) FROM tiles",
                [],
                |row| row.get(0),
            )
            .ok();
        let newest_tile: Option<i64> = self
            .db
            .query_row(
                "SELECT MAX(created_at) FROM tiles",
                [],
                |row| row.get(0),
            )
            .ok();

        // DB size - use pragma
        let db_size_bytes: u64 = self
            .db
            .query_row("SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size", [], |row| row.get::<_, i64>(0))
            .unwrap_or(0) as u64;

        Ok(StoreStats {
            total_tiles,
            total_rooms,
            by_type,
            by_status,
            db_size_bytes,
            oldest_tile,
            newest_tile,
        })
    }

    /// Reclaim space.
    pub fn vacuum(&self) -> Result<(), StoreError> {
        self.db.execute_batch("VACUUM")?;
        Ok(())
    }

    /// Export tiles matching filter as JSON.
    pub fn export_json(&self, filter: TileQuery) -> Result<String, StoreError> {
        let tiles = self.query(filter)?;
        Ok(serde_json::to_string_pretty(&tiles)?)
    }

    /// Import tiles from JSON.
    pub fn import_json(&self, json: &str) -> Result<u64, StoreError> {
        let tiles: Vec<StorableTile> = serde_json::from_str(json)?;
        let count = tiles.len() as u64;
        for tile in &tiles {
            self.store(tile)?;
        }
        Ok(count)
    }
}

fn row_to_tile(row: &rusqlite::Row<'_>) -> Result<StorableTile, StoreError> {
    let tile_type_str: String = row.get("tile_type")?;
    let status_str: String = row.get("status")?;
    let metadata_str: String = row.get("metadata")?;

    Ok(StorableTile {
        id: row.get("id")?,
        room_id: row.get("room_id")?,
        tile_type: TileType::parse(&tile_type_str)
            .ok_or_else(|| StoreError::InvalidData(format!("unknown tile type: {tile_type_str}")))?,
        parent_id: row.get("parent_id")?,
        status: TileStatus::parse(&status_str)
            .ok_or_else(|| StoreError::InvalidData(format!("unknown status: {status_str}")))?,
        content: row.get("content")?,
        content_type: row.get("content_type")?,
        deadband_lower: row.get("deadband_lower")?,
        deadband_upper: row.get("deadband_upper")?,
        deadband_current: row.get("deadband_current")?,
        deadband_trend: row.get("deadband_trend")?,
        ensign_id: row.get("ensign_id")?,
        model_used: row.get("model_used")?,
        tokens_used: row.get("tokens_used")?,
        conservation_delta: row.get("conservation_delta")?,
        metadata: StorableTile::metadata_from_json(&metadata_str),
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}
