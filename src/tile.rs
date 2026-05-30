use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::types::{TileStatus, TileType};

/// A tile as stored in SQLite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorableTile {
    pub id: String,
    pub room_id: Option<String>,
    pub tile_type: TileType,
    pub parent_id: Option<String>,
    pub status: TileStatus,
    pub content: String,
    pub content_type: Option<String>,
    pub deadband_lower: Option<f64>,
    pub deadband_upper: Option<f64>,
    pub deadband_current: Option<f64>,
    pub deadband_trend: Option<String>,
    pub ensign_id: Option<String>,
    pub model_used: Option<String>,
    pub tokens_used: u32,
    pub conservation_delta: f64,
    #[serde(
        serialize_with = "serialize_metadata",
        deserialize_with = "deserialize_metadata"
    )]
    pub metadata: HashMap<String, String>,
    pub created_at: i64,
    pub updated_at: i64,
}

fn serialize_metadata<S: serde::Serializer>(
    val: &HashMap<String, String>,
    s: S,
) -> Result<S::Ok, S::Error> {
    serde_json::to_string(val).map_err(serde::ser::Error::custom)?.serialize(s)
}

fn deserialize_metadata<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<HashMap<String, String>, D::Error> {
    let s = String::deserialize(d)?;
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

impl StorableTile {
    /// Create a new tile with an auto-generated ID.
    pub fn new(tile_type: TileType, content: &str) -> Self {
        let now = now_ts();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            room_id: None,
            tile_type,
            parent_id: None,
            status: TileStatus::Active,
            content: content.to_string(),
            content_type: None,
            deadband_lower: None,
            deadband_upper: None,
            deadband_current: None,
            deadband_trend: None,
            ensign_id: None,
            model_used: None,
            tokens_used: 0,
            conservation_delta: 0.0,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_room(mut self, room: &str) -> Self {
        self.room_id = Some(room.to_string());
        self
    }

    pub fn with_parent(mut self, parent: &str) -> Self {
        self.parent_id = Some(parent.to_string());
        self
    }

    pub fn with_metadata(mut self, key: &str, val: &str) -> Self {
        self.metadata.insert(key.to_string(), val.to_string());
        self
    }

    /// Create a child tile linked to this one as parent.
    pub fn child(&self, tile_type: TileType, content: &str) -> StorableTile {
        StorableTile::new(tile_type, content)
            .with_parent(&self.id)
            .with_room(self.room_id.as_deref().unwrap_or(""))
    }

    pub fn complete(&mut self) {
        self.status = TileStatus::Complete;
        self.updated_at = now_ts();
    }

    pub fn archive(&mut self) {
        self.status = TileStatus::Archived;
        self.updated_at = now_ts();
    }

    pub fn escalate(&mut self) {
        self.status = TileStatus::Escalated;
        self.updated_at = now_ts();
    }

    /// Age in seconds since creation.
    pub fn age_seconds(&self) -> i64 {
        now_ts() - self.created_at
    }

    /// Serialize metadata to JSON string for SQLite storage.
    pub fn metadata_json(&self) -> String {
        serde_json::to_string(&self.metadata).unwrap_or_else(|_| "{}".to_string())
    }

    /// Deserialize metadata from JSON string.
    pub fn metadata_from_json(json: &str) -> HashMap<String, String> {
        serde_json::from_str(json).unwrap_or_default()
    }
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
