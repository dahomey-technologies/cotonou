use crate::{app_state::*, authentication_service::*, health_check_service::*};
use axum::{
    middleware,
    routing::{get, put},
    Router,
};
use cotonou_common::authentication::{jwt_auth_middleware, JwtSecret};
use error::Error;
use std::net::SocketAddr;

mod app_state;
mod authentication_service;
mod error;
mod health_check_service;

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Starting cotonou-auth...");

    let app_state = AppState::new().await?;
    let jwt_secret = JwtSecret::new("secret");

    println!("cotonou-auth started!");

    // build our application with a route
    let app = Router::new()
        .route("/healthcheck", get(health_check))
        .route(
            "/authentication",
            put(keep_alive)
                .route_layer(middleware::from_fn_with_state(
                    jwt_secret,
                    jwt_auth_middleware,
                ))
                .post(authenticate),
        )
        .with_state(app_state);

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    Ok(axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?)
}
