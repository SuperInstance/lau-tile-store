# lau-tile-store

**THE memory system.** SQLite tile CRUD in 10 lines. Tiles ARE memory — no separate system needed.

```rust
use lau_tile_store::*;

let store = TileStore::open("tiles.db")?;

let obs = StorableTile::new(TileType::Observation, "User logged in").with_room("session-1");
store.store(&obs)?;

let thought = obs.child(TileType::Thought, "Should I greet them?");
store.store(&thought)?;

let active = store.query(TileQuery::new().in_room("session-1").newest_first().limit(10))?;
```

## Query Examples

```rust
// All tiles in a room
store.room_tiles("engineering", 50)?;

// Active observations since a timestamp
store.query(TileQuery::new().of_type(TileType::Observation).with_status(TileStatus::Active).since(1700000000))?;

// Content search
store.query(TileQuery::new().containing("deploy").limit(20))?;

// Hierarchy traversal
store.children_of(&parent_id)?;
store.ancestors(&child_id)?;
store.descendants(&root_id)?;   // recursive
```

## Export / Import Roundtrip

```rust
let json = store.export_json(TileQuery::new().in_room("session-1"))?;
let count = store.import_json(&json)?;
// count tiles migrated, zero data loss
```

## Stats

```rust
let stats = store.stats()?;
println!("{} tiles in {} rooms, {:.1} KB on disk",
    stats.total_tiles, stats.total_rooms, stats.db_size_bytes as f64 / 1024.0);
```

## Configuration

```rust
// Default: WAL mode, 5s busy timeout
TileStore::open("tiles.db")?;

// High performance: 100MB journal, 8GB cache
TileStore::open_with_config("tiles.db", StoreConfig::high_performance())?;

// Minimal: no WAL, 1s timeout
TileStore::open_with_config("tiles.db", StoreConfig::minimal())?;

// In-memory (tests)
TileStore::open_memory()?;
```

## Tile Types & Statuses

**Types:** Observation · Action · Thought · Delegation · Escalation · Artifact · System · Onboarding · StandDown

**Statuses:** Active · Complete · Deadband · Escalated · Archived · Orphaned

## Tests

**60 integration tests** — full CRUD, hierarchy traversal, query filtering, export/import roundtrip, stats, concurrent access.

## Ecosystem

- [lau-shell-kernel] — bare construct (includes in-memory TileStore)
- [lau-provider] — LLM provider abstraction
- **[lau-tile-store]** (this) — SQLite-backed tile persistence
- [lau-git-agent] — repo-as-agent
- [lau-git-render] — multi-format rendering

## License

MIT
