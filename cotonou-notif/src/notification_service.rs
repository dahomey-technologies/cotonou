use crate::Error;
use axum::{extract::State, response::Response, Extension};
use cotonou_common::{
    authentication::User, notifications::NotificationManager,
};
use hyper::StatusCode;
use std::{sync::Arc, time::Duration};

pub async fn get_notifications(
    State(notification_manager): State<Arc<NotificationManager>>,
    Extension(user): Extension<User>,
) -> Result<Response<String>, Error> {
    let channel_name = build_channel_name(&user);
    let notification_jsons = notification_manager
        .get_notifications_from_queue_with_timeout(channel_name, Duration::from_secs(10))
        .await?;
    let response_json = format!("{{\"notifications\":[{}]}}", notification_jsons.join(","));
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(response_json)
        .unwrap())
}

pub async fn clear_notifications(
    State(notification_manager): State<Arc<NotificationManager>>,
    Extension(user): Extension<User>,
) -> Result<(), Error> {
    let channel_name = build_channel_name(&user);
    notification_manager
        .clear_notification_queue(channel_name)
        .await?;

    Ok(())
}

pub async fn test_publish(
    State(notification_manager): State<Arc<NotificationManager>>,
    Extension(user): Extension<User>,
) -> Result<(), Error> {
    let channel_name = build_channel_name(&user);
    notification_manager
        .send_notification_as_str(channel_name, &format!("\"test-{}\"", user.subject))
        .await?;

    Ok(())
}

fn build_channel_name(user: &User) -> &str {
    &user.subject
}
