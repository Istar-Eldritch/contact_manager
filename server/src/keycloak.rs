use std::{future::Future, ops::Deref, pin::Pin};

use crate::state::State;
use common::jsonrpc::JSONRPCError;
use jsonwebtoken::{decode, errors::ErrorKind, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tide::{http::headers::HeaderName, Next, Request, Response};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {}

#[derive(Deserialize, Clone, Debug)]
pub struct JWK {
    kid: String,
    kty: String,
    alg: String, //	"RS256"
    #[serde(rename = "use")]
    _use: String, //	"sig"
    n: String, //	"mgJbG_JUAfh2hxR1Y9mYcEFTcxFvxCKGcyZkaN7maCcdFjij77ZlM_VwRaniC7WPL516Lra1UmxIMikvAzD7dMsnSyvthyr12vK8XKobfVS2cMLSmyJwnKbLKVbXRQqpmjCVcN5buHzz5Sr9XnSN1JQAlpiUrCK98qeFxwAXFCtvrEG449Ehut0k96VgOmH63ipAlxUryHVoinmhoGTRnmAvZaL-ARTsAPC93M-OfhRUC8IvpnXa56lv6bpKbX3-BiFG6LCAh_HGCj_Egm4gDKpijpUQsraylxS7LgKPNustmoCcn_ZQjYV4NhP7JUg_Zdt2_hwy9Yilgx1SvQkM_Q"
    e: String, //	"AQAB"
    x5c: Vec<String>,
    x5t: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct JWKS {
    keys: Vec<JWK>,
}

/// List of errors that can occur when parsing the authorization header.
///
/// Authorization headers must be in the following format:
/// `Authorization: Bearer <JWT>`
#[derive(Debug)]
pub enum AuthErrors {
    /// The authorization header was not provided
    EmptyHeader,

    /// The authorization header is formatted incorrectly or otherwise invalid
    InvalidHeader,

    /// The decoded token data is invalid
    InvalidToken,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct KeycloakRealmAccess {
    roles: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct KeycloakClaims {
    pub acr: String,
    #[serde(rename = "allowed-origins")]
    pub allowed_origins: Vec<String>,
    pub aud: String,
    pub auth_time: u32,
    pub azp: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub exp: u32,
    pub family_name: Option<String>,
    pub given_name: Option<String>,
    pub iat: u32,
    pub iss: String,
    pub jti: Uuid,
    pub name: Option<String>,
    pub nonce: Uuid,
    pub preferred_username: String,
    pub realm_access: KeycloakRealmAccess,
    pub session_state: Uuid,
    pub sub: Uuid,
    pub typ: String,
}

/// Extracts a JWT from a [`tide::Request`] and return a [`User`] struct on success
///
/// The verification uses [JWKS](https://auth0.com/docs/tokens/concepts/jwks) to verify a JWT
/// token that must be present in the `Authorization` header or `auth_token` query parameter.
/// The value in the `Authorization` header will always take precedence over the query
/// parameter.
pub fn from_request(req: &Request<State>) -> Result<KeycloakClaims, AuthErrors> {
    let (token_type, header_token_value) = req
        .header(&"Authorization".parse::<HeaderName>().unwrap())
        .and_then(|h| h.get(0).map(|h| h.as_str()))
        .map(|s| {
            let mut parts = s.split_ascii_whitespace();

            (
                parts.next().map(String::from),
                parts.next().map(String::from),
            )
        })
        .unwrap_or((None, None));

    #[derive(Deserialize, Debug)]
    struct QueryParam {
        access_token: Option<String>,
    }

    // Look for an optional access token in the query params.
    let query_param = req.query().ok().and_then(|q: QueryParam| q.access_token);

    log::debug!(
        "Auth header: {:?} {:?}, auth query param: {:?}",
        token_type,
        header_token_value,
        query_param
    );

    let token = match (token_type, query_param) {
        (Some(token_type), _) if token_type.to_lowercase() == "bearer" => {
            header_token_value.ok_or(AuthErrors::InvalidHeader)
        }
        (Some(_), _) => Err(AuthErrors::InvalidHeader),
        (None, Some(query_param)) => Ok(query_param),
        (None, None) => Err(AuthErrors::EmptyHeader),
    }?;

    log::debug!("Validating token {}", token);

    let key = req.state().auth_keys.keys.get(0).unwrap();
    let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e);
    let validating_algo = Validation::new(Algorithm::RS256);
    let credentials = decode::<KeycloakClaims>(&token, &decoding_key, &validating_algo);
    match credentials {
        Ok(token_data) => {
            log::debug!("Decoded credentials: {:?}", token_data);
            Ok(token_data.claims)
        }
        Err(err) => {
            // Do not report an error unless there is a potential bug.
            // - User errors are logged as debug
            // - Token invalid errors & potential tampering errors are reported as warn.
            // - Configuration errors & other issues that can be resolved through code improvements are reported as error.
            match err.kind() {
                ErrorKind::InvalidToken | ErrorKind::ExpiredSignature => {
                    log::debug!("Token error: {:?}", err)
                }
                ErrorKind::InvalidAudience
                | ErrorKind::InvalidIssuer
                | ErrorKind::InvalidSignature
                | ErrorKind::ImmatureSignature
                | ErrorKind::Base64(_) => log::warn!("{:?} decoding {:?}", err, token),
                _ => log::error!("Token error: {:?}", err),
            };

            Err(AuthErrors::InvalidToken)
        }
    }
}

/// The user who did the request.
///
/// The user may be empty if no auth token was provided to to the request
pub struct RequestActor(Option<KeycloakClaims>);

impl Deref for RequestActor {
    type Target = Option<KeycloakClaims>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

async fn inner_auth_middleware(
    request: &Request<State>,
) -> Result<Option<KeycloakClaims>, JSONRPCError> {
    match from_request(&request) {
        Ok(claims) => Ok(Some(claims)),
        Err(AuthErrors::EmptyHeader) => Ok(None),
        Err(err) => {
            // Could not get the credentials from the token, can be a bug in the client
            // application, the credentials may have expired or someone is trying to tamper with token.
            log::error!("Error retrieving credentials from token: {:?}", err);
            Err(JSONRPCError::unauthorized(()))
        }
    }
}

/// Provides the RequestActor object in the request
pub fn auth_middleware<'a>(
    mut request: Request<State>,
    next: Next<'a, State>,
) -> Pin<Box<dyn Future<Output = tide::Result> + Send + 'a>> {
    Box::pin(async {
        match inner_auth_middleware(&request).await {
            Ok(claims) => {
                request.set_ext(RequestActor(claims));
                Ok(next.run(request).await)
            }
            Err(err) => Ok(Response::from(err)),
        }
    })
}
