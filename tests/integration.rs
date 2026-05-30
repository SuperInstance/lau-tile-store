use lau_tile_store::*;

fn make_store() -> TileStore {
    TileStore::open_memory().unwrap()
}

fn make_tile(tt: TileType, content: &str) -> StorableTile {
    StorableTile::new(tt, content)
}

// ─── TileType tests ───

#[test]
fn tile_type_roundtrip() {
    for tt in TileType::all() {
        assert_eq!(TileType::parse(tt.as_str()), Some(tt));
    }
}

#[test]
fn tile_type_unknown() {
    assert_eq!(TileType::parse("bogus"), None);
}

#[test]
fn tile_type_display() {
    assert_eq!(format!("{}", TileType::Action), "action");
}

#[test]
fn tile_type_all_count() {
    assert_eq!(TileType::all().len(), 9);
}

// ─── TileStatus tests ───

#[test]
fn status_roundtrip() {
    for s in &[
        TileStatus::Active,
        TileStatus::Complete,
        TileStatus::Deadband,
        TileStatus::Escalated,
        TileStatus::Archived,
        TileStatus::Orphaned,
    ] {
        assert_eq!(TileStatus::parse(s.as_str()), Some(*s));
    }
}

#[test]
fn status_terminal() {
    assert!(TileStatus::Complete.is_terminal());
    assert!(TileStatus::Archived.is_terminal());
    assert!(TileStatus::Orphaned.is_terminal());
    assert!(!TileStatus::Active.is_terminal());
    assert!(!TileStatus::Escalated.is_terminal());
}

#[test]
fn status_display() {
    assert_eq!(format!("{}", TileStatus::Active), "active");
}

#[test]
fn status_default() {
    assert_eq!(TileStatus::default(), TileStatus::Active);
}

// ─── StorableTile tests ───

#[test]
fn tile_new_defaults() {
    let t = make_tile(TileType::Observation, "saw something");
    assert!(!t.id.is_empty());
    assert_eq!(t.tile_type, TileType::Observation);
    assert_eq!(t.content, "saw something");
    assert_eq!(t.status, TileStatus::Active);
    assert_eq!(t.tokens_used, 0);
    assert!(t.room_id.is_none());
    assert!(t.parent_id.is_none());
}

#[test]
fn tile_builder_pattern() {
    let t = StorableTile::new(TileType::Thought, "hmm")
        .with_room("r1")
        .with_parent("p1")
        .with_metadata("key", "val");
    assert_eq!(t.room_id.as_deref(), Some("r1"));
    assert_eq!(t.parent_id.as_deref(), Some("p1"));
    assert_eq!(t.metadata.get("key").map(|s| s.as_str()), Some("val"));
}

#[test]
fn tile_child() {
    let parent = make_tile(TileType::Observation, "parent");
    let child = parent.child(TileType::Action, "child");
    assert_eq!(child.parent_id.as_deref(), Some(parent.id.as_str()));
}

#[test]
fn tile_complete() {
    let mut t = make_tile(TileType::Action, "do it");
    t.complete();
    assert_eq!(t.status, TileStatus::Complete);
}

#[test]
fn tile_archive() {
    let mut t = make_tile(TileType::Action, "do it");
    t.archive();
    assert_eq!(t.status, TileStatus::Archived);
}

#[test]
fn tile_escalate() {
    let mut t = make_tile(TileType::Action, "do it");
    t.escalate();
    assert_eq!(t.status, TileStatus::Escalated);
}

#[test]
fn tile_age_seconds() {
    let t = make_tile(TileType::Observation, "now");
    assert!(t.age_seconds() >= 0);
    assert!(t.age_seconds() < 5);
}

#[test]
fn tile_metadata_json_roundtrip() {
    let mut t = make_tile(TileType::System, "sys");
    t.metadata.insert("foo".into(), "bar".into());
    let json = t.metadata_json();
    let parsed = StorableTile::metadata_from_json(&json);
    assert_eq!(parsed.get("foo").map(|s| s.as_str()), Some("bar"));
}

// ─── StoreConfig tests ───

#[test]
fn config_default() {
    let c = StoreConfig::default();
    assert!(c.wal_mode);
    assert_eq!(c.busy_timeout_ms, 5000);
}

#[test]
fn config_high_performance() {
    let c = StoreConfig::high_performance();
    assert!(c.wal_mode);
    assert_eq!(c.busy_timeout_ms, 10000);
    assert!(c.journal_size_limit.is_some());
    assert!(c.cache_size.is_some());
}

#[test]
fn config_minimal() {
    let c = StoreConfig::minimal();
    assert!(!c.wal_mode);
    assert_eq!(c.busy_timeout_ms, 1000);
}

// ─── Store open/close tests ───

#[test]
fn open_memory() {
    let store = make_store();
    let stats = store.stats().unwrap();
    assert_eq!(stats.total_tiles, 0);
}

#[test]
fn open_file_based() {
    let dir = std::env::temp_dir();
    let path = dir.join("lau_test_open_file_unique.db");
    // Clean up from previous runs
    let _ = std::fs::remove_file(&path);
    let path_str = path.to_str().unwrap();
    {
        let store = TileStore::open(path_str).unwrap();
        store.store(&make_tile(TileType::System, "hello")).unwrap();
    }
    // Reopen
    let store = TileStore::open(path_str).unwrap();
    assert_eq!(store.stats().unwrap().total_tiles, 1);
}

#[test]
fn open_with_custom_config() {
    let store = TileStore::open_with_config(":memory:", StoreConfig::minimal()).unwrap();
    store.store(&make_tile(TileType::System, "cfg")).unwrap();
    assert_eq!(store.count(&TileQuery::new()).unwrap(), 1);
}

// ─── Store CRUD tests ───

#[test]
fn store_and_get() {
    let store = make_store();
    let t = make_tile(TileType::Observation, "obs1");
    let id = t.id.clone();
    store.store(&t).unwrap();
    let got = store.get(&id).unwrap().unwrap();
    assert_eq!(got.content, "obs1");
    assert_eq!(got.tile_type, TileType::Observation);
}

#[test]
fn store_update() {
    let store = make_store();
    let mut t = make_tile(TileType::Action, "v1");
    store.store(&t).unwrap();
    t.content = "v2".to_string();
    store.store(&t).unwrap();
    let got = store.get(&t.id).unwrap().unwrap();
    assert_eq!(got.content, "v2");
}

#[test]
fn get_nonexistent() {
    let store = make_store();
    assert!(store.get("nope").unwrap().is_none());
}

#[test]
fn delete_tile() {
    let store = make_store();
    let t = make_tile(TileType::Thought, "think");
    store.store(&t).unwrap();
    store.delete(&t.id).unwrap();
    assert!(store.get(&t.id).unwrap().is_none());
}

#[test]
fn delete_nonexistent() {
    let store = make_store();
    assert!(store.delete("nope").is_err());
}

// ─── Query tests ───

#[test]
fn query_all() {
    let store = make_store();
    store.store(&make_tile(TileType::Observation, "a")).unwrap();
    store.store(&make_tile(TileType::Action, "b")).unwrap();
    assert_eq!(store.query(TileQuery::new()).unwrap().len(), 2);
}

#[test]
fn query_by_type() {
    let store = make_store();
    store.store(&make_tile(TileType::Observation, "a")).unwrap();
    store.store(&make_tile(TileType::Action, "b")).unwrap();
    let res = store.query(TileQuery::new().of_type(TileType::Observation)).unwrap();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].tile_type, TileType::Observation);
}

#[test]
fn query_by_status() {
    let store = make_store();
    let mut t = make_tile(TileType::Action, "done");
    t.complete();
    store.store(&t).unwrap();
    store.store(&make_tile(TileType::Action, "pending")).unwrap();
    let res = store.query(TileQuery::new().with_status(TileStatus::Complete)).unwrap();
    assert_eq!(res.len(), 1);
}

#[test]
fn query_by_room() {
    let store = make_store();
    store.store(&make_tile(TileType::Observation, "r1").with_room("room1")).unwrap();
    store.store(&make_tile(TileType::Observation, "r2").with_room("room2")).unwrap();
    let res = store.query(TileQuery::new().in_room("room1")).unwrap();
    assert_eq!(res.len(), 1);
}

#[test]
fn query_content_contains() {
    let store = make_store();
    store.store(&make_tile(TileType::Thought, "hello world")).unwrap();
    store.store(&make_tile(TileType::Thought, "foo bar")).unwrap();
    let res = store.query(TileQuery::new().containing("hello")).unwrap();
    assert_eq!(res.len(), 1);
    assert!(res[0].content.contains("hello"));
}

#[test]
fn query_with_limit() {
    let store = make_store();
    for i in 0..10 {
        store.store(&make_tile(TileType::System, &format!("t{i}"))).unwrap();
    }
    let res = store.query(TileQuery::new().limit(3)).unwrap();
    assert_eq!(res.len(), 3);
}

#[test]
fn query_with_offset() {
    let store = make_store();
    for i in 0..5 {
        store.store(&make_tile(TileType::System, &format!("t{i}"))).unwrap();
    }
    let res = store.query(TileQuery::new().limit(2).offset(3)).unwrap();
    assert_eq!(res.len(), 2);
}

#[test]
fn query_since_until() {
    let store = make_store();
    let mut t = make_tile(TileType::Observation, "old");
    t.created_at = 1000;
    t.updated_at = 1000;
    store.store(&t).unwrap();
    let mut t2 = make_tile(TileType::Observation, "new");
    t2.created_at = 2000;
    t2.updated_at = 2000;
    store.store(&t2).unwrap();
    let res = store.query(TileQuery::new().since(1500)).unwrap();
    assert_eq!(res.len(), 1);
    let res = store.query(TileQuery::new().until(1500)).unwrap();
    assert_eq!(res.len(), 1);
}

#[test]
fn query_order_newest_first() {
    let store = make_store();
    let mut t1 = make_tile(TileType::System, "old");
    t1.created_at = 100;
    let mut t2 = make_tile(TileType::System, "new");
    t2.created_at = 200;
    store.store(&t1).unwrap();
    store.store(&t2).unwrap();
    let res = store.query(TileQuery::new().newest_first()).unwrap();
    assert_eq!(res[0].content, "new");
}

#[test]
fn query_order_oldest_first() {
    let store = make_store();
    let mut t1 = make_tile(TileType::System, "old");
    t1.created_at = 100;
    let mut t2 = make_tile(TileType::System, "new");
    t2.created_at = 200;
    store.store(&t1).unwrap();
    store.store(&t2).unwrap();
    let res = store.query(TileQuery::new().oldest_first()).unwrap();
    assert_eq!(res[0].content, "old");
}

#[test]
fn query_order_recent_update() {
    let store = make_store();
    let mut t1 = make_tile(TileType::System, "a");
    t1.created_at = 100;
    t1.updated_at = 100;
    let mut t2 = make_tile(TileType::System, "b");
    t2.created_at = 200;
    t2.updated_at = 50; // older update
    store.store(&t1).unwrap();
    store.store(&t2).unwrap();
    let res = store.query(TileQuery::new().recent_update()).unwrap();
    assert_eq!(res[0].content, "a"); // most recently updated
}

#[test]
fn query_by_ensign() {
    let store = make_store();
    let mut t = make_tile(TileType::Observation, "ens");
    t.ensign_id = Some("e1".into());
    store.store(&t).unwrap();
    store.store(&make_tile(TileType::Observation, "noens")).unwrap();
    let _res = store.query(TileQuery::new().ensign_id("e1")).unwrap();
    // TileQuery doesn't have ensign_id builder — let me add one
    // Actually we need to handle this differently since the field is pub
    let mut q = TileQuery::new();
    q.ensign_id = Some("e1".into());
    let res = store.query(q).unwrap();
    assert_eq!(res.len(), 1);
}

#[test]
fn query_by_model() {
    let store = make_store();
    let mut t = make_tile(TileType::Thought, "model");
    t.model_used = Some("gpt-4".into());
    store.store(&t).unwrap();
    let mut q = TileQuery::new();
    q.model_used = Some("gpt-4".into());
    let res = store.query(q).unwrap();
    assert_eq!(res.len(), 1);
}

// ─── Count tests ───

#[test]
fn count_all() {
    let store = make_store();
    store.store(&make_tile(TileType::System, "a")).unwrap();
    store.store(&make_tile(TileType::System, "b")).unwrap();
    assert_eq!(store.count(&TileQuery::new()).unwrap(), 2);
}

#[test]
fn count_with_filter() {
    let store = make_store();
    store.store(&make_tile(TileType::Observation, "a")).unwrap();
    store.store(&make_tile(TileType::Action, "b")).unwrap();
    assert_eq!(
        store.count(&TileQuery::new().of_type(TileType::Observation)).unwrap(),
        1
    );
}

// ─── Children / Room / Recent tests ───

#[test]
fn children_of() {
    let store = make_store();
    let parent = make_tile(TileType::Thought, "parent");
    let c1 = parent.child(TileType::Action, "c1");
    let c2 = parent.child(TileType::Action, "c2");
    store.store(&parent).unwrap();
    store.store(&c1).unwrap();
    store.store(&c2).unwrap();
    let kids = store.children_of(&parent.id).unwrap();
    assert_eq!(kids.len(), 2);
}

#[test]
fn room_tiles() {
    let store = make_store();
    store.store(&make_tile(TileType::System, "r1a").with_room("r1")).unwrap();
    store.store(&make_tile(TileType::System, "r1b").with_room("r1")).unwrap();
    store.store(&make_tile(TileType::System, "r2a").with_room("r2")).unwrap();
    let tiles = store.room_tiles("r1", 10).unwrap();
    assert_eq!(tiles.len(), 2);
}

#[test]
fn room_tiles_limit() {
    let store = make_store();
    for i in 0..5 {
        store.store(&make_tile(TileType::System, &format!("t{i}")).with_room("r1")).unwrap();
    }
    let tiles = store.room_tiles("r1", 3).unwrap();
    assert_eq!(tiles.len(), 3);
}

#[test]
fn recent_tiles() {
    let store = make_store();
    for i in 0..5 {
        store.store(&make_tile(TileType::System, &format!("t{i}"))).unwrap();
    }
    let tiles = store.recent(3).unwrap();
    assert_eq!(tiles.len(), 3);
}

// ─── Ancestors / Descendants tests ───

#[test]
fn ancestors_chain() {
    let store = make_store();
    let grandparent = make_tile(TileType::Observation, "gp");
    let parent = grandparent.child(TileType::Thought, "p");
    let child = parent.child(TileType::Action, "c");
    store.store(&grandparent).unwrap();
    store.store(&parent).unwrap();
    store.store(&child).unwrap();
    let anc = store.ancestors(&child.id).unwrap();
    // ancestors returns the child itself + parent, but not grandparent since parent has no parent... wait
    // Actually ancestors walks up from tile_id, starting with tile_id itself, following parent_id
    // So for child: child (parent=p.id), parent (parent=gp.id), then gp has no parent -> stop
    // result = [child, parent] — the loop pushes before breaking
    assert_eq!(anc.len(), 2);
}

#[test]
fn ancestors_no_parent() {
    let store = make_store();
    let t = make_tile(TileType::System, "root");
    store.store(&t).unwrap();
    let anc = store.ancestors(&t.id).unwrap();
    assert!(anc.is_empty());
}

#[test]
fn descendants_tree() {
    let store = make_store();
    let root = make_tile(TileType::Thought, "root");
    let c1 = root.child(TileType::Action, "c1");
    let c2 = root.child(TileType::Action, "c2");
    let gc1 = c1.child(TileType::Artifact, "gc1");
    store.store(&root).unwrap();
    store.store(&c1).unwrap();
    store.store(&c2).unwrap();
    store.store(&gc1).unwrap();
    let desc = store.descendants(&root.id).unwrap();
    assert_eq!(desc.len(), 3);
}

// ─── Stats tests ───

#[test]
fn stats_empty() {
    let store = make_store();
    let stats = store.stats().unwrap();
    assert_eq!(stats.total_tiles, 0);
    assert_eq!(stats.total_rooms, 0);
    assert!(stats.oldest_tile.is_none());
}

#[test]
fn stats_with_data() {
    let store = make_store();
    store.store(&make_tile(TileType::Observation, "a").with_room("r1")).unwrap();
    store.store(&make_tile(TileType::Action, "b").with_room("r2")).unwrap();
    let mut t = make_tile(TileType::Thought, "c");
    t.complete();
    store.store(&t).unwrap();
    let stats = store.stats().unwrap();
    assert_eq!(stats.total_tiles, 3);
    assert_eq!(stats.total_rooms, 2);
    assert_eq!(stats.by_type.get("observation"), Some(&1));
    assert_eq!(stats.by_status.get("complete"), Some(&1));
    assert!(stats.oldest_tile.is_some());
    assert!(stats.newest_tile.is_some());
}

// ─── Vacuum test ───

#[test]
fn vacuum_works() {
    let store = make_store();
    let t = make_tile(TileType::System, "tmp");
    store.store(&t).unwrap();
    store.delete(&t.id).unwrap();
    store.vacuum().unwrap();
}

// ─── Export / Import tests ───

#[test]
fn export_json() {
    let store = make_store();
    store.store(&make_tile(TileType::Observation, "export me")).unwrap();
    let json = store.export_json(TileQuery::new()).unwrap();
    assert!(json.contains("export me"));
    assert!(json.starts_with('['));
}

#[test]
fn import_json() {
    let store = make_store();
    let tiles = vec![
        make_tile(TileType::Observation, "imported1"),
        make_tile(TileType::Action, "imported2"),
    ];
    let json = serde_json::to_string(&tiles).unwrap();
    let count = store.import_json(&json).unwrap();
    assert_eq!(count, 2);
    assert_eq!(store.count(&TileQuery::new()).unwrap(), 2);
}

#[test]
fn export_import_roundtrip() {
    let store = make_store();
    let t = make_tile(TileType::Artifact, "roundtrip").with_room("r1").with_metadata("k", "v");
    store.store(&t).unwrap();
    let json = store.export_json(TileQuery::new()).unwrap();
    let store2 = make_store();
    store2.import_json(&json).unwrap();
    let tiles = store2.query(TileQuery::new()).unwrap();
    assert_eq!(tiles.len(), 1);
    assert_eq!(tiles[0].content, "roundtrip");
    assert_eq!(tiles[0].room_id.as_deref(), Some("r1"));
}

#[test]
fn import_invalid_json() {
    let store = make_store();
    assert!(store.import_json("not json").is_err());
}

// ─── Error type tests ───

#[test]
fn error_display() {
    let e = StoreError::NotFound("x".into());
    assert!(e.to_string().contains("not found"));
    let e = StoreError::OpenFailed("x".into());
    assert!(e.to_string().contains("open failed"));
    let e = StoreError::QueryFailed("x".into());
    assert!(e.to_string().contains("query failed"));
    let e = StoreError::InvalidData("x".into());
    assert!(e.to_string().contains("invalid data"));
    let e = StoreError::IoError("x".into());
    assert!(e.to_string().contains("io error"));
}

// ─── Query builder tests ───

#[test]
fn query_builder_chaining() {
    let q = TileQuery::new()
        .in_room("r")
        .of_type(TileType::Action)
        .with_status(TileStatus::Active)
        .since(0)
        .until(9999)
        .limit(10)
        .offset(5)
        .containing("hello")
        .newest_first();
    assert_eq!(q.room_id.as_deref(), Some("r"));
    assert_eq!(q.tile_type, Some(TileType::Action));
    assert_eq!(q.status, Some(TileStatus::Active));
    assert_eq!(q.since, Some(0));
    assert_eq!(q.until, Some(9999));
    assert_eq!(q.limit, Some(10));
    assert_eq!(q.offset, Some(5));
    assert_eq!(q.content_contains.as_deref(), Some("hello"));
}

// ─── Complex integration test ───

#[test]
fn full_workflow() {
    let store = make_store();

    // Create a thought chain
    let obs = StorableTile::new(TileType::Observation, "User logged in")
        .with_room("session-1");
    store.store(&obs).unwrap();

    let thought = obs.child(TileType::Thought, "Should I greet them?");
    store.store(&thought).unwrap();

    let action = thought.child(TileType::Action, "Sent greeting");
    store.store(&action).unwrap();
    
    let mut got = store.get(&action.id).unwrap().unwrap();
    got.complete();
    store.store(&got).unwrap();

    // Verify hierarchy
    let ancestors = store.ancestors(&action.id).unwrap();
    assert_eq!(ancestors.len(), 2);

    let descendants = store.descendants(&obs.id).unwrap();
    assert_eq!(descendants.len(), 2);

    // Stats
    let stats = store.stats().unwrap();
    assert_eq!(stats.total_tiles, 3);
    assert_eq!(stats.total_rooms, 1);
    assert_eq!(stats.by_type.get("action"), Some(&1));
    assert_eq!(stats.by_status.get("complete"), Some(&1));
}

#[test]
fn store_with_all_fields() {
    let store = make_store();
    let mut t = StorableTile::new(TileType::Observation, "full tile")
        .with_room("r1")
        .with_parent("p1")
        .with_metadata("env", "prod");
    t.content_type = Some("text/plain".into());
    t.deadband_lower = Some(10.0);
    t.deadband_upper = Some(20.0);
    t.deadband_current = Some(15.0);
    t.deadband_trend = Some("rising".into());
    t.ensign_id = Some("ens1".into());
    t.model_used = Some("gpt-4".into());
    t.tokens_used = 42;
    t.conservation_delta = 0.5;
    store.store(&t).unwrap();

    let got = store.get(&t.id).unwrap().unwrap();
    assert_eq!(got.content_type.as_deref(), Some("text/plain"));
    assert_eq!(got.deadband_lower, Some(10.0));
    assert_eq!(got.deadband_upper, Some(20.0));
    assert_eq!(got.deadband_current, Some(15.0));
    assert_eq!(got.deadband_trend.as_deref(), Some("rising"));
    assert_eq!(got.ensign_id.as_deref(), Some("ens1"));
    assert_eq!(got.model_used.as_deref(), Some("gpt-4"));
    assert_eq!(got.tokens_used, 42);
    assert!((got.conservation_delta - 0.5).abs() < f64::EPSILON);
    assert_eq!(got.metadata.get("env").map(|s| s.as_str()), Some("prod"));
}
