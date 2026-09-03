pub struct AuditLogInput<'a> {
    pub user_id: Option<&'a str>,
    pub action_type: &'a str,
    pub entity_name: &'a str,
    pub entity_id: Option<&'a str>,
    pub before_state: Option<&'a str>,
    pub after_state: Option<&'a str>,
    pub client_ip: Option<&'a str>,
}

use crate::db::errors::{DbError, Result};
use rusqlite::Connection;
use sha2::{Digest, Sha256};

pub const GENESIS_PREVIOUS_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

pub struct AuditEntry {
    pub id: i64,
    pub user_id: Option<String>,
    pub action_type: String,
    pub entity_name: String,
    pub entity_id: Option<String>,
    pub before_state: Option<String>,
    pub after_state: Option<String>,
    pub client_ip: Option<String>,
    pub previous_log_hash: String,
    pub signature: String,
    pub logged_at: String,
}

/// Computes the SHA-256 signature for an audit record combined with the previous record's signature.
/// Signature formula (ADR 0025):
/// Hash_n = SHA256(id_n + timestamp_n + action_n + prev_hash_{n-1} + after_state_hash_n)
pub fn compute_audit_signature(
    id: i64,
    timestamp: &str,
    action_type: &str,
    previous_log_hash: &str,
    after_state: Option<&str>,
) -> String {
    let after_state_hash = match after_state {
        Some(state) => {
            let mut hasher = Sha256::new();
            hasher.update(state.as_bytes());
            hex::encode(hasher.finalize())
        }
        None => "".to_string(),
    };

    let payload = format!(
        "{}{}{}{}{}",
        id, timestamp, action_type, previous_log_hash, after_state_hash
    );

    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    hex::encode(hasher.finalize())
}

/// Appends a new cryptographically signed audit log record to system_audit_logs.
pub fn append_audit_log(conn: &Connection, input: AuditLogInput) -> Result<i64> {
    // 1. Fetch the signature of the last audit entry to use as previous_log_hash
    let mut stmt =
        conn.prepare("SELECT signature FROM system_audit_logs ORDER BY id DESC LIMIT 1;")?;
    let prev_hash: String = stmt
        .query_row([], |row| row.get(0))
        .unwrap_or_else(|_| GENESIS_PREVIOUS_HASH.to_string());

    // 2. Insert row placeholder to obtain auto-increment ID and default ISO timestamp
    let tx = conn.unchecked_transaction()?;

    tx.execute(
        "INSERT INTO system_audit_logs (
            user_id, action_type, entity_name, entity_id,
            before_state, after_state, client_ip, previous_log_hash, signature
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'PENDING');",
        rusqlite::params![
            input.user_id,
            input.action_type,
            input.entity_name,
            input.entity_id,
            input.before_state,
            input.after_state,
            input.client_ip,
            prev_hash
        ],
    )?;

    let id = tx.last_insert_rowid();

    // 3. Fetch auto-generated timestamp
    let timestamp: String = tx.query_row(
        "SELECT logged_at FROM system_audit_logs WHERE id = ?1;",
        [id],
        |row| row.get(0),
    )?;

    // 4. Compute cryptographic signature
    let signature = compute_audit_signature(
        id,
        &timestamp,
        input.action_type,
        &prev_hash,
        input.after_state,
    );

    // 5. Update signature
    tx.execute(
        "UPDATE system_audit_logs SET signature = ?1 WHERE id = ?2;",
        rusqlite::params![signature, id],
    )?;

    tx.commit()?;

    Ok(id)
}

/// Verifies the entire audit log hash chain from genesis to head.
/// Returns Ok(count) if valid, or DbError::AuditTampering if tampered.
pub fn verify_audit_log_chain(conn: &Connection) -> Result<usize> {
    let mut stmt = conn.prepare(
        "SELECT id, user_id, action_type, entity_name, entity_id,
                before_state, after_state, client_ip, previous_log_hash,
                signature, logged_at
         FROM system_audit_logs ORDER BY id ASC;",
    )?;

    let entries = stmt
        .query_map([], |row| {
            Ok(AuditEntry {
                id: row.get(0)?,
                user_id: row.get(1)?,
                action_type: row.get(2)?,
                entity_name: row.get(3)?,
                entity_id: row.get(4)?,
                before_state: row.get(5)?,
                after_state: row.get(6)?,
                client_ip: row.get(7)?,
                previous_log_hash: row.get(8)?,
                signature: row.get(9)?,
                logged_at: row.get(10)?,
            })
        })?
        .collect::<std::result::Result<Vec<AuditEntry>, _>>()?;

    let mut expected_prev_hash = GENESIS_PREVIOUS_HASH.to_string();

    for entry in entries.iter() {
        // 1. Check previous hash linking
        if entry.previous_log_hash != expected_prev_hash {
            return Err(DbError::AuditTampering(format!(
                "Previous hash mismatch at ID {}. Expected {}, got {}",
                entry.id, expected_prev_hash, entry.previous_log_hash
            )));
        }

        // 2. Verify signature calculation
        let computed_sig = compute_audit_signature(
            entry.id,
            &entry.logged_at,
            &entry.action_type,
            &entry.previous_log_hash,
            entry.after_state.as_deref(),
        );

        if entry.signature != computed_sig {
            return Err(DbError::AuditTampering(format!(
                "Signature mismatch at audit log ID {}. Stored {}, computed {}",
                entry.id, entry.signature, computed_sig
            )));
        }

        expected_prev_hash = entry.signature.clone();
    }

    Ok(entries.len())
}
