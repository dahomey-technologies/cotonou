use super::notification::Notification;
use crate::redis::redis_connection_manager::RedisConnectionManager;
use futures::StreamExt;
use rustis::{client::Client, commands::{ListCommands, PubSubCommands, GenericCommands}};
use std::{result, time::Duration};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error;

impl From<rustis::Error> for Error {
    fn from(redis_error: rustis::Error) -> Self {
        println!("Redis Error: {:?}", redis_error);
        Error
    }
}

impl From<serde_json::Error> for Error {
    fn from(json_error: serde_json::Error) -> Self {
        println!("Json Error: {:?}", json_error);
        Error
    }
}

#[derive(Clone)]
pub struct NotificationManager {
    pubsub: Client,
    regular: Client,
}

impl NotificationManager {
    pub fn new(redis_connection_manager: &RedisConnectionManager) -> Self {
        Self {
            pubsub: redis_connection_manager
                .get_client("NOTIFICATIONS_PUBSUB")
                .unwrap(),
            regular: redis_connection_manager
                .get_client("NOTIFICATIONS")
                .unwrap(),
        }
    }

    pub async fn send_notification(
        &self,
        channel_name: &str,
        notification: &dyn Notification,
    ) -> Result<()> {
        let json = serde_json::to_string(notification)?;
        self.send_notification_as_str(channel_name, &json).await
    }

    pub async fn send_notification_as_str(
        &self,
        channel_name: &str,
        notification: &str,
    ) -> Result<()> {
        // data is not sent via pub/sub; the pub/sub API is used only to notify subscriber to check for new notifications
        // the actual data is pushed into a list used as a queue
        self.regular
            .lpush(self.build_channel_queue_key(channel_name), notification.to_owned())
            .await?;
        self.regular.publish(channel_name.to_owned(), "new").await?;
        Ok(())
    }

    pub async fn get_notifications_from_queue(&self, channel_name: &str) -> Result<Vec<String>> {
        let key = self.build_channel_queue_key(channel_name);
        Ok(self.regular.lpop(key, i32::MAX as usize).await?)
    }

    pub async fn get_notifications_from_queue_with_timeout(
        &self,
        channel_name: &str,
        timeout: Duration,
    ) -> Result<Vec<String>> {
        let notifications = self.get_notifications_from_queue(channel_name).await?;
        if timeout.is_zero() || !notifications.is_empty() {
            return Ok(notifications);
        }

        let mut messages = self.pubsub.subscribe(channel_name.to_owned()).await?;
        let msg = tokio::time::timeout(timeout, messages.next()).await;

        match msg {
            Ok(msg) => match msg {
                Some(_) => self.get_notifications_from_queue(channel_name).await,
                // stream closed
                None => Ok(Vec::new()),
            },
            // timeout
            Err(_e) => Ok(Vec::new()),
        }
    }

    pub async fn clear_notification_queue(&self, channel_name: &str) -> Result<()> {
        let key = self.build_channel_queue_key(channel_name);
        self.regular.del(key).await?;
        Ok(())
    }

    fn build_channel_queue_key(&self, channel_name: &str) -> String {
        format!("nq:{channel_name}")
    }
}
