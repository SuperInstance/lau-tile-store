# lau-tile-store

SQLite-backed tile persistence. CRUD, hierarchy, queries, export/import — in about 10 lines.

Tiles *are* memory. This crate makes that memory durable.

## The concept in 60 seconds

A **tile** is a typed data unit with an ID, content, room, parent, metadata, and status. The `TileStore` persists tiles in SQLite with full CRUD, hierarchical queries (parent/children), filtering by type/room/status, and import/export. It's the persistence layer that turns ephemeral in-memory tiles into something that survives restarts.

No ORM, no migrations, no ceremony. Open a database, store tiles, query them.

## Quick start

```rust
use lau_tile_store::*;

// Open a store (file or in-memory)
let store = TileStore::open("tiles.db")?;
let store = TileStore::open_memory()?;  // for tests

// Create and store a tile
let tile = StorableTile::new(TileType::Memory, "sensor reading: 42.3°C")
    .with_room("engineering")
    .with_metadata("source", "thermal-sensor-1");
store.store(&tile)?;

// Retrieve it
let retrieved = store.get(&tile.id)?.unwrap();

// Create a child tile
let child = tile.child(TileType::Event, "threshold exceeded");
store.store(&child)?;

// Query children
let children = store.children_of(&tile.id)?;

// Query by room
let room_tiles = store.room_tiles("engineering", 100)?;

// Filter with TileQuery
let results = store.query(
    TileQuery::new()
        .with_type(TileType::Memory)
        .in_room("engineering")
        .with_status(TileStatus::Active)
        .limit(50)
        .order(QueryOrder::NewestFirst)
)?;
```

## Key types

| Type | What it does |
|------|-------------|
| `TileStore` | SQLite-backed store: CRUD, queries, hierarchy |
| `StorableTile` | A persistent tile: id, type, content, room, parent, metadata |
| `TileQuery` | Filter builder: type, room, status, parent, limit, order |
| `StoreConfig` | Database configuration (WAL mode, pool size, etc.) |
| `StoreStats` | Tile counts by type, room, status |
| `TileType` | Memory, Event, State, Config, Log, Custom |
| `TileStatus` | Active, Archived, Deleted |

## Tile hierarchy

```rust
let parent = StorableTile::new(TileType::Config, "room settings");
store.store(&parent)?;

let child1 = parent.child(TileType::Event, "setting changed: temp → 45");
store.store(&child1)?;

let child2 = parent.child(TileType::Event, "setting changed: mode → auto");
store.store(&child2)?;

// Query all children
let all = store.children_of(&parent.id)?;
assert_eq!(all.len(), 2);
```

## Query builder

```rust
let query = TileQuery::new()
    .with_type(TileType::Event)
    .in_room("bridge")
    .with_status(TileStatus::Active)
    .with_parent("parent-id")
    .limit(100)
    .order(QueryOrder::NewestFirst);

let tiles = store.query(query)?;
let count = store.count(&query)?;
```

## Contributing

PRs welcome. This crate is part of the [SuperInstance](https://github.com/SuperInstance) ecosystem. The SQLite schema is intentionally simple — if you need indexes or query features that aren't there yet, open an issue.
