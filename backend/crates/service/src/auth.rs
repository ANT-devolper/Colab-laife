//! Native authentication. Verifies login credentials against the `public`
//! schema and issues/validates stateless **JWT** session tokens (HS256). The
//! token is the only session state — there is no server-side session store
//! (see ADR 0008).

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::password::verify_password;
use entity::{organization, user};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

/// Default session lifetime. Tokens are stateless, so this is the only expiry.
pub const DEFAULT_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// Claims carried by a session JWT. `exp` is validated on decode; the rest let
/// downstream middleware resolve the tenant and authorize the request without a
/// database round-trip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — the user id.
    pub sub: String,
    /// The id of the organization the user belongs to.
    pub org: String,
    /// The tenant schema slug (the organization name).
    pub schema: String,
    /// Whether the user is an organization admin.
    pub is_admin: bool,
    /// Expiration, in seconds since the Unix epoch.
    pub exp: usize,
}

impl Claims {
    /// Builds claims for a session expiring `ttl` from now. Takes primitives
    /// (not entities) so it stays trivially unit-testable.
    pub fn new(
        sub: impl Into<String>,
        org: impl Into<String>,
        schema: impl Into<String>,
        is_admin: bool,
        ttl: Duration,
    ) -> Self {
        let exp = (SystemTime::now() + ttl)
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before the Unix epoch")
            .as_secs() as usize;
        Self {
            sub: sub.into(),
            org: org.into(),
            schema: schema.into(),
            is_admin,
            exp,
        }
    }
}

/// A successfully authenticated user together with its organization.
pub struct Authenticated {
    pub user: user::Model,
    pub organization: organization::Model,
}

#[derive(Debug)]
pub enum AuthError {
    /// Unknown email or wrong password. The two are deliberately
    /// indistinguishable so callers cannot probe which emails exist.
    InvalidCredentials,
    /// Credentials are valid but the organization is deactivated.
    OrganizationInactive,
    /// A database error.
    Db(DbErr),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCredentials => write!(f, "invalid credentials"),
            Self::OrganizationInactive => write!(f, "organization is inactive"),
            Self::Db(e) => write!(f, "database error: {e}"),
        }
    }
}

impl std::error::Error for AuthError {}

impl From<DbErr> for AuthError {
    fn from(error: DbErr) -> Self {
        Self::Db(error)
    }
}

/// Verifies `email`/`password` against the `public` schema. A missing user, a
/// soft-deleted user and a wrong password all map to `InvalidCredentials` to
/// avoid user enumeration. A valid user in a deactivated organization yields
/// `OrganizationInactive`.
pub async fn authenticate(
    db: &DatabaseConnection,
    email: &str,
    password: &str,
) -> Result<Authenticated, AuthError> {
    let Some(user) = user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .filter(user::Column::Deleted.eq(false))
        .one(db)
        .await?
    else {
        return Err(AuthError::InvalidCredentials);
    };

    if !verify_password(password, &user.password_hash) {
        return Err(AuthError::InvalidCredentials);
    }

    let Some(organization) = organization::Entity::find_by_id(user.organization_id)
        .one(db)
        .await?
    else {
        // A user without its organization is a data-integrity fault; do not
        // leak it as a distinct outcome.
        return Err(AuthError::InvalidCredentials);
    };

    if !organization.is_active {
        return Err(AuthError::OrganizationInactive);
    }

    Ok(Authenticated { user, organization })
}

/// Encodes `claims` into a signed HS256 JWT.
pub fn encode_token(claims: &Claims, secret: &[u8]) -> Result<String, jsonwebtoken::errors::Error> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret),
    )
}

/// Decodes and validates a JWT, returning its claims. Expiry is enforced
/// (HS256, default validation).
pub fn decode_token(token: &str, secret: &[u8]) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn new_copies_fields_and_sets_a_future_expiry() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        let claims = Claims::new("user-1", "org-1", "acme", true, Duration::from_secs(3600));

        assert_eq!(claims.sub, "user-1");
        assert_eq!(claims.org, "org-1");
        assert_eq!(claims.schema, "acme");
        assert!(claims.is_admin);
        assert!(claims.exp >= now + 3600);
    }

    #[test]
    fn encodes_and_decodes_round_trip() {
        let secret = b"test-secret";
        let claims = Claims::new("u", "o", "acme", false, Duration::from_secs(3600));

        let token = encode_token(&claims, secret).expect("encode");
        let decoded = decode_token(&token, secret).expect("decode");

        assert_eq!(decoded, claims);
    }

    #[test]
    fn rejects_a_token_signed_with_a_different_secret() {
        let claims = Claims::new("u", "o", "acme", false, Duration::from_secs(3600));
        let token = encode_token(&claims, b"secret-a").expect("encode");

        assert!(decode_token(&token, b"secret-b").is_err());
    }

    #[test]
    fn rejects_an_expired_token() {
        let secret = b"test-secret";
        let mut claims = Claims::new("u", "o", "acme", false, Duration::from_secs(3600));
        // An hour in the past — beyond the default 60s leeway.
        claims.exp = (SystemTime::now() - Duration::from_secs(3600))
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        let token = encode_token(&claims, secret).expect("encode");

        assert!(decode_token(&token, secret).is_err());
    }
}
