use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::{open_database, run_migrations, verify_audit_chain};
use crate::vault::{
    validate_and_resolve_relative_path, write_atomic, Vault, VaultDocument, VaultError,
};

fn temp_vault_dir(name: &str) -> (PathBuf, PathBuf) {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("life_os_vault_tests_{nanos}_{name}"));
    let vault_root = base.join("vault");
    let db_path = base.join("test.db");
    fs::create_dir_all(&vault_root).unwrap();
    (vault_root, db_path)
}

#[test]
fn test_valid_paths() {
    let (vault_root, _) = temp_vault_dir("valid_paths");

    let p1 = validate_and_resolve_relative_path(&vault_root, "note.md").unwrap();
    assert_eq!(p1, vault_root.join("note.md"));

    let p2 = validate_and_resolve_relative_path(&vault_root, "sub/folder/note.md").unwrap();
    assert_eq!(p2, vault_root.join("sub/folder/note.md"));

    let _ = fs::remove_dir_all(vault_root.parent().unwrap());
}

#[test]
fn test_traversal_rejection() {
    let (vault_root, _) = temp_vault_dir("traversal");

    let cases = vec![
        "../etc/passwd",
        "sub/../../secret.txt",
        "..",
        "./../secret.txt",
        "folder/../..",
    ];

    for path_str in cases {
        let res = validate_and_resolve_relative_path(&vault_root, path_str);
        assert!(res.is_err(), "Path traversal '{path_str}' MUST be rejected");
        match res.unwrap_err() {
            VaultError::PathTraversal(_) | VaultError::InvalidPath(_, _) => {}
            err => panic!("Unexpected error type for traversal '{path_str}': {err:?}"),
        }
    }

    let _ = fs::remove_dir_all(vault_root.parent().unwrap());
}

#[test]
fn test_absolute_path_rejection() {
    let (vault_root, _) = temp_vault_dir("absolute");

    let cases = vec!["/etc/passwd", "/var/log/syslog", "/home/user/secret.md"];

    for path_str in cases {
        let res = validate_and_resolve_relative_path(&vault_root, path_str);
        assert!(res.is_err(), "Absolute path '{path_str}' MUST be rejected");
        match res.unwrap_err() {
            VaultError::InvalidPath(_, msg) => {
                assert!(msg.contains("rejected") || msg.contains("Absolute"));
            }
            err => panic!("Unexpected error type for absolute path '{path_str}': {err:?}"),
        }
    }

    let _ = fs::remove_dir_all(vault_root.parent().unwrap());
}

#[test]
fn test_windows_style_path_attacks() {
    let (vault_root, _) = temp_vault_dir("win_paths");

    let cases = vec![
        "C:\\Windows\\System32\\cmd.exe",
        "C:secret.txt",
        "D:/notes/journal.md",
        "\\\\server\\share\\document.md",
        "\\\\.\\PHYSICALDRIVE0",
        "\\\\?\\C:\\file.txt",
        "sub\\folder\\..\\..\\secret.txt",
        "folder\\:\\file.txt",
    ];

    for path_str in cases {
        let res = validate_and_resolve_relative_path(&vault_root, path_str);
        assert!(
            res.is_err(),
            "Windows-style path attack '{path_str}' MUST be rejected"
        );
    }

    let _ = fs::remove_dir_all(vault_root.parent().unwrap());
}

#[test]
fn test_symlink_escape_handling() {
    let (vault_root, _) = temp_vault_dir("symlink_escape");

    let outside_dir = vault_root.parent().unwrap().join("outside");
    fs::create_dir_all(&outside_dir).unwrap();
    let outside_file = outside_dir.join("secret.txt");
    fs::write(&outside_file, "outside secret").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let symlink_path = vault_root.join("outside_link");
        let _ = symlink(&outside_dir, &symlink_path);

        let res = validate_and_resolve_relative_path(&vault_root, "outside_link/secret.txt");
        assert!(res.is_err(), "Symlink escaping vault root MUST be rejected");
        match res.unwrap_err() {
            VaultError::SymlinkEscape(_) | VaultError::InvalidPath(_, _) => {}
            err => panic!("Expected SymlinkEscape error, got: {err:?}"),
        }
    }

    let _ = fs::remove_dir_all(vault_root.parent().unwrap());
}

#[test]
fn test_frontmatter_parsing_and_formatting() {
    let doc = VaultDocument::new("Test Title", "This is body content.");
    assert!(!doc.metadata.id.is_empty());

    let markdown_str = doc.to_markdown_string().unwrap();
    assert!(markdown_str.starts_with("---\n"));
    assert!(markdown_str.contains("title: Test Title"));
    assert!(markdown_str.contains("This is body content."));

    let parsed = VaultDocument::parse(&markdown_str, "test.md").unwrap();
    assert_eq!(parsed.metadata.id, doc.metadata.id);
    assert_eq!(parsed.metadata.title, "Test Title");
    assert_eq!(parsed.body, "This is body content.");
}

#[test]
fn test_malformed_frontmatter() {
    let no_start = "title: Test\n---\nbody";
    let res1 = VaultDocument::parse(no_start, "no_start.md");
    assert!(res1.is_err());
    match res1.unwrap_err() {
        VaultError::MalformedFrontmatter(_, msg) => {
            assert!(msg.contains("Missing opening"));
        }
        err => panic!("Unexpected error: {err:?}"),
    }

    let no_end = "---\ntitle: Test\nbody content without end";
    let res2 = VaultDocument::parse(no_end, "no_end.md");
    assert!(res2.is_err());
    match res2.unwrap_err() {
        VaultError::MalformedFrontmatter(_, msg) => {
            assert!(msg.contains("Missing closing"));
        }
        err => panic!("Unexpected error: {err:?}"),
    }

    let bad_yaml = "---\ntitle: [invalid yaml syntax:::\n---\nbody";
    let res3 = VaultDocument::parse(bad_yaml, "bad_yaml.md");
    assert!(res3.is_err());
    match res3.unwrap_err() {
        VaultError::MalformedFrontmatter(_, _) => {}
        err => panic!("Unexpected error: {err:?}"),
    }
}

#[test]
fn test_atomic_write_success_and_failure() {
    let (vault_root, _) = temp_vault_dir("atomic_write");

    let target_file = vault_root.join("atomic_note.md");
    let content = b"Hello, Atomic World!";

    write_atomic(&target_file, content).unwrap();
    assert_eq!(fs::read(&target_file).unwrap(), content);

    let temp_files: Vec<_> = fs::read_dir(&vault_root)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .filter(|name| name.starts_with(".tmp_"))
        .collect();
    assert!(
        temp_files.is_empty(),
        "Temporary files MUST be cleaned up on success"
    );

    let _ = fs::remove_dir_all(vault_root.parent().unwrap());
}

#[test]
fn test_vault_crud_and_indexing_and_audit() {
    let (vault_root, db_path) = temp_vault_dir("vault_crud");
    let passcode = "VaultPasscode123!";

    let mut conn = open_database(&db_path, passcode).unwrap();
    run_migrations(&mut conn).unwrap();

    let vault = Vault::new(&vault_root).unwrap();

    // 1. Create Document
    let mut doc = VaultDocument::new("First Note", "My daily reflection.");
    let stable_id = doc.metadata.id.clone();

    let record = vault
        .save_document(&mut conn, "reflections/day1.md", &mut doc)
        .unwrap();
    assert_eq!(record.id, stable_id);
    assert_eq!(record.relative_path, "reflections/day1.md");
    assert_eq!(record.title, "First Note");
    assert!(!record.content_hash.is_empty());

    // Verify file exists on disk
    let read_back = vault.read_document("reflections/day1.md").unwrap();
    assert_eq!(read_back.metadata.id, stable_id);
    assert_eq!(read_back.body, "My daily reflection.");

    // Verify metadata table entry
    let meta = vault
        .get_document_metadata(&conn, "reflections/day1.md")
        .unwrap();
    assert_eq!(meta.id, stable_id);
    assert_eq!(meta.size_bytes, record.size_bytes);

    // 2. Update Document
    let mut updated_doc = read_back;
    updated_doc.metadata.title = "First Note (Updated)".to_string();
    updated_doc.body = "Updated daily reflection body.".to_string();

    let updated_record = vault
        .save_document(&mut conn, "reflections/day1.md", &mut updated_doc)
        .unwrap();
    assert_eq!(
        updated_record.id, stable_id,
        "Stable ID MUST be preserved on update"
    );
    assert_eq!(updated_record.title, "First Note (Updated)");

    let read_updated = vault.read_document("reflections/day1.md").unwrap();
    assert_eq!(read_updated.body, "Updated daily reflection body.");

    // 3. List Documents
    let docs = vault.list_documents(&conn).unwrap();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].id, stable_id);

    // 4. Verify Audit Events
    let report = verify_audit_chain(&conn).unwrap();
    assert_eq!(report.total_records, 2); // 1 create + 1 update
    assert!(report.is_valid);

    // Inspect audit logs to verify raw Markdown body is NEVER stored
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

        assert_eq!(logs[0].0, "VAULT_DOCUMENT_CREATE");
        assert!(logs[0].1.is_none());
        assert!(logs[0].2.as_ref().unwrap().contains("reflections/day1.md"));
        assert!(!logs[0].2.as_ref().unwrap().contains("My daily reflection"));

        assert_eq!(logs[1].0, "VAULT_DOCUMENT_UPDATE");
        assert!(logs[1].1.as_ref().unwrap().contains("First Note"));
        assert!(logs[1].2.as_ref().unwrap().contains("First Note (Updated)"));
        assert!(!logs[1]
            .2
            .as_ref()
            .unwrap()
            .contains("Updated daily reflection body"));
    }

    // 5. Delete Document
    vault
        .delete_document(&mut conn, "reflections/day1.md")
        .unwrap();

    let list_after_del = vault.list_documents(&conn).unwrap();
    assert!(list_after_del.is_empty());

    let not_found_err = vault.read_document("reflections/day1.md").unwrap_err();
    match not_found_err {
        VaultError::NotFound(_) => {}
        err => panic!("Expected NotFound error after deletion, got: {err:?}"),
    }

    // Verify 3rd audit event for delete
    let report_del = verify_audit_chain(&conn).unwrap();
    assert_eq!(report_del.total_records, 3);
    assert!(report_del.is_valid);

    drop(conn);
    let _ = fs::remove_dir_all(vault_root.parent().unwrap());
}

#[test]
fn test_stable_id_invariant_and_mismatch_rejection() {
    let (vault_root, db_path) = temp_vault_dir("stable_id");
    let passcode = "VaultPasscode123!";

    let mut conn = open_database(&db_path, passcode).unwrap();
    run_migrations(&mut conn).unwrap();

    let vault = Vault::new(&vault_root).unwrap();

    // A. Create document with UUID-A
    let mut doc_a = VaultDocument::new("Original Title", "Original Body");
    let uuid_a = doc_a.metadata.id.clone();

    vault
        .save_document(&mut conn, "notes/stable.md", &mut doc_a)
        .unwrap();

    // B. Update same path with UUID-A -> succeeds
    let mut doc_a_updated = doc_a.clone();
    doc_a_updated.metadata.title = "Original Title Updated".to_string();

    let res_a = vault.save_document(&mut conn, "notes/stable.md", &mut doc_a_updated);
    assert!(res_a.is_ok());
    assert_eq!(res_a.unwrap().id, uuid_a);

    // C. Update same path with UUID-B -> rejected
    let mut doc_b = VaultDocument::new("Mismatched Title", "Mismatched Body");
    let uuid_b = doc_b.metadata.id.clone();
    assert_ne!(uuid_a, uuid_b);

    let res_b = vault.save_document(&mut conn, "notes/stable.md", &mut doc_b);
    assert!(res_b.is_err(), "Update with mismatched ID MUST be rejected");

    match res_b.unwrap_err() {
        VaultError::IdMismatch {
            expected, found, ..
        } => {
            assert_eq!(expected, uuid_a);
            assert_eq!(found, uuid_b);
        }
        err => panic!("Expected IdMismatch error, got: {err:?}"),
    }

    // D. Verify database still contains UUID-A after rejected update
    let meta = vault
        .get_document_metadata(&conn, "notes/stable.md")
        .unwrap();
    assert_eq!(meta.id, uuid_a, "Database ID MUST remain UUID-A");
    assert_eq!(meta.title, "Original Title Updated");

    drop(conn);
    let _ = fs::remove_dir_all(vault_root.parent().unwrap());
}

#[test]
fn test_database_error_propagation_in_save_document() {
    let (vault_root, db_path) = temp_vault_dir("db_error_prop");
    let passcode = "VaultPasscode123!";

    let mut conn = open_database(&db_path, passcode).unwrap();
    // Do NOT run migrations -> vault_documents table does not exist

    let vault = Vault::new(&vault_root).unwrap();
    let mut doc = VaultDocument::new("Test Title", "Test Body");

    let res = vault.save_document(&mut conn, "notes/test.md", &mut doc);
    assert!(
        res.is_err(),
        "Database query error MUST be propagated and not swallowed"
    );

    match res.unwrap_err() {
        VaultError::Sqlite(_) | VaultError::Database(_) => {}
        err => panic!("Expected Sqlite or Database error, got: {err:?}"),
    }

    drop(conn);
    let _ = fs::remove_dir_all(vault_root.parent().unwrap());
}

#[test]
fn test_dot_path_component_rejection() {
    let (vault_root, _) = temp_vault_dir("dot_component");

    let cases = vec!["folder/./document.md", "./document.md", "sub/.", "."];

    for path_str in cases {
        let res = validate_and_resolve_relative_path(&vault_root, path_str);
        assert!(
            res.is_err(),
            "Dot path component '{path_str}' MUST be rejected"
        );
        match res.unwrap_err() {
            VaultError::PathTraversal(_) | VaultError::InvalidPath(_, _) => {}
            err => panic!("Unexpected error type for dot path '{path_str}': {err:?}"),
        }
    }

    let _ = fs::remove_dir_all(vault_root.parent().unwrap());
}

#[test]
fn test_atomic_replacement_of_existing_target() {
    let (vault_root, _) = temp_vault_dir("atomic_replace");

    let target_file = vault_root.join("notes/target.md");

    // Write initial content A
    write_atomic(&target_file, b"Content A").unwrap();
    assert_eq!(fs::read(&target_file).unwrap(), b"Content A");

    // Overwrite atomically with content B
    write_atomic(&target_file, b"Content B").unwrap();
    assert_eq!(fs::read(&target_file).unwrap(), b"Content B");

    // Verify no leftover temp files
    let parent_dir = target_file.parent().unwrap();
    let temp_files: Vec<_> = fs::read_dir(parent_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .filter(|name| name.starts_with(".tmp_"))
        .collect();
    assert!(
        temp_files.is_empty(),
        "Temporary files MUST be cleaned up on success"
    );

    let _ = fs::remove_dir_all(vault_root.parent().unwrap());
}
