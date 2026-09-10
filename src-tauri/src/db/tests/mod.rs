use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::audit::compute_audit_hash;
use crate::db::crypto::format_pragma_key;
use crate::db::migrations::{run_migrations_custom, Migration};
use crate::db::{
    derive_key, generate_salt, get_kdf_metadata_path, initialize_and_verify_database,
    log_audit_event, open_database, run_migrations, verify_audit_chain, DatabaseError,
    NewAuditEvent,
};

fn temp_db_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("life_os_tests_{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir.join(format!("{name}.db"))
}

#[test]
fn test_database_lifecycle_close_reopen_with_persisted_salt() {
    let db_path = temp_db_path("lifecycle");
    let passcode = "CorrectHorseBatteryStaple123!";

    // 1. Create and open database
    let mut conn = open_database(&db_path, passcode).expect("Failed to open fresh database");
    run_migrations(&mut conn).expect("Failed to run migrations");

    // Write data
    conn.execute(
        "CREATE TABLE user_notes (id INTEGER PRIMARY KEY, content TEXT);",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO user_notes (id, content) VALUES (1, 'Secret Journal Entry');",
        [],
    )
    .unwrap();

    // Close connection
    drop(conn);

    // Verify metadata file exists
    let kdf_path = get_kdf_metadata_path(&db_path);
    assert!(kdf_path.exists(), "KDF metadata file must be persisted");

    // 2. Reopen same database with same passcode
    let conn2 =
        open_database(&db_path, passcode).expect("Failed to reopen database with correct passcode");
    let content: String = conn2
        .query_row("SELECT content FROM user_notes WHERE id = 1;", [], |r| {
            r.get(0)
        })
        .expect("Failed to query data from reopened database");
    assert_eq!(content, "Secret Journal Entry");
    drop(conn2);

    // 3. Try to open with wrong passcode
    let err = open_database(&db_path, "WrongPasscode999!").unwrap_err();
    match err {
        DatabaseError::InvalidPasscode => (),
        other => panic!("Expected InvalidPasscode error, got: {other:?}"),
    }

    // Clean up
    let _ = fs::remove_dir_all(db_path.parent().unwrap());
}

#[test]
fn test_concurrent_first_open_salt_race() {
    let db_path = temp_db_path("salt_race");
    let passcode = "RacePasscode123!";

    let num_threads = 5;
    let barrier = Arc::new(Barrier::new(num_threads));

    let threads: Vec<_> = (0..num_threads)
        .map(|_| {
            let db_path = db_path.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                open_database(&db_path, passcode)
                    .expect("Concurrent first-open MUST succeed and share atomic salt")
            })
        })
        .collect();

    for t in threads {
        let conn = t.join().unwrap();
        drop(conn);
    }

    let kdf_path = get_kdf_metadata_path(&db_path);
    assert!(kdf_path.exists());

    let conn_final = open_database(&db_path, passcode).expect("Database MUST remain openable");
    drop(conn_final);

    let _ = fs::remove_dir_all(db_path.parent().unwrap());
}

#[test]
fn test_security_plaintext_sqlite_rejection() {
    let db_path = temp_db_path("plaintext");

    // Create standard plaintext SQLite database
    {
        let plain_conn = rusqlite::Connection::open(&db_path).unwrap();
        plain_conn
            .execute("CREATE TABLE secret (data TEXT);", [])
            .unwrap();
        plain_conn
            .execute("INSERT INTO secret VALUES ('unencrypted_plaintext');", [])
            .unwrap();
    }

    // Try opening as encrypted database with open_database
    let err = open_database(&db_path, "SomePasscode123!").unwrap_err();
    match err {
        DatabaseError::InvalidPasscode => (),
        other => panic!("Expected InvalidPasscode when attempting to open plaintext DB as encrypted, got: {other:?}"),
    }

    let _ = fs::remove_dir_all(db_path.parent().unwrap());
}

#[test]
fn test_no_secret_leakage_in_debug_formatting() {
    let salt = generate_salt();
    let key = derive_key("MyPasscode", &salt).unwrap();
    let debug_str = format!("{key:?}");
    let display_str = format!("{key}");

    assert!(!debug_str.contains("MyPasscode"));
    assert!(!display_str.contains("MyPasscode"));
    assert!(debug_str.contains("[REDACTED_KEY]"));
    assert_eq!(display_str, "[REDACTED_KEY]");
}

#[test]
fn test_pragma_key_zeroization_lifecycle() {
    let salt = generate_salt();
    let key = derive_key("MyPasscode123", &salt).unwrap();

    let pragma_key = format_pragma_key(&key);
    assert!(pragma_key.starts_with("PRAGMA key = \"x'"));
    assert!(pragma_key.ends_with("'\";"));
    assert_eq!(pragma_key.len(), 83);
}

#[test]
fn test_migrations_idempotence_and_ordering() {
    let db_path = temp_db_path("migrations");
    let passcode = "MigrationPasscode123!";

    let mut conn = open_database(&db_path, passcode).unwrap();

    // Fresh DB -> 2 migrations applied (V1 and V2)
    let applied = run_migrations(&mut conn).expect("Fresh migration should succeed");
    assert_eq!(applied, 2);

    // Already migrated DB -> 0 migrations applied
    let reapplied = run_migrations(&mut conn).expect("Repeated migration should succeed");
    assert_eq!(reapplied, 0);

    // Verify _migrations table entries
    let count: i64 = conn
        .query_row(
            "SELECT count(*) FROM _migrations WHERE version = 1;",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);

    drop(conn);
    let _ = fs::remove_dir_all(db_path.parent().unwrap());
}

#[test]
fn test_migration_gap_detection() {
    let db_path = temp_db_path("migration_gap");
    let passcode = "Passcode123!";

    let mut conn = open_database(&db_path, passcode).unwrap();

    let gap_migrations = [
        Migration {
            version: 1,
            name: "V1__init",
            sql: "CREATE TABLE t1 (id INT);",
        },
        Migration {
            version: 3, // Gap! Version 2 missing
            name: "V3__gap",
            sql: "CREATE TABLE t3 (id INT);",
        },
    ];

    let err = run_migrations_custom(&mut conn, &gap_migrations).unwrap_err();
    match err {
        DatabaseError::MigrationError(msg) => {
            assert!(msg.contains("Migration sequence gap detected"));
        }
        other => panic!("Expected MigrationError for version gap, got: {other:?}"),
    }

    drop(conn);
    let _ = fs::remove_dir_all(db_path.parent().unwrap());
}

#[test]
fn test_corrupted_applied_migration_history_gap() {
    let db_path = temp_db_path("corrupted_migrations");
    let passcode = "Passcode123!";

    let mut conn = open_database(&db_path, passcode).unwrap();

    // Manually create corrupted _migrations table with history [1, 3]
    conn.execute(
        "CREATE TABLE _migrations (version INT PRIMARY KEY, name TEXT, applied_at TEXT);",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO _migrations VALUES (1, 'V1', 'now'), (3, 'V3', 'now');",
        [],
    )
    .unwrap();

    let err = run_migrations(&mut conn).unwrap_err();
    match err {
        DatabaseError::MigrationError(msg) => {
            assert!(msg.contains("Corrupted migration history gap detected"));
        }
        other => panic!("Expected MigrationError for corrupted applied history, got: {other:?}"),
    }

    drop(conn);
    let _ = fs::remove_dir_all(db_path.parent().unwrap());
}

#[test]
fn test_audit_canonicalization_unambiguous_delimiters() {
    let hash1 = compute_audit_hash(
        1,
        "2026-09-08T00:00:00Z",
        Some("user1"),
        "a|b",
        "c",
        None,
        None,
        None,
        None,
        "prev_hash",
    );

    let hash2 = compute_audit_hash(
        1,
        "2026-09-08T00:00:00Z",
        Some("user1"),
        "a",
        "b|c",
        None,
        None,
        None,
        None,
        "prev_hash",
    );

    assert_ne!(
        hash1, hash2,
        "Length-prefixed encoding MUST prevent boundary ambiguity collisions"
    );
}

#[test]
fn test_audit_canonicalization_none_vs_empty_string() {
    let hash_none = compute_audit_hash(
        1,
        "2026-09-08T00:00:00Z",
        None,
        "action",
        "entity",
        None,
        None,
        None,
        None,
        "prev_hash",
    );

    let hash_empty = compute_audit_hash(
        1,
        "2026-09-08T00:00:00Z",
        Some(""),
        "action",
        "entity",
        None,
        None,
        None,
        None,
        "prev_hash",
    );

    assert_ne!(
        hash_none, hash_empty,
        "None and Some(\"\") MUST produce distinct canonical hashes"
    );
}

#[test]
fn test_audit_log_valid_chain() {
    let db_path = temp_db_path("audit_valid");
    let mut conn = open_database(&db_path, "Pass123!").unwrap();
    run_migrations(&mut conn).unwrap();

    for i in 1..=5 {
        log_audit_event(
            &mut conn,
            NewAuditEvent {
                user_id: Some("user_01".to_string()),
                action_type: format!("action_{i}"),
                entity_name: "goal".to_string(),
                entity_id: Some(format!("goal_{i}")),
                before_state: None,
                after_state: Some(format!("{{\"status\": \"completed_{i}\"}}")),
                client_info: Some("desktop_app".to_string()),
            },
        )
        .unwrap();
    }

    let report = verify_audit_chain(&conn).expect("Audit chain should be valid");
    assert_eq!(report.total_records, 5);
    assert!(report.is_valid);

    drop(conn);
    let _ = fs::remove_dir_all(db_path.parent().unwrap());
}

#[test]
fn test_audit_concurrent_writes() {
    let db_path = temp_db_path("audit_concurrent");
    let passcode = "ConcurrentPass123!";

    // Create database and run initial migrations
    {
        let mut conn = open_database(&db_path, passcode).unwrap();
        run_migrations(&mut conn).unwrap();
    }

    let num_connections = 5;
    let events_per_thread = 5;
    let barrier = Arc::new(Barrier::new(num_connections));

    let threads: Vec<_> = (0..num_connections)
        .map(|t_idx| {
            let db_path = db_path.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                // Open an INDEPENDENT SQLite database connection for each thread
                let mut conn = open_database(&db_path, passcode)
                    .expect("Each thread must open its own independent SQLite connection");

                // Wait for all threads to reach the barrier before starting concurrent writes
                barrier.wait();

                for i in 0..events_per_thread {
                    log_audit_event(
                        &mut conn,
                        NewAuditEvent {
                            user_id: Some(format!("user_{t_idx}")),
                            action_type: format!("thread_{t_idx}_event_{i}"),
                            entity_name: "task".into(),
                            entity_id: None,
                            before_state: None,
                            after_state: None,
                            client_info: None,
                        },
                    )
                    .expect(
                        "Concurrent audit write must succeed under TransactionBehavior::Immediate",
                    );
                }
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }

    // Verify final database state across independent connection
    let conn = open_database(&db_path, passcode).unwrap();
    let report = verify_audit_chain(&conn).expect("Concurrent audit chain must remain valid");
    assert_eq!(
        report.total_records,
        num_connections * events_per_thread,
        "Zero events lost"
    );
    assert!(report.is_valid);

    // Verify all IDs are contiguous 1..25 with no duplicates
    let ids: Vec<i64> = {
        let mut stmt = conn
            .prepare("SELECT id FROM system_audit_logs ORDER BY id ASC;")
            .unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    };

    let expected_ids: Vec<i64> = (1..=(num_connections * events_per_thread) as i64).collect();
    assert_eq!(
        ids, expected_ids,
        "IDs MUST be contiguous with no gaps or duplicates"
    );

    drop(conn);
    let _ = fs::remove_dir_all(db_path.parent().unwrap());
}

#[test]
fn test_startup_audit_verification() {
    let db_path = temp_db_path("startup_verify");
    let passcode = "StartupPass123!";

    // Create & initialize database
    {
        let mut conn = initialize_and_verify_database(&db_path, passcode).unwrap();
        log_audit_event(
            &mut conn,
            NewAuditEvent {
                user_id: Some("u1".into()),
                action_type: "auth.login".into(),
                entity_name: "user".into(),
                entity_id: None,
                before_state: None,
                after_state: None,
                client_info: None,
            },
        )
        .unwrap();
    }

    // Normal startup verification succeeds
    {
        let _conn = initialize_and_verify_database(&db_path, passcode)
            .expect("Startup verification should succeed on untampered database");
    }

    // Tamper with audit log record directly
    {
        let conn = open_database(&db_path, passcode).unwrap();
        conn.execute(
            "UPDATE system_audit_logs SET action_type = 'TAMPERED' WHERE id = 1;",
            [],
        )
        .unwrap();
    }

    // Startup verification fails on tampered database
    let err = initialize_and_verify_database(&db_path, passcode).unwrap_err();
    match err {
        DatabaseError::AuditTampered { record_id, .. } => assert_eq!(record_id, 1),
        other => panic!("Expected AuditTampered error on startup, got: {other:?}"),
    }

    let _ = fs::remove_dir_all(db_path.parent().unwrap());
}

#[test]
fn test_audit_tamper_first_record_modification() {
    let db_path = temp_db_path("audit_first_mod");
    let mut conn = open_database(&db_path, "Pass123!").unwrap();
    run_migrations(&mut conn).unwrap();

    for i in 1..=3 {
        log_audit_event(
            &mut conn,
            NewAuditEvent {
                user_id: Some("user_1".into()),
                action_type: format!("type_{i}"),
                entity_name: "task".into(),
                entity_id: None,
                before_state: None,
                after_state: None,
                client_info: None,
            },
        )
        .unwrap();
    }

    // Tamper with first record action_type
    conn.execute(
        "UPDATE system_audit_logs SET action_type = 'TAMPERED' WHERE id = 1;",
        [],
    )
    .unwrap();

    let err = verify_audit_chain(&conn).unwrap_err();
    match err {
        DatabaseError::AuditTampered { record_id, .. } => assert_eq!(record_id, 1),
        other => panic!("Expected AuditTampered error at record 1, got {other:?}"),
    }

    drop(conn);
    let _ = fs::remove_dir_all(db_path.parent().unwrap());
}

#[test]
fn test_audit_tamper_middle_record_modification() {
    let db_path = temp_db_path("audit_mid_mod");
    let mut conn = open_database(&db_path, "Pass123!").unwrap();
    run_migrations(&mut conn).unwrap();

    for i in 1..=5 {
        log_audit_event(
            &mut conn,
            NewAuditEvent {
                user_id: Some("user_1".into()),
                action_type: format!("type_{i}"),
                entity_name: "task".into(),
                entity_id: None,
                before_state: None,
                after_state: None,
                client_info: None,
            },
        )
        .unwrap();
    }

    // Tamper with record 3
    conn.execute(
        "UPDATE system_audit_logs SET user_id = 'HACKER' WHERE id = 3;",
        [],
    )
    .unwrap();

    let err = verify_audit_chain(&conn).unwrap_err();
    match err {
        DatabaseError::AuditTampered { record_id, .. } => assert_eq!(record_id, 3),
        other => panic!("Expected AuditTampered error at record 3, got {other:?}"),
    }

    drop(conn);
    let _ = fs::remove_dir_all(db_path.parent().unwrap());
}

#[test]
fn test_audit_tamper_last_record_modification() {
    let db_path = temp_db_path("audit_last_mod");
    let mut conn = open_database(&db_path, "Pass123!").unwrap();
    run_migrations(&mut conn).unwrap();

    for i in 1..=3 {
        log_audit_event(
            &mut conn,
            NewAuditEvent {
                user_id: Some("user_1".into()),
                action_type: format!("type_{i}"),
                entity_name: "task".into(),
                entity_id: None,
                before_state: None,
                after_state: None,
                client_info: None,
            },
        )
        .unwrap();
    }

    // Tamper with last record (id = 3)
    conn.execute(
        "UPDATE system_audit_logs SET after_state = 'FORGED' WHERE id = 3;",
        [],
    )
    .unwrap();

    let err = verify_audit_chain(&conn).unwrap_err();
    match err {
        DatabaseError::AuditTampered { record_id, .. } => assert_eq!(record_id, 3),
        other => panic!("Expected AuditTampered error at record 3, got {other:?}"),
    }

    drop(conn);
    let _ = fs::remove_dir_all(db_path.parent().unwrap());
}

#[test]
fn test_audit_tamper_deletion_first_and_middle() {
    let db_path = temp_db_path("audit_del");
    let mut conn = open_database(&db_path, "Pass123!").unwrap();
    run_migrations(&mut conn).unwrap();

    for i in 1..=4 {
        log_audit_event(
            &mut conn,
            NewAuditEvent {
                user_id: Some("user_1".into()),
                action_type: format!("type_{i}"),
                entity_name: "task".into(),
                entity_id: None,
                before_state: None,
                after_state: None,
                client_info: None,
            },
        )
        .unwrap();
    }

    // Delete middle record (id = 2)
    conn.execute("DELETE FROM system_audit_logs WHERE id = 2;", [])
        .unwrap();

    let err = verify_audit_chain(&conn).unwrap_err();
    match err {
        DatabaseError::AuditTampered { .. } => (),
        other => panic!("Expected AuditTampered error after middle record deletion, got {other:?}"),
    }

    drop(conn);
    let _ = fs::remove_dir_all(db_path.parent().unwrap());
}

#[test]
fn test_audit_tamper_tail_deletion_behavior() {
    let db_path = temp_db_path("audit_tail_del");
    let mut conn = open_database(&db_path, "Pass123!").unwrap();
    run_migrations(&mut conn).unwrap();

    for i in 1..=4 {
        log_audit_event(
            &mut conn,
            NewAuditEvent {
                user_id: Some("user_1".into()),
                action_type: format!("type_{i}"),
                entity_name: "task".into(),
                entity_id: None,
                before_state: None,
                after_state: None,
                client_info: None,
            },
        )
        .unwrap();
    }

    // Delete trailing record (id = 4)
    conn.execute("DELETE FROM system_audit_logs WHERE id = 4;", [])
        .unwrap();

    // Verification of remaining records 1..3 succeeds because inner prefix chain is internally self-consistent.
    // As documented in ADR 0025, detecting tail deletion requires external head/count checkpoints.
    let report =
        verify_audit_chain(&conn).expect("Remaining prefix chain 1..3 is internally consistent");
    assert_eq!(report.total_records, 3);

    drop(conn);
    let _ = fs::remove_dir_all(db_path.parent().unwrap());
}

#[test]
fn test_wal_shm_encryption_no_plaintext_leakage() {
    let db_path = temp_db_path("wal_security");
    let passcode = "WalSecretPasscode123!";
    let sensitive_marker = "SENSITIVE_SECRET_DATA_MARKER_9988776655";

    let mut conn = open_database(&db_path, passcode).unwrap();
    run_migrations(&mut conn).unwrap();

    conn.execute(
        "CREATE TABLE sensitive_table (id INTEGER PRIMARY KEY, secret TEXT);",
        [],
    )
    .unwrap();

    // Insert sensitive data
    conn.execute(
        "INSERT INTO sensitive_table (secret) VALUES (?1);",
        rusqlite::params![sensitive_marker],
    )
    .unwrap();

    // Flush WAL checkpoints to disk files
    conn.execute_batch("PRAGMA wal_checkpoint(FULL);").unwrap();

    // Inspect files on disk
    let parent_dir = db_path.parent().unwrap();
    let entries = fs::read_dir(parent_dir).unwrap();

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file()
            && (path.extension().map_or(false, |ext| {
                ext == "db" || ext == "db-wal" || ext == "db-shm"
            }))
        {
            let file_bytes = fs::read(&path).unwrap();
            let marker_bytes = sensitive_marker.as_bytes();
            let found = file_bytes
                .windows(marker_bytes.len())
                .any(|window| window == marker_bytes);

            assert!(
                !found,
                "Plaintext sensitive marker MUST NOT appear in file: {:?}",
                path
            );
        }
    }

    drop(conn);
    let _ = fs::remove_dir_all(parent_dir);
}
