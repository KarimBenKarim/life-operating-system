pub mod audit;
pub mod connection;
pub mod crypto;
pub mod errors;
pub mod migrations;

#[cfg(test)]
pub mod tests;

pub use audit::{log_audit_event, verify_audit_chain, AuditLogEntry, NewAuditEvent};
pub use connection::{
    get_kdf_metadata_path, initialize_and_verify_database, open_database, KdfMetadata,
};
pub use crypto::{derive_key, generate_salt, DerivedKey};
pub use errors::DatabaseError;
pub use migrations::run_migrations;
