mod binding;

use std::collections::HashMap;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen::{prelude::Closure, JsValue};

#[derive(Serialize)]
pub struct KeycloakConfig {
    pub url: String,
    pub realm: String,
    #[serde(rename = "clientId")]
    pub client_id: String,
}

pub struct Keycloak {
    binding: binding::Keycloak,
}

pub struct CallbackHandle {
    _closure: Closure<dyn Fn()>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RealmAccess {
    roles: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedToken {
    exp: u32,
    iat: u32,
    auth_time: u32,
    jti: Uuid,
    iss: String,
    aud: String,
    sub: Uuid,
    typ: String,
    azp: String,
    nonce: String,
    session_state: Uuid,
    acr: String,
    #[serde(rename = "allowed-origins")]
    allowed_origins: Vec<String>,
    realm_access: RealmAccess,
    resource_access: HashMap<String, RealmAccess>,
    scope: String,
    email_verified: bool,
    name: Option<String>,
    preferred_username: String,
    given_name: Option<String>,
    family_name: Option<String>,
    email: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedIdToken {
    exp: u32,
    iat: u32,
    auth_time: u32,
    jti: Uuid,
    iss: String,
    aud: String,
    sub: Uuid,
    typ: String,
    azp: String,
    nonce: String,
    session_state: Uuid,
    at_hash: String,
    acr: String,
    email_verified: bool,
    name: Option<String>,
    preferred_username: String,
    given_name: Option<String>,
    family_name: Option<String>,
    email: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedRefreshToken {
    exp: u32,
    iat: u32,
    jti: Uuid,
    iss: String,
    aud: String,
    sub: Uuid,
    typ: String,
    azp: String,
    nonce: String,
    session_state: Uuid,
    scope: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile<A> {
    pub username: String,
    pub email_verified: bool,
    pub attributes: A,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
}

impl Keycloak {
    pub fn new(config: KeycloakConfig) -> Self {
        Self {
            binding: binding::Keycloak::new(JsValue::from_serde(&config).unwrap()),
        }
    }

    pub async fn init(&self) -> Result<bool, JsValue> {
        let value = self.binding.init().await?;
        value.as_bool().ok_or(JsValue::null())
    }

    pub async fn login(&self) -> Result<(), JsValue> {
        self.binding.login().await
    }

    pub async fn logout(&self) -> Result<(), JsValue> {
        self.binding.logout().await
    }

    pub async fn register(&self) -> Result<(), JsValue> {
        self.binding.register().await
    }

    pub async fn load_user_profile<A: DeserializeOwned>(&self) -> Result<UserProfile<A>, JsValue> {
        let value = self.binding.load_user_profile().await?;
        Ok(JsValue::into_serde(&value).unwrap())
    }

    pub fn is_token_expired(&self, min_validity: u32) -> bool {
        self.binding.is_token_expired(min_validity)
    }

    pub async fn update_token(&self, min_validity: u32) -> Result<bool, JsValue> {
        let value = self.binding.update_token(min_validity).await?;
        value.as_bool().ok_or(JsValue::null())
    }

    // CALLBACKS

    pub fn on_ready(&self, f: Closure<dyn Fn()>) -> CallbackHandle {
        self.binding.on_ready(&f);
        CallbackHandle { _closure: f }
    }

    pub fn on_auth_success(&self, f: Closure<dyn Fn()>) -> CallbackHandle {
        self.binding.on_auth_success(&f);
        CallbackHandle { _closure: f }
    }

    pub fn on_auth_error(&self, f: Closure<dyn Fn()>) -> CallbackHandle {
        self.binding.on_auth_error(&f);
        CallbackHandle { _closure: f }
    }

    pub fn on_auth_refresh_success(&self, f: Closure<dyn Fn()>) -> CallbackHandle {
        self.binding.on_auth_success(&f);
        CallbackHandle { _closure: f }
    }

    pub fn on_auth_refresh_error(&self, f: Closure<dyn Fn()>) -> CallbackHandle {
        self.binding.on_auth_success(&f);
        CallbackHandle { _closure: f }
    }

    pub fn on_auth_logout(&self, f: Closure<dyn Fn()>) -> CallbackHandle {
        self.binding.on_auth_logout(&f);
        CallbackHandle { _closure: f }
    }

    pub fn on_token_expired(&self, f: Closure<dyn Fn()>) -> CallbackHandle {
        self.binding.on_token_expired(&f);
        CallbackHandle { _closure: f }
    }

    // GETTERS

    pub fn authenticated(&self) -> bool {
        self.binding.authenticated()
    }

    pub fn token(&self) -> Option<String> {
        self.binding.token()
    }

    pub fn token_parsed(&self) -> Option<ParsedToken> {
        let val = self.binding.token_parsed();
        JsValue::into_serde(&val).unwrap()
    }

    pub fn subject(&self) -> Option<Uuid> {
        self.binding.subject().map(|s| Uuid::parse_str(&s).unwrap())
    }

    pub fn id_token(&self) -> Option<String> {
        self.binding.id_token()
    }

    pub fn id_token_parsed(&self) -> Option<ParsedIdToken> {
        let val = self.binding.id_token_parsed();
        JsValue::into_serde(&val).unwrap()
    }

    pub fn realm_access(&self) -> Option<RealmAccess> {
        JsValue::into_serde(&self.binding.realm_access()).unwrap()
    }

    pub fn resource_access(&self) -> Option<HashMap<String, RealmAccess>> {
        JsValue::into_serde(&self.binding.resource_access()).unwrap()
    }

    pub fn refresh_token(&self) -> Option<String> {
        self.binding.refresh_token()
    }

    pub fn refresh_token_parsed(&self) -> Option<ParsedRefreshToken> {
        let val = self.binding.refresh_token_parsed();
        JsValue::into_serde(&val).unwrap()
    }

    pub fn time_skew(&self) -> u32 {
        self.binding.time_skew()
    }

    pub fn response_mode(&self) -> Option<String> {
        self.binding.response_mode()
    }

    pub fn flow(&self) -> String {
        self.binding.flow()
    }

    pub fn response_type(&self) -> String {
        self.binding.response_type()
    }
}
