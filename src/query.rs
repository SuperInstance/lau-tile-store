use crate::types::{TileStatus, TileType};

/// Sort order for query results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QueryOrder {
    #[default]
    NewestFirst,
    OldestFirst,
    RecentUpdate,
}

#[derive(Default)]
pub struct TileQuery {
    pub room_id: Option<String>,
    pub tile_type: Option<TileType>,
    pub status: Option<TileStatus>,
    pub ensign_id: Option<String>,
    pub parent_id: Option<String>,
    pub model_used: Option<String>,
    pub since: Option<i64>,
    pub until: Option<i64>,
    pub content_contains: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub order: QueryOrder,
}

impl TileQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn in_room(mut self, room: &str) -> Self {
        self.room_id = Some(room.to_string());
        self
    }

    pub fn of_type(mut self, t: TileType) -> Self {
        self.tile_type = Some(t);
        self
    }

    pub fn with_status(mut self, s: TileStatus) -> Self {
        self.status = Some(s);
        self
    }

    pub fn since(mut self, ts: i64) -> Self {
        self.since = Some(ts);
        self
    }

    pub fn until(mut self, ts: i64) -> Self {
        self.until = Some(ts);
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn offset(mut self, n: usize) -> Self {
        self.offset = Some(n);
        self
    }

    pub fn containing(mut self, text: &str) -> Self {
        self.content_contains = Some(text.to_string());
        self
    }

    pub fn with_parent(mut self, pid: &str) -> Self {
        self.parent_id = Some(pid.to_string());
        self
    }

    pub fn ensign_id(mut self, eid: &str) -> Self {
        self.ensign_id = Some(eid.to_string());
        self
    }

    pub fn model(mut self, m: &str) -> Self {
        self.model_used = Some(m.to_string());
        self
    }

    pub fn newest_first(mut self) -> Self {
        self.order = QueryOrder::NewestFirst;
        self
    }

    pub fn oldest_first(mut self) -> Self {
        self.order = QueryOrder::OldestFirst;
        self
    }

    pub fn recent_update(mut self) -> Self {
        self.order = QueryOrder::RecentUpdate;
        self
    }

    /// Build SQL WHERE clause fragments and params.
    pub(crate) fn to_sql(&self) -> (String, Vec<Box<dyn rusqlite::types::ToSql>>) {
        let mut clauses: Vec<String> = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref v) = self.room_id {
            clauses.push("room_id = ?".to_string());
            params.push(Box::new(v.clone()));
        }
        if let Some(ref v) = self.tile_type {
            clauses.push("tile_type = ?".to_string());
            params.push(Box::new(v.as_str().to_string()));
        }
        if let Some(ref v) = self.status {
            clauses.push("status = ?".to_string());
            params.push(Box::new(v.as_str().to_string()));
        }
        if let Some(ref v) = self.ensign_id {
            clauses.push("ensign_id = ?".to_string());
            params.push(Box::new(v.clone()));
        }
        if let Some(ref v) = self.parent_id {
            clauses.push("parent_id = ?".to_string());
            params.push(Box::new(v.clone()));
        }
        if let Some(ref v) = self.model_used {
            clauses.push("model_used = ?".to_string());
            params.push(Box::new(v.clone()));
        }
        if let Some(v) = self.since {
            clauses.push("created_at >= ?".to_string());
            params.push(Box::new(v));
        }
        if let Some(v) = self.until {
            clauses.push("created_at <= ?".to_string());
            params.push(Box::new(v));
        }
        if let Some(ref v) = self.content_contains {
            clauses.push("content LIKE ?".to_string());
            params.push(Box::new(format!("%{v}%")));
        }

        let where_clause = if clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", clauses.join(" AND "))
        };

        (where_clause, params)
    }

    /// Build ORDER BY clause.
    pub(crate) fn order_sql(&self) -> &'static str {
        match self.order {
            QueryOrder::NewestFirst => "ORDER BY created_at DESC",
            QueryOrder::OldestFirst => "ORDER BY created_at ASC",
            QueryOrder::RecentUpdate => "ORDER BY updated_at DESC",
        }
    }
}
