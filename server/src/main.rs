#![doc(
    html_logo_url = "https://bitbucket-assetroot.s3.amazonaws.com/c/photos/2020/Mar/24/193508354-1-cloudapi-logo_avatar.png",
    html_favicon_url = "https://bitbucket-assetroot.s3.amazonaws.com/c/photos/2020/Mar/24/193508354-1-cloudapi-logo_avatar.png"
)]
#![deny(broken_intra_doc_links)]

mod keycloak;
mod state;

use common::jsonrpc::{JSONRPCError, JSONRPCSuccess};
use keycloak::RequestActor;
use sqlx::PgPool;
use state::State;
use std::convert::TryInto;
use tide::{
    http::headers::HeaderValue,
    security::{CorsMiddleware, Origin},
    Request,
};

use crate::keycloak::{auth_middleware, JWKS};

#[async_std::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init_timed();

    let postgres_url = std::env::var_os("POSTGRES_URL")
        .expect("The environment variable POSTGRES_URL is mandatory")
        .into_string()
        .expect("POSTGRES_URL is not a valid string");
    let port = std::env::var("PORT").unwrap_or_else(|_| "8083".into());
    let auth_server_url =
        std::env::var("AUTH_SERVER_URL").unwrap_or(String::from("http://localhost:8081"));

    let auth_keys: JWKS = surf::get(&format!(
        "{}/auth/realms/demo/protocol/openid-connect/certs",
        auth_server_url
    ))
    .recv_json()
    .await?;

    log::debug!("Loaded keys: {:?}", auth_keys);

    let postgres = PgPool::connect(&postgres_url)
        .await
        .expect("Error creating postgres pool");

    log::debug!("Postgres pool created");

    let state = State {
        postgres,
        auth_keys,
    };

    log::debug!("Database migrations completed");

    log::info!("Using port {}", port);

    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    let mut app = tide::with_state(state);
    app.with(cors);
    app.with(auth_middleware);

    app.at("/").get(handler);

    app.listen(format!("0.0.0.0:{}", port)).await?;
    Ok(())
}

// pub async fn inner_handler(req: Request<State>) -> tide::Result {}

pub async fn handler(req: Request<State>) -> tide::Result {
    req.ext::<RequestActor>()
        .unwrap()
        .as_ref()
        .ok_or_else(|| {
            log::error!("Error retrieving user from request");
            JSONRPCError::forbidden(())
        })
        .map(|actor| JSONRPCSuccess::ok(actor).try_into().unwrap())
        .or_else(|e| Ok(e.into()))
}
