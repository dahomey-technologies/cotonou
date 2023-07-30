use crate::{
    app_state::*, error::*, game_server_service::*, health_check_service::*,
    matchmaking_assembler::*, matchmaking_service::*, matchmaking_started_notification::*,
    profile_for_matchmaking_entity::*, profile_for_matchmaking_manager::*,
};
use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};
use cotonou_common::authentication::{jwt_auth_middleware, JwtSecret};
use std::{net::SocketAddr, result::Result};

mod app_state;
mod error;
mod game_server_service;
mod health_check_service;
mod matchmaking_assembler;
mod matchmaking_service;
mod matchmaking_started_notification;
mod profile_for_matchmaking_entity;
mod profile_for_matchmaking_manager;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = env_logger::builder()
        .format_target(false)
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Stdout)
        .try_init();

    log::info!("Starting cotonou-matchmaking-service...");

    let app_state = AppState::new().await?;
    let jwt_secret = JwtSecret::new("secret");

    // build our application with a route
    let app = Router::new()
        .route("/healthcheck", get(health_check))
        .route(
            "/matchmaking/:region_system_name/tickets/:owner_profile_id",
            post(create_matchmaking_ticket)
                .delete(delete_matchmaking_ticket)
                .route_layer(middleware::from_fn_with_state(
                    jwt_secret.clone(),
                    jwt_auth_middleware,
                )),
        )
        .route(
            "/matchmaking/:region_system_name/sessions/:session_id",
            put(activate_session)
                .head(update_session)
                .delete(delete_session)
                .route_layer(middleware::from_fn_with_state(
                    jwt_secret.clone(),
                    jwt_auth_middleware,
                )),
        )
        .route(
            "/matchmaking/:region_system_name/sessions/:session_id/:profile_id",
            put(activate_player_session)
                .delete(delete_player_session)
                .route_layer(middleware::from_fn_with_state(
                    jwt_secret.clone(),
                    jwt_auth_middleware,
                )),
        )
        .route(
            "/gameserver/:region_system_name/:game_server_id",
            post(initialize_game_server)
                .put(keep_alive_game_server)
                .delete(shutdown_game_server)
                .route_layer(middleware::from_fn_with_state(
                    jwt_secret,
                    jwt_auth_middleware,
                )),
        )
        .with_state(app_state);

    println!("cotonou-matchmaking-service started!");

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    Ok(axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?)
}
