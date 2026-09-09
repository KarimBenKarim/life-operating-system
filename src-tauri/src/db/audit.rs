use chrono::Utc;
use rusqlite::{Connection, TransactionBehavior};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Write;

use crate::db::errors::DatabaseError;

/// Genesis hash value for the very first audit log row (64 zero hex characters).
pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Audit log record stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditLogEntry {
    pub id: i64,
    pub user_id: Option<String>,
    pub action_type: String,
    pub entity_name: String,
    pub entity_id: Option<String>,
    pub before_state: Option<String>,
    pub after_state: Option<String>,
    pub client_info: Option<String>,
    pub created_at: String,
    pub previous_hash: String,
    pub hash: String,
}

/// Event parameters used to insert a new audit log entry.
#[derive(Debug, Clone)]
pub struct NewAuditEvent {
    pub user_id: Option<String>,
    pub action_type: String,
    pub entity_name: String,
    pub entity_id: Option<String>,
    pub before_state: Option<String>,
    pub after_state: Option<String>,
    pub client_info: Option<String>,
}

/// Verification report summary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditVerificationReport {
    pub total_records: usize,
    pub is_valid: bool,
}

/// Encode a mandatory string field with a deterministic tagged length prefix (`S:<len>:<data>;`).
fn encode_string_field(buf: &mut String, val: &str) {
    let _ = write!(buf, "S:{}:{val};", val.len());
}

/// Encode an optional string field with tagged markers (`N;` for None, `S:<len>:<data>;` for Some).
/// Unambiguously distinguishes `None` from `Some("")`.
fn encode_option_field(buf: &mut String, val: Option<&str>) {
    match val {
        Some(v) => {
            let _ = write!(buf, "S:{}:{v};", v.len());
        }
        None => buf.push_str("N;"),
    }
}

/// Compute canonical SHA-256 hash using tagged length-prefixed encoding.
/// Unambiguously distinguishes `None` from `Some("")` and prevents boundary collisions.
#[allow(clippy::too_many_arguments)]
pub fn compute_audit_hash(
    id: i64,
    created_at: &str,
    user_id: Option<&str>,
    action_type: &str,
    entity_name: &str,
    entity_id: Option<&str>,
    before_state: Option<&str>,
    after_state: Option<&str>,
    client_info: Option<&str>,
    previous_hash: &str,
) -> String {
    let mut canonical = String::with_capacity(512);

    let id_str = id.to_string();
    encode_string_field(&mut canonical, &id_str);
    encode_string_field(&mut canonical, created_at);
    encode_option_field(&mut canonical, user_id);
    encode_string_field(&mut canonical, action_type);
    encode_string_field(&mut canonical, entity_name);
    encode_option_field(&mut canonical, entity_id);
    encode_option_field(&mut canonical, before_state);
    encode_option_field(&mut canonical, after_state);
    encode_option_field(&mut canonical, client_info);
    encode_string_field(&mut canonical, previous_hash);

    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    hex::encode(hasher.finalize())
}

/// Insert a new cryptographic audit log entry chained to the previous record's hash.
/// Uses an immediate transaction (`TransactionBehavior::Immediate`) to acquire an immediate
/// database write lock, guaranteeing atomicity and sequence integrity under concurrent write attempts.
pub fn log_audit_event(
    conn: &mut Connection,
    event: NewAuditEvent,
) -> Result<AuditLogEntry, DatabaseError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;

    let mut stmt =
        tx.prepare("SELECT id, hash FROM system_audit_logs ORDER BY id DESC LIMIT 1;")?;

    let last_record: Option<(i64, String)> = stmt
        .query_row([], |row| Ok((row.get(0)?, row.get(1)?)))
        .ok();
    drop(stmt);

    let (next_id, previous_hash) = match last_record {
        Some((id, hash)) => (id + 1, hash),
        None => (1, GENESIS_HASH.to_string()),
    };

    let created_at = Utc::now().to_rfc3339();

    let current_hash = compute_audit_hash(
        next_id,
        &created_at,
        event.user_id.as_deref(),
        &event.action_type,
        &event.entity_name,
        event.entity_id.as_deref(),
        event.before_state.as_deref(),
        event.after_state.as_deref(),
        event.client_info.as_deref(),
        &previous_hash,
    );

    tx.execute(
        "INSERT INTO system_audit_logs (
            id, user_id, action_type, entity_name, entity_id,
            before_state, after_state, client_info, created_at, previous_hash, hash
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11);",
        rusqlite::params![
            next_id,
            event.user_id,
            event.action_type,
            event.entity_name,
            event.entity_id,
            event.before_state,
            event.after_state,
            event.client_info,
            created_at,
            previous_hash,
            current_hash
        ],
    )?;

    tx.commit()?;

    Ok(AuditLogEntry {
        id: next_id,
        user_id: event.user_id,
        action_type: event.action_type,
        entity_name: event.entity_name,
        entity_id: event.entity_id,
        before_state: event.before_state,
        after_state: event.after_state,
        client_info: event.client_info,
        created_at,
        previous_hash,
        hash: current_hash,
    })
}

/// Verify the entire audit log hash chain sequentially.
pub fn verify_audit_chain(conn: &Connection) -> Result<AuditVerificationReport, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT id, user_id, action_type, entity_name, entity_id,
                before_state, after_state, client_info, created_at, previous_hash, hash
         FROM system_audit_logs ORDER BY id ASC;",
    )?;

    let entries: Vec<AuditLogEntry> = stmt
        .query_map([], |row| {
            Ok(AuditLogEntry {
                id: row.get(0)?,
                user_id: row.get(1)?,
                action_type: row.get(2)?,
                entity_name: row.get(3)?,
                entity_id: row.get(4)?,
                before_state: row.get(5)?,
                after_state: row.get(6)?,
                client_info: row.get(7)?,
                created_at: row.get(8)?,
                previous_hash: row.get(9)?,
                hash: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    if entries.is_empty() {
        return Ok(AuditVerificationReport {
            total_records: 0,
            is_valid: true,
        });
    }

    let mut expected_previous_hash = GENESIS_HASH.to_string();

    for (idx, entry) in entries.iter().enumerate() {
        let expected_id = (idx as i64) + 1;

        // Check ID sequence
        if entry.id != expected_id {
            return Err(DatabaseError::AuditTampered {
                record_id: entry.id,
                reason: format!(
                    "ID sequence gap detected at index {idx}: expected ID {expected_id}, got {}",
                    entry.id
                ),
            });
        }

        // Check previous hash linking
        if entry.previous_hash != expected_previous_hash {
            return Err(DatabaseError::AuditTampered {
                record_id: entry.id,
                reason: format!(
                    "Previous hash mismatch at ID {}: expected {expected_previous_hash}, got {}",
                    entry.id, entry.previous_hash
                ),
            });
        }

        // Check recalculation of record hash
        let computed_hash = compute_audit_hash(
            entry.id,
            &entry.created_at,
            entry.user_id.as_deref(),
            &entry.action_type,
            &entry.entity_name,
            entry.entity_id.as_deref(),
            entry.before_state.as_deref(),
            entry.after_state.as_deref(),
            entry.client_info.as_deref(),
            &entry.previous_hash,
        );

        if entry.hash != computed_hash {
            return Err(DatabaseError::AuditTampered {
                record_id: entry.id,
                reason: format!(
                    "Content hash mismatch at ID {}: expected {computed_hash}, stored {}",
                    entry.id, entry.hash
                ),
            });
        }

        expected_previous_hash = entry.hash.clone();
    }

    Ok(AuditVerificationReport {
        total_records: entries.len(),
        is_valid: true,
    })
}
