use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::{open_database, run_migrations, verify_audit_chain};
use crate::identity::{
    create_user_profile, get_local_user_profile, get_user_profile, update_user_profile,
    CreateProfileInput, IdentityError, UpdateProfileInput,
};

fn temp_db_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("life_os_identity_tests_{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir.join(format!("{name}.db"))
}

#[test]
fn test_migration_v3_execution_and_schema() {
    let db_path = temp_db_path("migration_v3");
    let passcode = "IdentityTestPass123!";

    let mut conn = open_database(&db_path, passcode).unwrap();
    let applied = run_migrations(&mut conn).expect("Migrations V1, V2, V3 must apply cleanly");
    assert_eq!(applied, 3, "Exactly 3 contiguous migrations applied");

    let count: i64 = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='user_profiles';",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "user_profiles table must exist");

    drop(conn);
    let _ = fs::remove_dir_all(db_path.parent().unwrap());
}

#[test]
fn test_user_profile_crud_and_audit() {
    let db_path = temp_db_path("profile_crud");
    let passcode = "IdentityTestPass123!";

    let mut conn = open_database(&db_path, passcode).unwrap();
    run_migrations(&mut conn).unwrap();

    // 1. Create User Profile
    let input = CreateProfileInput {
        display_name: "Alice Nomad".to_string(),
        email: Some("alice@example.com".to_string()),
        timezone: Some("America/New_York".to_string()),
    };

    let profile = create_user_profile(&mut conn, input).unwrap();
    let user_id = profile.id.clone();
    assert_eq!(profile.display_name, "Alice Nomad");
    assert_eq!(profile.email.as_deref(), Some("alice@example.com"));
    assert_eq!(profile.timezone, "America/New_York");

    // Attempting to create a second local profile is rejected
    let input2 = CreateProfileInput {
        display_name: "Bob Fraud".to_string(),
        email: None,
        timezone: None,
    };
    let res_duplicate = create_user_profile(&mut conn, input2);
    assert!(res_duplicate.is_err());
    match res_duplicate.unwrap_err() {
        IdentityError::AlreadyExists(_) => {}
        err => panic!("Expected AlreadyExists error, got: {err:?}"),
    }

    // 2. Retrieve Profile by ID and via get_local_user_profile
    let fetched = get_user_profile(&conn, &user_id).unwrap();
    assert_eq!(fetched, profile);

    let local_fetched = get_local_user_profile(&conn).unwrap();
    assert_eq!(local_fetched, profile);

    // 3. Update Profile
    let update_input = UpdateProfileInput {
        display_name: Some("Alice Nomad Updated".to_string()),
        email: Some("alice.new@example.com".to_string()),
        timezone: Some("UTC".to_string()),
    };

    let updated = update_user_profile(&mut conn, &user_id, update_input).unwrap();
    assert_eq!(updated.id, user_id, "User ID MUST remain immutable");
    assert_eq!(updated.display_name, "Alice Nomad Updated");
    assert_eq!(updated.email.as_deref(), Some("alice.new@example.com"));
    assert_eq!(updated.timezone, "UTC");

    // 4. Verify Audit Log Chain
    let report = verify_audit_chain(&conn).expect("Audit log chain must be valid");
    assert_eq!(report.total_records, 2);
    assert!(report.is_valid);

    // Inspect audit entries
    {
        let mut stmt = conn
            .prepare(
                "SELECT action_type, before_state, after_state FROM system_audit_logs ORDER BY id ASC;",
            )
            .unwrap();
        let logs: Vec<(String, Option<String>, Option<String>)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(logs[0].0, "USER_PROFILE_CREATE");
        assert!(logs[0].1.is_none());
        assert!(logs[0].2.as_ref().unwrap().contains("Alice Nomad"));

        assert_eq!(logs[1].0, "USER_PROFILE_UPDATE");
        assert!(logs[1].1.as_ref().unwrap().contains("Alice Nomad"));
        assert!(logs[1].2.as_ref().unwrap().contains("Alice Nomad Updated"));

        for log in &logs {
            let state_str = format!("{:?}{:?}", log.1, log.2);
            assert!(!state_str.contains("passcode"));
            assert!(!state_str.contains("password"));
            assert!(!state_str.contains("secret"));
            assert!(!state_str.contains("token"));
        }
    }

    drop(conn);
    let _ = fs::remove_dir_all(db_path.parent().unwrap());
}

#[test]
fn test_immutable_user_id_invariant() {
    let db_path = temp_db_path("immutable_id");
    let passcode = "IdentityTestPass123!";

    let mut conn = open_database(&db_path, passcode).unwrap();
    run_migrations(&mut conn).unwrap();

    let profile = create_user_profile(
        &mut conn,
        CreateProfileInput {
            display_name: "Original User".to_string(),
            email: None,
            timezone: None,
        },
    )
    .unwrap();

    let original_id = profile.id.clone();

    let res_mismatch = update_user_profile(
        &mut conn,
        "non-existent-uuid-1234",
        UpdateProfileInput {
            display_name: Some("Hacked User".to_string()),
            ..Default::default()
        },
    );
    assert!(res_mismatch.is_err());
    match res_mismatch.unwrap_err() {
        IdentityError::NotFound(_) | IdentityError::Sqlite(_) => {}
        err => panic!("Expected NotFound error for non-existent ID, got: {err:?}"),
    }

    let current = get_local_user_profile(&conn).unwrap();
    assert_eq!(current.id, original_id);
    assert_eq!(current.display_name, "Original User");

    drop(conn);
    let _ = fs::remove_dir_all(db_path.parent().unwrap());
}

#[test]
fn test_database_error_propagation_and_transaction_rollback() {
    let db_path = temp_db_path("profile_rollback");
    let passcode = "IdentityTestPass123!";

    let mut conn = open_database(&db_path, passcode).unwrap();
    run_migrations(&mut conn).unwrap();

    conn.execute_batch("DROP TABLE system_audit_logs;").unwrap();

    let res = create_user_profile(
        &mut conn,
        CreateProfileInput {
            display_name: "Rollback User".to_string(),
            email: None,
            timezone: None,
        },
    );
    assert!(
        res.is_err(),
        "Creation MUST fail if audit event insertion fails"
    );

    {
        let mut stmt = conn.prepare("SELECT count(*) FROM user_profiles;").unwrap();
        let count: i64 = stmt.query_row([], |r| r.get(0)).unwrap();
        assert_eq!(
            count, 0,
            "Profile metadata MUST NOT remain committed after audit failure"
        );
    }

    drop(conn);
    let _ = fs::remove_dir_all(db_path.parent().unwrap());
}
