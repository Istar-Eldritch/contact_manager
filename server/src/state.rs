use crate::keycloak::JWKS;
use sqlx::PgPool;

#[derive(Clone)]
pub struct State {
    pub postgres: PgPool,
    pub auth_keys: JWKS,
}
