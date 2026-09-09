use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::{initialize_and_verify_database, verify_audit_chain};
use crate::vault::document::VaultDocument;
use crate::vault::errors::VaultError;
use crate::vault::path::{RelativePath, VaultRoot};
use crate::vault::MarkdownVault;

fn temp_vault_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("life_os_vault_tests_{name}_{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_vault_create_read_update_delete_lifecycle() {
    let root_dir = temp_vault_dir("lifecycle");
    let vault = MarkdownVault::new(&root_dir).unwrap();

    let rel_path = "notes/hello.md";
    let doc = VaultDocument::new("Hello World", "This is the initial markdown body content.");

    // 1. Create document
    let resolved = vault.create_document(None, rel_path, &doc).unwrap();
    assert!(resolved.as_path().exists());
    assert!(vault.exists(rel_path));

    // 2. Read document
    let read_doc = vault.read_document(rel_path).unwrap();
    assert_eq!(read_doc.frontmatter.title, "Hello World");
    assert_eq!(read_doc.body, "This is the initial markdown body content.");

    // 3. Update document
    let mut updated_doc = read_doc;
    updated_doc.body = "Updated markdown body content.".to_string();
    vault.update_document(None, rel_path, &updated_doc).unwrap();

    let read_updated = vault.read_document(rel_path).unwrap();
    assert_eq!(read_updated.body, "Updated markdown body content.");

    // 4. List documents
    let docs = vault.list_documents().unwrap();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].to_string_lossy(), "notes/hello.md");

    // 5. Delete document
    vault.delete_document(None, rel_path).unwrap();
    assert!(!vault.exists(rel_path));

    let _ = fs::remove_dir_all(&root_dir);
}

#[test]
fn test_vault_duplicate_creation_error() {
    let root_dir = temp_vault_dir("duplicate");
    let vault = MarkdownVault::new(&root_dir).unwrap();
    let rel_path = "dup.md";
    let doc = VaultDocument::new("Title", "Body");

    vault.create_document(None, rel_path, &doc).unwrap();

    let err = vault.create_document(None, rel_path, &doc).unwrap_err();
    match err {
        VaultError::DocumentAlreadyExists(p) => assert_eq!(p, "dup.md"),
        other => panic!("Expected DocumentAlreadyExists, got: {other:?}"),
    }

    let _ = fs::remove_dir_all(&root_dir);
}

#[test]
fn test_vault_missing_document_error() {
    let root_dir = temp_vault_dir("missing");
    let vault = MarkdownVault::new(&root_dir).unwrap();
    let rel_path = "nonexistent.md";

    let err = vault.read_document(rel_path).unwrap_err();
    match err {
        VaultError::DocumentNotFound(p) => assert_eq!(p, "nonexistent.md"),
        other => panic!("Expected DocumentNotFound, got: {other:?}"),
    }

    let _ = fs::remove_dir_all(&root_dir);
}

#[test]
fn test_vault_nested_directory_creation() {
    let root_dir = temp_vault_dir("nested");
    let vault = MarkdownVault::new(&root_dir).unwrap();
    let rel_path = "a/b/c/d/deep_note.md";
    let doc = VaultDocument::new("Deep Note", "Deep content");

    vault.create_document(None, rel_path, &doc).unwrap();
    assert!(vault.exists(rel_path));

    let read_doc = vault.read_document(rel_path).unwrap();
    assert_eq!(read_doc.frontmatter.title, "Deep Note");

    let _ = fs::remove_dir_all(&root_dir);
}

#[test]
fn test_markdown_frontmatter_valid_parsing() {
    let raw = "---
id: custom_uuid_123
title: Valid Frontmatter
created_at: 2026-09-08T12:00:00Z
updated_at: 2026-09-08T12:00:00Z
schema_version: 1
---
Markdown body text here.";

    let doc = VaultDocument::parse(raw).unwrap();
    assert_eq!(doc.frontmatter.id, "custom_uuid_123");
    assert_eq!(doc.frontmatter.title, "Valid Frontmatter");
    assert_eq!(doc.body, "Markdown body text here.");
}

#[test]
fn test_markdown_frontmatter_missing() {
    let raw = "Just raw markdown body without frontmatter headers.";
    let doc = VaultDocument::parse(raw).unwrap();

    assert_eq!(doc.frontmatter.title, "Untitled");
    assert_eq!(doc.frontmatter.schema_version, 1);
    assert_eq!(
        doc.body,
        "Just raw markdown body without frontmatter headers."
    );
}

#[test]
fn test_markdown_frontmatter_malformed_yaml() {
    let raw = "---
title: [unclosed list
---
Body text";

    let err = VaultDocument::parse(raw).unwrap_err();
    match err {
        VaultError::InvalidFrontmatter(_) => (),
        other => panic!("Expected InvalidFrontmatter, got: {other:?}"),
    }
}

#[test]
fn test_markdown_unicode_and_multiline() {
    let doc = VaultDocument::new(
        "Unicode Document 🌟 🚀 汉字 العربي",
        "Line 1: Hello Unicode! 🌍\nLine 2: Multi-line text body.\nLine 3: 12345.",
    );

    let markdown_text = doc.to_markdown().unwrap();
    let parsed = VaultDocument::parse(&markdown_text).unwrap();

    assert_eq!(
        parsed.frontmatter.title,
        "Unicode Document 🌟 🚀 汉字 العربي"
    );
    assert_eq!(
        parsed.body,
        "Line 1: Hello Unicode! 🌍\nLine 2: Multi-line text body.\nLine 3: 12345."
    );
}

#[test]
fn test_markdown_unknown_frontmatter_fields_preserved() {
    let raw = r#"---
id: custom_uuid_999
title: Unknown Fields Document
created_at: 2026-09-08T12:00:00Z
updated_at: 2026-09-08T12:00:00Z
schema_version: 1
custom_tag: "high_priority"
author_name: "Alice"
sub_object:
  key1: "val1"
---
Body text here."#;

    let doc = VaultDocument::parse(raw).unwrap();
    assert_eq!(
        doc.frontmatter
            .extra
            .get("custom_tag")
            .and_then(|v| v.as_str()),
        Some("high_priority")
    );
    assert_eq!(
        doc.frontmatter
            .extra
            .get("author_name")
            .and_then(|v| v.as_str()),
        Some("Alice")
    );

    // Re-serialize and verify extra fields are preserved
    let markdown_text = doc.to_markdown().unwrap();
    let reparsed = VaultDocument::parse(&markdown_text).unwrap();
    assert_eq!(
        reparsed
            .frontmatter
            .extra
            .get("custom_tag")
            .and_then(|v| v.as_str()),
        Some("high_priority")
    );
}

#[test]
fn test_path_safety_parent_directory_traversal() {
    let err = RelativePath::parse("../secret.md").unwrap_err();
    match err {
        VaultError::PathTraversal(msg) => assert!(msg.contains("Parent directory traversal")),
        other => panic!("Expected PathTraversal for '../', got: {other:?}"),
    }

    let err2 = RelativePath::parse("notes/../../secret.md").unwrap_err();
    match err2 {
        VaultError::PathTraversal(msg) => assert!(msg.contains("Parent directory traversal")),
        other => panic!("Expected PathTraversal for '../../', got: {other:?}"),
    }
}

#[test]
fn test_path_safety_absolute_paths() {
    let err_unix = RelativePath::parse("/etc/passwd").unwrap_err();
    match err_unix {
        VaultError::PathTraversal(msg) => assert!(msg.contains("Absolute paths or drive prefixes")),
        other => panic!("Expected PathTraversal for absolute Unix path, got: {other:?}"),
    }

    let err_win = RelativePath::parse("C:\\Windows\\System32\\cmd.exe").unwrap_err();
    match err_win {
        VaultError::PathTraversal(msg) => assert!(msg.contains("Absolute paths or drive prefixes")),
        other => panic!("Expected PathTraversal for Windows drive path, got: {other:?}"),
    }
}

#[test]
fn test_path_safety_windows_backslash_normalization() {
    let rel_path = RelativePath::parse("notes\\journal\\daily.md").unwrap();
    assert_eq!(rel_path.to_string_lossy(), "notes/journal/daily.md");
}

#[test]
fn test_path_safety_symlink_escape_attempt() {
    let root_dir = temp_vault_dir("symlink_root");
    let outside_dir = temp_vault_dir("symlink_outside");

    let secret_file = outside_dir.join("outside_secret.txt");
    fs::write(&secret_file, "SENSITIVE_OUTSIDE_DATA").unwrap();

    let root = VaultRoot::new(&root_dir).unwrap();
    let validator = crate::vault::path::PathValidator::new(root);

    // Create symlink inside vault root pointing outside
    let symlink_path = root_dir.join("symlink_file.md");
    #[cfg(unix)]
    let _ = std::os::unix::fs::symlink(&secret_file, &symlink_path);

    if symlink_path.exists() {
        let rel_path = RelativePath::parse("symlink_file.md").unwrap();
        let err = validator.resolve(&rel_path).unwrap_err();
        match err {
            VaultError::PathTraversal(msg) => assert!(msg.contains("escapes vault root boundary")),
            other => panic!("Expected PathTraversal for symlink escape attempt, got: {other:?}"),
        }
    }

    let _ = fs::remove_dir_all(&root_dir);
    let _ = fs::remove_dir_all(&outside_dir);
}

#[test]
fn test_atomic_write_temp_file_cleanup() {
    let root_dir = temp_vault_dir("temp_cleanup");
    let vault = MarkdownVault::new(&root_dir).unwrap();
    let rel_path = "notes/atomic.md";
    let doc = VaultDocument::new("Atomic Note", "Atomic body content");

    vault.create_document(None, rel_path, &doc).unwrap();

    // Check directory entries: verify no .tmp_* files remain
    let parent = root_dir.join("notes");
    for entry in fs::read_dir(parent).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().to_string();
        assert!(
            !name.starts_with(".tmp_"),
            "Temporary write file .tmp_* MUST be cleaned up on completion"
        );
    }

    let _ = fs::remove_dir_all(&root_dir);
}

#[test]
fn test_vault_sqlite_metadata_index_sync() {
    let root_dir = temp_vault_dir("sqlite_index_vault");
    let db_dir = temp_vault_dir("sqlite_index_db");
    let db_path = db_dir.join("test_vault.db");
    let passcode = "VaultDbPasscode123!";

    let vault = MarkdownVault::new(&root_dir).unwrap();
    let mut conn = initialize_and_verify_database(&db_path, passcode).unwrap();

    let rel_path = "notes/sync_test.md";
    let doc = VaultDocument::new("Sync Title", "Sync Body Content");

    // 1. Create document -> syncs metadata to SQLite
    vault
        .create_document(Some(&mut conn), rel_path, &doc)
        .unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT count(*) FROM vault_documents WHERE relative_path = ?1;",
            rusqlite::params![rel_path],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);

    // 2. Update document -> updates metadata in SQLite
    let mut updated_doc = doc;
    updated_doc.body = "Updated Sync Body Content".to_string();
    vault
        .update_document(Some(&mut conn), rel_path, &updated_doc)
        .unwrap();

    let title: String = conn
        .query_row(
            "SELECT title FROM vault_documents WHERE relative_path = ?1;",
            rusqlite::params![rel_path],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(title, "Sync Title");

    // 3. Delete document -> deletes metadata from SQLite
    vault.delete_document(Some(&mut conn), rel_path).unwrap();

    let count_after_delete: i64 = conn
        .query_row(
            "SELECT count(*) FROM vault_documents WHERE relative_path = ?1;",
            rusqlite::params![rel_path],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count_after_delete, 0);

    let _ = fs::remove_dir_all(&root_dir);
    let _ = fs::remove_dir_all(&db_dir);
}

#[test]
fn test_vault_audit_trail_integration() {
    let root_dir = temp_vault_dir("audit_vault");
    let db_dir = temp_vault_dir("audit_db");
    let db_path = db_dir.join("test_audit_vault.db");
    let passcode = "VaultAuditPass123!";

    let vault = MarkdownVault::new(&root_dir).unwrap();
    let mut conn = initialize_and_verify_database(&db_path, passcode).unwrap();

    let rel_path = "notes/audited_note.md";
    let doc = VaultDocument::new("Audited Title", "SECRET_SENSITIVE_MARKDOWN_BODY_TEXT");

    // 1. Create -> logs audit event
    vault
        .create_document(Some(&mut conn), rel_path, &doc)
        .unwrap();

    // 2. Update -> logs audit event
    let mut updated_doc = doc;
    updated_doc.body = "UPDATED_SECRET_SENSITIVE_MARKDOWN_BODY_TEXT".to_string();
    vault
        .update_document(Some(&mut conn), rel_path, &updated_doc)
        .unwrap();

    // 3. Delete -> logs audit event
    vault.delete_document(Some(&mut conn), rel_path).unwrap();

    // 4. Verify audit chain validity
    let report =
        verify_audit_chain(&conn).expect("Audit chain MUST remain valid after vault mutations");
    assert_eq!(report.total_records, 3);
    assert!(report.is_valid);

    // 5. Verify raw Markdown text is NOT stored in system_audit_logs
    let mut stmt = conn
        .prepare("SELECT before_state, after_state FROM system_audit_logs;")
        .unwrap();

    let logs: Vec<(Option<String>, Option<String>)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    for (before, after) in logs {
        if let Some(b) = before {
            assert!(
                !b.contains("SECRET_SENSITIVE_MARKDOWN_BODY_TEXT"),
                "Raw Markdown text MUST NOT appear in audit log records"
            );
        }
        if let Some(a) = after {
            assert!(
                !a.contains("SECRET_SENSITIVE_MARKDOWN_BODY_TEXT"),
                "Raw Markdown text MUST NOT appear in audit log records"
            );
        }
    }

    let _ = fs::remove_dir_all(&root_dir);
    let _ = fs::remove_dir_all(&db_dir);
}
