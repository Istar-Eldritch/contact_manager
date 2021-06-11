use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {

    pub type Keycloak;

    // METHODS

    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Keycloak;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "init", catch)]
    pub async fn init(this: &Keycloak) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "login", catch)]
    pub async fn login(this: &Keycloak) -> Result<(), JsValue>;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "logout", catch)]
    pub async fn logout(this: &Keycloak) -> Result<(), JsValue>;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "register", catch)]
    pub async fn register(this: &Keycloak) -> Result<(), JsValue>;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "loadUserProfile", catch)]
    pub async fn load_user_profile(this: &Keycloak) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "isTokenExpired")]
    pub fn is_token_expired(this: &Keycloak, min_validity: u32) -> bool;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "isTokenExpired", catch)]
    pub async fn update_token(this: &Keycloak, min_validity: u32) -> Result<JsValue, JsValue>;

    // CALLBACKS
    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "onReady", setter)]
    pub fn on_ready(this: &Keycloak, f: &Closure<dyn Fn()>);

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "onAuthSuccess", setter)]
    pub fn on_auth_success(this: &Keycloak, f: &Closure<dyn Fn() -> ()>);

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "onAuthError", setter)]
    pub fn on_auth_error(this: &Keycloak, f: &Closure<dyn Fn() -> ()>);

    #[wasm_bindgen(
        method,
        js_class = "Keycloak",
        js_name = "onAuthRefreshSuccess",
        setter
    )]
    pub fn on_auth_refresh_success(this: &Keycloak, f: &Closure<dyn Fn()>);

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "onAuthRefreshError", setter)]
    pub fn on_auth_refresh_error(this: &Keycloak, f: &Closure<dyn Fn()>);

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "onAuthLogout", setter)]
    pub fn on_auth_logout(this: &Keycloak, f: &Closure<dyn Fn()>);

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "onTokenExpired", setter)]
    pub fn on_token_expired(this: &Keycloak, f: &Closure<dyn Fn()>);

    // PROPERTIES

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "authenticated", getter)]
    pub fn authenticated(this: &Keycloak) -> bool;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "token", getter)]
    pub fn token(this: &Keycloak) -> Option<String>;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "tokenParsed", getter)]
    pub fn token_parsed(this: &Keycloak) -> JsValue;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "subject", getter)]
    pub fn subject(this: &Keycloak) -> Option<String>;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "idToken", getter)]
    pub fn id_token(this: &Keycloak) -> Option<String>;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "idTokenParsed", getter)]
    pub fn id_token_parsed(this: &Keycloak) -> JsValue;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "realmAccess", getter)]
    pub fn realm_access(this: &Keycloak) -> JsValue;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "resourceAccess", getter)]
    pub fn resource_access(this: &Keycloak) -> JsValue;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "refreshToken", getter)]
    pub fn refresh_token(this: &Keycloak) -> Option<String>;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "refreshTokenParsed", getter)]
    pub fn refresh_token_parsed(this: &Keycloak) -> JsValue;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "timeSkew", getter)]
    pub fn time_skew(this: &Keycloak) -> u32;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "response_mode", getter)]
    pub fn response_mode(this: &Keycloak) -> Option<String>;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "flow", getter)]
    pub fn flow(this: &Keycloak) -> String;

    #[wasm_bindgen(method, js_class = "Keycloak", js_name = "responseType", getter)]
    pub fn response_type(this: &Keycloak) -> String;
}
