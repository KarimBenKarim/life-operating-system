use std::collections::BTreeMap;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::vault::errors::VaultError;

/// Frontmatter metadata schema for vault Markdown documents.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultDocumentMetadata {
    pub id: String,
    pub title: String,
    pub schema_version: u32,
    pub created_at: String,
    pub updated_at: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_yaml::Value>,
}

/// In-memory structure representing a Markdown vault document with YAML frontmatter and body.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultDocument {
    pub metadata: VaultDocumentMetadata,
    pub body: String,
}

impl VaultDocument {
    /// Create a new document with generated UUID v4 stable ID and current RFC3339 timestamps.
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            metadata: VaultDocumentMetadata {
                id: Uuid::new_v4().to_string(),
                title: title.into(),
                schema_version: 1,
                created_at: now.clone(),
                updated_at: now,
                extra: BTreeMap::new(),
            },
            body: body.into(),
        }
    }

    /// Serialize document into UTF-8 Markdown text with YAML frontmatter.
    pub fn to_markdown_string(&self) -> Result<String, VaultError> {
        let yaml_str = serde_yaml::to_string(&self.metadata)?;
        Ok(format!("---\n{yaml_str}---\n{}", self.body))
    }

    /// Parse UTF-8 Markdown text containing YAML frontmatter.
    pub fn parse(raw_content: &str, file_identifier: &str) -> Result<Self, VaultError> {
        let content = raw_content.trim_start_matches('\u{feff}');

        if !content.starts_with("---") {
            return Err(VaultError::MalformedFrontmatter(
                file_identifier.to_string(),
                "Missing opening frontmatter delimiter '---'".to_string(),
            ));
        }

        let rest = &content[3..];
        let rest_trimmed = rest.trim_start_matches(['\r', '\n']);

        let delimiter_pos = rest_trimmed
            .find("\n---")
            .or_else(|| rest_trimmed.find("\r\n---"));

        let (yaml_block, body_part) = match delimiter_pos {
            Some(idx) => {
                let yaml = &rest_trimmed[..idx];
                let after_yaml = &rest_trimmed[idx..];
                let closing_pos = after_yaml.find("---").unwrap() + 3;
                let body = after_yaml[closing_pos..].trim_start_matches(['\r', '\n']);
                (yaml, body)
            }
            None => {
                return Err(VaultError::MalformedFrontmatter(
                    file_identifier.to_string(),
                    "Missing closing frontmatter delimiter '---'".to_string(),
                ));
            }
        };

        let metadata: VaultDocumentMetadata = serde_yaml::from_str(yaml_block).map_err(|e| {
            VaultError::MalformedFrontmatter(file_identifier.to_string(), e.to_string())
        })?;

        Ok(Self {
            metadata,
            body: body_part.to_string(),
        })
    }
}
