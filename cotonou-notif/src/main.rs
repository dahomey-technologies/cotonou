use crate::{error::*, notification_service::*};
use axum::{middleware, routing::get, Router};
use common_macros::hash_map;
use cotonou_common::{
    authentication::{jwt_auth_middleware, JwtSecret},
    notifications::NotificationManager,
    redis::{RedisConfig, RedisConnectionManager},
};
use std::{net::SocketAddr, sync::Arc};

mod error;
mod notification_service;

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Starting cotonou-notif...");

    let redis_connection_manager = RedisConnectionManager::initialize(RedisConfig {
        connection_strings: hash_map! {
            "NOTIFICATIONS".to_owned() => "redis://redis:6379/0".to_owned(),
            "NOTIFICATIONS_PUBSUB".to_owned() => "redis://redis:6379/0".to_owned()
        },
    })
    .await?;

    let notification_manager = Arc::new(NotificationManager::new(&redis_connection_manager));

    let jwt_secret = JwtSecret::new("secret");

    println!("cotonou-notif started!");

    // build our application with a route
    let app = Router::new()
        .route(
            "/",
            get(get_notifications)
                .delete(clear_notifications)
                .post(test_publish),
        )
        .route_layer(middleware::from_fn_with_state(
            jwt_secret,
            jwt_auth_middleware,
        ))
        .with_state(notification_manager);

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    Ok(axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?)
}
