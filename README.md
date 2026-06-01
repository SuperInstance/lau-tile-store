# lau-tile-store

SQLite-backed tile persistence layer for agent systems.

A tile is a typed data unit with identity, hierarchy, metadata, and lifecycle. This crate stores them in SQLite with full CRUD, tree traversal, filtered queries, and JSON import/export. No ORM, no migrations, no ceremony. Open a database, store tiles, query them.

**~65 integration tests · rusqlite + serde**

---

## What This Does

`lau-tile-store` provides durable storage for "tiles" — typed data records used by agent systems. Each tile has:

- A UUID, a type (observation, action, thought, etc.), and content
- Optional room assignment and parent linkage (tree structure)
- Deadband fields for monitoring
- Arbitrary metadata (key-value, JSON-serialized)
- Lifecycle status (active → complete → archived)

The store is SQLite-backed with WAL mode, indexed for fast queries by room, type, status, parent, ensign, and creation time.

---

## Key Idea

**Tiles are memory. This crate makes that memory durable.**

Tiles form trees (parent → children). Queries compose with a builder pattern (`TileQuery::new().in_room("bridge").of_type(TileType::Action).limit(50)`). The store supports ancestor/descendant traversal, export/import for backup, and stats aggregation. It's the persistence layer that survives restarts.

---

## Install

```toml
[dependencies]
lau-tile-store = { git = "https://github.com/SuperInstance/lau-tile-store" }
```

Requires Rust 2021 edition. Dependencies: `rusqlite` (bundled SQLite), `serde`, `serde_json`, `uuid`.

---

## Quick Start

```rust
use lau_tile_store::*;

// Open a store (file-backed or in-memory)
let store = TileStore::open("tiles.db")?;
// let store = TileStore::open_memory()?;  // for tests

// Create and store a tile
let obs = StorableTile::new(TileType::Observation, "User logged in")
    .with_room("session-1")
    .with_metadata("source", "auth-service");
store.store(&obs)?;

// Retrieve it
let tile = store.get(&obs.id)?.unwrap();
assert_eq!(tile.content, "User logged in");

// Create a chain: observation → thought → action
let thought = obs.child(TileType::Thought, "Should I greet them?");
store.store(&thought)?;

let action = thought.child(TileType::Action, "Sent greeting");
store.store(&action)?;

// Mark complete
let mut got = store.get(&action.id)?.unwrap();
got.complete();
store.store(&got)?;

// Query children and ancestors
let children = store.children_of(&obs.id)?;
let ancestors = store.ancestors(&action.id)?;

// Filtered query
let results = store.query(
    TileQuery::new()
        .in_room("session-1")
        .of_type(TileType::Action)
        .with_status(TileStatus::Complete)
        .newest_first()
        .limit(50)
)?;

// Export and import
let json = store.export_json(TileQuery::new())?;
let count = store.import_json(&json)?;
```

---

## API Reference

### `TileStore`

| Method | Description |
|--------|-------------|
| `open(path)` | Open file-backed store (creates if needed) |
| `open_memory()` | Open in-memory store (for testing) |
| `open_with_config(path, config)` | Open with custom configuration |
| `store(&tile)` | Insert or update a tile (upsert) |
| `get(id)` | Retrieve tile by ID |
| `delete(id)` | Delete tile by ID |
| `query(filter)` | Query with `TileQuery` filter |
| `count(filter)` | Count tiles matching filter |
| `children_of(parent_id)` | Get direct children |
| `room_tiles(room_id, limit)` | Get tiles in a room |
| `recent(limit)` | Most recent tiles |
| `ancestors(tile_id)` | Walk up parent chain |
| `descendants(tile_id)` | Walk down children recursively |
| `stats()` | Database statistics |
| `vacuum()` | Reclaim space |
| `export_json(filter)` | Export tiles as JSON |
| `import_json(json)` | Import tiles from JSON |

### `StorableTile`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Auto-generated UUID |
| `room_id` | `Option<String>` | Room assignment |
| `tile_type` | `TileType` | Observation, Action, Thought, Delegation, Escalation, Artifact, System, Onboarding, StandDown |
| `parent_id` | `Option<String>` | Parent tile (tree structure) |
| `status` | `TileStatus` | Active, Complete, Deadband, Escalated, Archived, Orphaned |
| `content` | `String` | Tile body |
| `content_type` | `Option<String>` | MIME type |
| `deadband_lower/upper/current` | `Option<f64>` | Monitoring bounds |
| `deadband_trend` | `Option<String>` | Trend direction |
| `ensign_id` | `Option<String>` | Agent identifier |
| `model_used` | `Option<String>` | LLM model used |
| `tokens_used` | `u32` | Token count |
| `conservation_delta` | `f64` | Energy delta |
| `metadata` | `HashMap<String, String>` | Arbitrary key-value data |
| `created_at` / `updated_at` | `i64` | Unix timestamps |

Builder methods: `.with_room()`, `.with_parent()`, `.with_metadata()`, `.child()`, `.complete()`, `.archive()`, `.escalate()`.

### `TileQuery`

Builder pattern with methods: `.in_room()`, `.of_type()`, `.with_status()`, `.with_parent()`, `.ensign_id()`, `.model()`, `.since()`, `.until()`, `.containing()`, `.limit()`, `.offset()`, `.newest_first()`, `.oldest_first()`, `.recent_update()`.

### `StoreConfig`

| Field | Default | Description |
|-------|---------|-------------|
| `wal_mode` | `true` | Enable WAL journal mode |
| `busy_timeout_ms` | 5000 | SQLite busy timeout |
| `journal_size_limit` | None | WAL file size limit |
| `cache_size` | None | Page cache size |

Presets: `StoreConfig::default()`, `StoreConfig::high_performance()`, `StoreConfig::minimal()`.

### `TileType`

`Observation`, `Action`, `Thought`, `Delegation`, `Escalation`, `Artifact`, `System`, `Onboarding`, `StandDown` (9 variants).

### `TileStatus`

`Active` (default), `Complete`, `Deadband`, `Escalated`, `Archived`, `Orphaned`. Terminal states: `Complete`, `Archived`, `Orphaned`.

### `StoreStats`

`total_tiles`, `total_rooms`, `by_type` (HashMap), `by_status` (HashMap), `db_size_bytes`, `oldest_tile`, `newest_tile`.

---

## How It Works

### Schema

Single `tiles` table with indexes on `room_id`, `tile_type`, `status`, `parent_id`, `ensign_id`, `created_at`, and `model_used`. Schema is created automatically on `open()` via `CREATE TABLE IF NOT EXISTS`.

### Queries

`TileQuery` builds a SQL WHERE clause dynamically. Each filter method adds a clause and a parameter. The `to_sql()` method returns `(WHERE_clause, params_vec)`. Ordering is configurable (newest first, oldest first, recent update).

### Hierarchy

Parent-child relationships use the `parent_id` field. `children_of()` queries direct children. `ancestors()` walks up the chain iteratively. `descendants()` does a BFS/DFS using a stack.

### Metadata

Metadata is stored as a JSON string in SQLite and deserialized on read. Custom serde serializers handle the conversion transparently.

### Export/Import

`export_json()` serializes queried tiles to pretty JSON. `import_json()` deserializes a JSON array and stores each tile. This enables backup, migration, and cross-store transfer.

---

## The Math

The tile store implements a **directed tree** (each tile has at most one parent):

```
children(parent) = { t | t.parent_id = parent.id }
ancestors(tile) = tile ∪ ancestors(parent(tile))
descendants(tile) = children(tile) ∪ ⋃ descendants(child) for child in children(tile)
```

The conservation ledger per tile tracks:

```
conservation_delta = energy_in - energy_out
```

Aggregated across a room or the full store via `stats()`.

---

## License

MIT
