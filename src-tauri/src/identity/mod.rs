pub mod errors;
pub mod model;

#[cfg(test)]
pub mod tests;

use chrono::Utc;
use rusqlite::Connection;
use uuid::Uuid;

pub use errors::IdentityError;
pub use model::{CreateProfileInput, UpdateProfileInput, UserProfile};

use crate::db::{log_audit_event_tx, NewAuditEvent};

/// Create a new canonical local user profile record in SQLCipher database.
/// Executes user_profiles table creation AND USER_PROFILE_CREATE audit event insertion
/// inside ONE single immediate SQLite transaction.
pub fn create_user_profile(
    conn: &mut Connection,
    input: CreateProfileInput,
) -> Result<UserProfile, IdentityError> {
    if get_local_user_profile(conn).is_ok() {
        return Err(IdentityError::AlreadyExists(
            "Local user profile already exists".to_string(),
        ));
    }

    let now = Utc::now().to_rfc3339();
    let profile = UserProfile {
        id: Uuid::new_v4().to_string(),
        email: input.email,
        display_name: input.display_name,
        timezone: input.timezone.unwrap_or_else(|| "UTC".to_string()),
        created_at: now.clone(),
        updated_at: now,
    };

    profile.validate()?;

    let json_after = serde_json::to_string(&profile)?;

    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;

    tx.execute(
        "INSERT INTO user_profiles (id, email, display_name, timezone, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
        rusqlite::params![
            profile.id,
            profile.email,
            profile.display_name,
            profile.timezone,
            profile.created_at,
            profile.updated_at,
        ],
    )?;

    log_audit_event_tx(
        &tx,
        NewAuditEvent {
            user_id: Some(profile.id.clone()),
            action_type: "USER_PROFILE_CREATE".to_string(),
            entity_name: "user_profile".to_string(),
            entity_id: Some(profile.id.clone()),
            before_state: None,
            after_state: Some(json_after),
            client_info: Some("desktop_app".to_string()),
        },
    )?;

    tx.commit()?;

    Ok(profile)
}

/// Retrieve a user profile by ID.
pub fn get_user_profile(conn: &Connection, id: &str) -> Result<UserProfile, IdentityError> {
    let mut stmt = conn.prepare(
        "SELECT id, email, display_name, timezone, created_at, updated_at
         FROM user_profiles WHERE id = ?1;",
    )?;

    stmt.query_row(rusqlite::params![id], |row| {
        Ok(UserProfile {
            id: row.get(0)?,
            email: row.get(1)?,
            display_name: row.get(2)?,
            timezone: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => IdentityError::NotFound(id.to_string()),
        other => IdentityError::Sqlite(other),
    })
}

/// Retrieve the single local user profile.
pub fn get_local_user_profile(conn: &Connection) -> Result<UserProfile, IdentityError> {
    let mut stmt = conn.prepare(
        "SELECT id, email, display_name, timezone, created_at, updated_at
         FROM user_profiles ORDER BY created_at ASC LIMIT 1;",
    )?;

    stmt.query_row([], |row| {
        Ok(UserProfile {
            id: row.get(0)?,
            email: row.get(1)?,
            display_name: row.get(2)?,
            timezone: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => IdentityError::NotFound("local_user".to_string()),
        other => IdentityError::Sqlite(other),
    })
}

/// Update an existing user profile record.
/// Enforces immutable primary key ID and executes user_profiles table mutation AND USER_PROFILE_UPDATE audit event
/// inside ONE single immediate SQLite transaction.
pub fn update_user_profile(
    conn: &mut Connection,
    user_id: &str,
    input: UpdateProfileInput,
) -> Result<UserProfile, IdentityError> {
    let existing_profile = get_user_profile(conn, user_id)?;

    if existing_profile.id != user_id {
        return Err(IdentityError::IdMismatch {
            expected: existing_profile.id,
            found: user_id.to_string(),
        });
    }

    let json_before = serde_json::to_string(&existing_profile)?;

    let mut updated_profile = existing_profile;

    if let Some(dn) = input.display_name {
        if dn.trim().is_empty() {
            return Err(IdentityError::InvalidInput(
                "Display name cannot be empty".to_string(),
            ));
        }
        updated_profile.display_name = dn;
    }

    if let Some(em) = input.email {
        updated_profile.email = Some(em);
    }

    if let Some(tz) = input.timezone {
        updated_profile.timezone = tz;
    }

    updated_profile.updated_at = Utc::now().to_rfc3339();
    updated_profile.validate()?;

    let json_after = serde_json::to_string(&updated_profile)?;

    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;

    tx.execute(
        "UPDATE user_profiles SET
            email = ?1,
            display_name = ?2,
            timezone = ?3,
            updated_at = ?4
         WHERE id = ?5;",
        rusqlite::params![
            updated_profile.email,
            updated_profile.display_name,
            updated_profile.timezone,
            updated_profile.updated_at,
            user_id,
        ],
    )?;

    log_audit_event_tx(
        &tx,
        NewAuditEvent {
            user_id: Some(user_id.to_string()),
            action_type: "USER_PROFILE_UPDATE".to_string(),
            entity_name: "user_profile".to_string(),
            entity_id: Some(user_id.to_string()),
            before_state: Some(json_before),
            after_state: Some(json_after),
            client_info: Some("desktop_app".to_string()),
        },
    )?;

    tx.commit()?;

    Ok(updated_profile)
}
