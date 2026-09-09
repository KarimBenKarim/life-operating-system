use rusqlite::Connection;
use serde_json::json;

use crate::db::{log_audit_event, NewAuditEvent};
use crate::vault::document::VaultDocument;
use crate::vault::errors::VaultError;

/// Audit logger integration connecting Vault mutations to Task 13's cryptographic audit trail.
pub struct VaultAudit;

impl VaultAudit {
    /// Log document creation event to the audit trail (excluding raw document body).
    pub fn log_create(
        conn: &mut Connection,
        doc: &VaultDocument,
        rel_path: &str,
        file_size_bytes: u64,
        content_hash: &str,
    ) -> Result<(), VaultError> {
        let after_state = json!({
            "relative_path": rel_path,
            "title": doc.frontmatter.title,
            "file_size_bytes": file_size_bytes,
            "content_hash": content_hash,
        })
        .to_string();

        log_audit_event(
            conn,
            NewAuditEvent {
                user_id: None,
                action_type: "vault.document_create".to_string(),
                entity_name: "vault_document".to_string(),
                entity_id: Some(doc.frontmatter.id.clone()),
                before_state: None,
                after_state: Some(after_state),
                client_info: Some("local_vault".to_string()),
            },
        )?;

        Ok(())
    }

    /// Log document update event to the audit trail (excluding raw document body).
    pub fn log_update(
        conn: &mut Connection,
        doc: &VaultDocument,
        rel_path: &str,
        file_size_bytes: u64,
        old_hash: &str,
        new_hash: &str,
    ) -> Result<(), VaultError> {
        let before_state = json!({
            "relative_path": rel_path,
            "content_hash": old_hash,
        })
        .to_string();

        let after_state = json!({
            "relative_path": rel_path,
            "title": doc.frontmatter.title,
            "file_size_bytes": file_size_bytes,
            "content_hash": new_hash,
        })
        .to_string();

        log_audit_event(
            conn,
            NewAuditEvent {
                user_id: None,
                action_type: "vault.document_update".to_string(),
                entity_name: "vault_document".to_string(),
                entity_id: Some(doc.frontmatter.id.clone()),
                before_state: Some(before_state),
                after_state: Some(after_state),
                client_info: Some("local_vault".to_string()),
            },
        )?;

        Ok(())
    }

    /// Log document deletion event to the audit trail (excluding raw document body).
    pub fn log_delete(
        conn: &mut Connection,
        doc_id: &str,
        rel_path: &str,
        old_hash: Option<&str>,
    ) -> Result<(), VaultError> {
        let before_state = json!({
            "relative_path": rel_path,
            "content_hash": old_hash,
        })
        .to_string();

        log_audit_event(
            conn,
            NewAuditEvent {
                user_id: None,
                action_type: "vault.document_delete".to_string(),
                entity_name: "vault_document".to_string(),
                entity_id: Some(doc_id.to_string()),
                before_state: Some(before_state),
                after_state: None,
                client_info: Some("local_vault".to_string()),
            },
        )?;

        Ok(())
    }
}
