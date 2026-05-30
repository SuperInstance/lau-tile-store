# lau-tile-store

SQLite-backed tile persistence layer for agent systems. System-agnostic, synchronous, and fast.

## Overview

A **tile** is the fundamental unit of agent cognition — an observation, thought, action, artifact, etc. This crate provides durable SQLite storage for tiles with querying, hierarchy traversal, and import/export.

## Quick Start

```rust
use lau_tile_store::*;

let store = TileStore::open("tiles.db").unwrap();

// Create and store a tile
let obs = StorableTile::new(TileType::Observation, "User logged in")
    .with_room("session-1");
store.store(&obs).unwrap();

// Create a child thought
let thought = obs.child(TileType::Thought, "Should I greet them?");
store.store(&thought).unwrap();

// Query tiles
let active = store.query(
    TileQuery::new()
        .in_room("session-1")
        .with_status(TileStatus::Active)
        .newest_first()
        .limit(10)
).unwrap();

// Walk hierarchy
let ancestors = store.ancestors(&thought.id).unwrap();
let descendants = store.descendants(&obs.id).unwrap();

// Stats
let stats = store.stats().unwrap();
println!("{} tiles in {} rooms", stats.total_tiles, stats.total_rooms);
```

## Tile Types

`Observation`, `Action`, `Thought`, `Delegation`, `Escalation`, `Artifact`, `System`, `Onboarding`, `StandDown`

## Tile Statuses

`Active`, `Complete`, `Deadband`, `Escalated`, `Archived`, `Orphaned`

## Configuration

```rust
// Default (WAL mode, 5s busy timeout)
TileStore::open("tiles.db").unwrap();

// High performance
TileStore::open_with_config("tiles.db", StoreConfig::high_performance()).unwrap();

// Minimal (no WAL, 1s timeout)
TileStore::open_with_config("tiles.db", StoreConfig::minimal()).unwrap();
```

## License

MIT
