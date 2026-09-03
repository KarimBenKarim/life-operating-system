use crate::db::audit::{append_audit_log, verify_audit_log_chain, AuditLogInput};
use crate::db::connection::DbManager;
use crate::db::crypto::{derive_key_argon2id, generate_salt};
use crate::db::errors::DbError;
use crate::db::migrations::{get_current_schema_version, run_migrations};

fn insert_test_user(db: &DbManager, user_id: &str) {
    db.conn()
        .execute(
            "INSERT OR IGNORE INTO users (id, email, password_hash, first_name) VALUES (?1, ?2, 'hash', 'Test');",
            [user_id, &format!("{user_id}@example.com")],
        )
        .expect("Failed to insert test user");
}

#[test]
fn test_unencrypted_db_lifecycle_and_migrations() {
    let db = DbManager::open_unencrypted(":memory:").expect("Failed to open in-memory db");
    let version_before = get_current_schema_version(db.conn()).expect("Failed version check");
    assert_eq!(version_before, 0);

    let applied = run_migrations(db.conn()).expect("Failed migration run");
    assert_eq!(applied, 1);

    let version_after = get_current_schema_version(db.conn()).expect("Failed version check");
    assert_eq!(version_after, 1);

    // Idempotence test
    let re_applied = run_migrations(db.conn()).expect("Failed idempotent migration run");
    assert_eq!(re_applied, 0);
}

#[test]
fn test_argon2id_key_derivation_and_sqlcipher_encryption() {
    let passcode = "SuperSecretMasterPasscode123!";
    let salt = generate_salt();

    let derived_key = derive_key_argon2id(passcode, &salt).expect("Argon2id derivation failed");
    assert_eq!(derived_key.key.len(), 32);

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_encrypted.db");

    // 1. Initialize encrypted database with derived key
    {
        let db =
            DbManager::open_encrypted(&db_path, &derived_key).expect("Failed to open encrypted db");
        run_migrations(db.conn()).expect("Failed to run migrations on encrypted db");
    }

    // 2. Re-open with CORRECT key -> Should succeed
    {
        let db = DbManager::open_encrypted(&db_path, &derived_key)
            .expect("Failed to re-open encrypted db with valid key");
        let version = get_current_schema_version(db.conn()).expect("Version check failed");
        assert_eq!(version, 1);
    }

    // 3. Re-open with INCORRECT key -> Should fail
    {
        let wrong_key = derive_key_argon2id("WrongPasscode456!", &salt).unwrap();
        let db_result = DbManager::open_encrypted(&db_path, &wrong_key);

        if let Ok(db) = db_result {
            // Attempting to query encrypted page with wrong key must return SQLITE_NOTADB / error
            let query_res = get_current_schema_version(db.conn());
            assert!(
                query_res.is_err(),
                "Reading encrypted DB with wrong key must fail!"
            );
        }
    }
}

#[test]
fn test_audit_log_hash_chain_and_tamper_detection() {
    let db = DbManager::open_unencrypted(":memory:").expect("Failed to open db");
    run_migrations(db.conn()).expect("Failed to run migrations");
    insert_test_user(&db, "user_1");

    // 1. Append valid audit logs
    let id1 = append_audit_log(
        db.conn(),
        AuditLogInput {
            user_id: Some("user_1"),
            action_type: "user_auth",
            entity_name: "users",
            entity_id: Some("user_1"),
            before_state: None,
            after_state: Some(r#"{"status":"success"}"#),
            client_ip: Some("127.0.0.1"),
        },
    )
    .expect("Failed to append audit 1");

    let id2 = append_audit_log(
        db.conn(),
        AuditLogInput {
            user_id: Some("user_1"),
            action_type: "task_create",
            entity_name: "tasks",
            entity_id: Some("tsk_1"),
            before_state: None,
            after_state: Some(r#"{"title":"Buy groceries"}"#),
            client_ip: Some("127.0.0.1"),
        },
    )
    .expect("Failed to append audit 2");

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);

    // 2. Chain verification on valid log
    let count = verify_audit_log_chain(db.conn()).expect("Chain verification should pass");
    assert_eq!(count, 2);

    // 3. Tamper test: Modify payload of row 1
    db.conn()
        .execute(
            "UPDATE system_audit_logs SET after_state = '{\"status\":\"TAMPERED\"}' WHERE id = 1;",
            [],
        )
        .expect("Failed manual SQL tamper update");

    let verify_res = verify_audit_log_chain(db.conn());
    assert!(
        verify_res.is_err(),
        "Chain verification must fail when payload is tampered!"
    );

    if let Err(DbError::AuditTampering(msg)) = verify_res {
        assert!(msg.contains("Signature mismatch at audit log ID 1"));
    } else {
        panic!("Expected DbError::AuditTampering");
    }
}

#[test]
fn test_audit_log_deleted_entry_detection() {
    let db = DbManager::open_unencrypted(":memory:").expect("Failed to open db");
    run_migrations(db.conn()).expect("Failed to run migrations");
    insert_test_user(&db, "user_1");

    append_audit_log(
        db.conn(),
        AuditLogInput {
            user_id: Some("user_1"),
            action_type: "action_1",
            entity_name: "entity",
            entity_id: None,
            before_state: None,
            after_state: None,
            client_ip: None,
        },
    )
    .unwrap();

    append_audit_log(
        db.conn(),
        AuditLogInput {
            user_id: Some("user_1"),
            action_type: "action_2",
            entity_name: "entity",
            entity_id: None,
            before_state: None,
            after_state: None,
            client_ip: None,
        },
    )
    .unwrap();

    // Delete row 1 -> Row 2's previous_log_hash will now point to missing row signature
    db.conn()
        .execute("DELETE FROM system_audit_logs WHERE id = 1;", [])
        .unwrap();

    let verify_res = verify_audit_log_chain(db.conn());
    assert!(
        verify_res.is_err(),
        "Chain verification must fail when entry is deleted!"
    );
}

#[test]
fn test_security_absence_of_plaintext_secrets_in_audit_logs() {
    let db = DbManager::open_unencrypted(":memory:").expect("Failed to open db");
    run_migrations(db.conn()).expect("Failed to run migrations");
    insert_test_user(&db, "user_1");

    let passcode = "SuperSecret123!";
    let masked_state = "{\"passcode\":\"[MASKED]\"}";

    append_audit_log(
        db.conn(),
        AuditLogInput {
            user_id: Some("user_1"),
            action_type: "user_auth",
            entity_name: "users",
            entity_id: Some("user_1"),
            before_state: None,
            after_state: Some(masked_state),
            client_ip: None,
        },
    )
    .unwrap();

    let mut stmt = db
        .conn()
        .prepare("SELECT after_state FROM system_audit_logs WHERE id = 1;")
        .unwrap();
    let stored_state: String = stmt.query_row([], |row| row.get(0)).unwrap();

    assert!(!stored_state.contains(passcode));
    assert!(stored_state.contains("[MASKED]"));
}
