use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::identity::errors::IdentityError;

/// Canonical local user profile entity stored in SQLCipher database.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserProfile {
    pub id: String,
    pub email: Option<String>,
    pub display_name: String,
    pub timezone: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Input parameters for creating a new local user profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProfileInput {
    pub display_name: String,
    pub email: Option<String>,
    pub timezone: Option<String>,
}

/// Input parameters for updating an existing local user profile.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateProfileInput {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub timezone: Option<String>,
}

impl UserProfile {
    /// Validate user profile data invariants (display name non-empty, valid UUID v4 ID).
    pub fn validate(&self) -> Result<(), IdentityError> {
        if self.display_name.trim().is_empty() {
            return Err(IdentityError::InvalidInput(
                "Display name cannot be empty".to_string(),
            ));
        }

        let parsed_uuid = Uuid::parse_str(&self.id).map_err(|_| {
            IdentityError::InvalidInput(format!("User ID '{}' is not a valid UUID", self.id))
        })?;

        if parsed_uuid.get_version() != Some(uuid::Version::Random) {
            return Err(IdentityError::InvalidInput(format!(
                "User ID '{}' must be a UUID v4",
                self.id
            )));
        }

        Ok(())
    }
}
