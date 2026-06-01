//! Per-request tenant schema resolution. A `TenantRegistry` hands out (and
//! caches) one `DatabaseConnection` per tenant schema, each opened with its
//! `search_path` pinned to that schema. This mirrors the legacy pool-per-schema
//! design (see ADR 0009) and reuses the same `set_schema_search_path` mechanism
//! as tenant provisioning.

use std::collections::HashMap;
use std::sync::Arc;

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use tokio::sync::RwLock;

/// `true` if `name` is a safe, unquoted PostgreSQL identifier we are willing to
/// use as a schema (lowercase, starts with a letter, ≤ 63 chars). Shared by the
/// provisioner (which interpolates it into DDL) and the registry.
pub fn is_valid_schema_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return false,
    }
    name.len() <= 63
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

#[derive(Debug)]
pub enum TenantError {
    /// `schema` is not a safe schema identifier.
    InvalidSchema,
    /// Opening the tenant connection failed.
    Db(DbErr),
}

impl std::fmt::Display for TenantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSchema => write!(f, "invalid tenant schema"),
            Self::Db(e) => write!(f, "database error: {e}"),
        }
    }
}

impl std::error::Error for TenantError {}

impl From<DbErr> for TenantError {
    fn from(error: DbErr) -> Self {
        Self::Db(error)
    }
}

/// Resolves and caches a `DatabaseConnection` per tenant schema. Cheap to share
/// behind an `Arc`; one registry serves the whole process.
pub struct TenantRegistry {
    database_url: String,
    connections: RwLock<HashMap<String, Arc<DatabaseConnection>>>,
}

impl TenantRegistry {
    pub fn new(database_url: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
            connections: RwLock::new(HashMap::new()),
        }
    }

    /// Returns a connection whose `search_path` targets `schema`, reusing a
    /// cached one when present. The schema name is validated before use; the
    /// registry does not verify that the schema actually exists (a missing
    /// schema surfaces as a query error later).
    pub async fn connection(&self, schema: &str) -> Result<Arc<DatabaseConnection>, TenantError> {
        if !is_valid_schema_name(schema) {
            return Err(TenantError::InvalidSchema);
        }

        // Fast path: already connected.
        if let Some(conn) = self.connections.read().await.get(schema) {
            return Ok(conn.clone());
        }

        // Connect outside the lock so a slow connect never blocks other schemas.
        let mut options = ConnectOptions::new(self.database_url.clone());
        options.set_schema_search_path(schema.to_owned());
        let connection = Arc::new(Database::connect(options).await?);

        // Double-checked insert: if another task won the race, keep theirs and
        // drop ours, so a given schema always yields the same cached `Arc`.
        let mut map = self.connections.write().await;
        let entry = map
            .entry(schema.to_owned())
            .or_insert_with(|| connection.clone());
        Ok(entry.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::is_valid_schema_name;

    #[test]
    fn accepts_safe_identifiers() {
        assert!(is_valid_schema_name("acme"));
        assert!(is_valid_schema_name("acme_corp_2"));
    }

    #[test]
    fn rejects_unsafe_identifiers() {
        assert!(!is_valid_schema_name(""));
        assert!(!is_valid_schema_name("Acme")); // uppercase
        assert!(!is_valid_schema_name("1acme")); // leading digit
        assert!(!is_valid_schema_name("ac me")); // space
        assert!(!is_valid_schema_name("ac\"me")); // quote
        assert!(!is_valid_schema_name(&"a".repeat(64))); // too long
    }
}
