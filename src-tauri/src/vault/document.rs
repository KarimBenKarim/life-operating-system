use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::vault::errors::VaultError;

/// Document metadata block stored in YAML frontmatter at the head of every Markdown file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Frontmatter {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub schema_version: u32,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Frontmatter {
    pub fn new(title: impl Into<String>) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            created_at: now.clone(),
            updated_at: now,
            schema_version: 1,
            extra: serde_json::Map::new(),
        }
    }
}

/// In-memory representation of a Life OS Markdown document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultDocument {
    pub frontmatter: Frontmatter,
    pub body: String,
}

impl VaultDocument {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            frontmatter: Frontmatter::new(title),
            body: body.into(),
        }
    }

    /// Parse a raw UTF-8 Markdown file into a structured VaultDocument.
    pub fn parse(content: &str) -> Result<Self, VaultError> {
        let trimmed = content.trim_start_matches('\u{feff}'); // Strip UTF-8 BOM if present

        if !trimmed.starts_with("---") {
            // Missing frontmatter: construct default frontmatter and treat entire content as body
            let default_fm = Frontmatter::new("Untitled");
            return Ok(Self {
                frontmatter: default_fm,
                body: trimmed.to_string(),
            });
        }

        // Find closing '---'
        let rest = &trimmed[3..];
        let end_idx = rest.find("\n---").or_else(|| rest.find("\r\n---"));

        let end_pos = match end_idx {
            Some(idx) => idx,
            None => {
                return Err(VaultError::InvalidFrontmatter(
                    "Unclosed YAML frontmatter block: missing closing '---'".to_string(),
                ));
            }
        };

        let yaml_str = &rest[..end_pos];
        let body_start = if rest[end_pos..].starts_with("\r\n---") {
            end_pos + 5
        } else {
            end_pos + 4
        };

        let body = rest[body_start..]
            .trim_start_matches("\r\n")
            .trim_start_matches('\n')
            .to_string();

        // Parse YAML frontmatter into serde_json::Value to extract fields and unknown metadata
        let yaml_val: serde_json::Value = serde_yaml::from_str(yaml_str).map_err(|e| {
            VaultError::InvalidFrontmatter(format!("Failed to parse YAML frontmatter: {e}"))
        })?;

        let mut obj = match yaml_val {
            serde_json::Value::Object(map) => map,
            _ => serde_json::Map::new(),
        };

        let now = Utc::now().to_rfc3339();

        let id = obj
            .remove("id")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let title = obj
            .remove("title")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "Untitled".to_string());

        let created_at = obj
            .remove("created_at")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| now.clone());

        let updated_at = obj
            .remove("updated_at")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or(now);

        let schema_version = obj
            .remove("schema_version")
            .and_then(|v| v.as_u64().map(|n| n as u32))
            .unwrap_or(1);

        let frontmatter = Frontmatter {
            id,
            title,
            created_at,
            updated_at,
            schema_version,
            extra: obj, // Preserves all unknown/additional frontmatter fields
        };

        Ok(Self { frontmatter, body })
    }

    /// Serialize the VaultDocument to UTF-8 Markdown text with YAML frontmatter.
    pub fn to_markdown(&self) -> Result<String, VaultError> {
        let yaml_str = serde_yaml::to_string(&self.frontmatter).map_err(|e| {
            VaultError::SerializationError(format!("Failed to serialize frontmatter to YAML: {e}"))
        })?;

        Ok(format!("---\n{}\n---\n{}", yaml_str.trim(), self.body))
    }
}
