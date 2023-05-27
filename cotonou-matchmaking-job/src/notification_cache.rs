use crate::error::Error;
use cotonou_common::{
    matchmaking::game_server::GameServerId,
    notifications::{notification::Notification, notification_manager::NotificationManager},
    models::ProfileId,
};
use std::collections::VecDeque;

struct NotificationInfo {
    channel: String,
    notification: Box<dyn Notification>,
}

pub struct NotificationCache {
    notification_manager: NotificationManager,
    notification_queue: VecDeque<NotificationInfo>,
}

impl NotificationCache {
    //-------------------------------------------------------------------------------------------------
    pub fn new(notification_manager: NotificationManager) -> Self {
        Self {
            notification_manager,
            notification_queue: VecDeque::new(),
        }
    }

    //-------------------------------------------------------------------------------------------------
    pub fn queue_gamer_server_notification<N: Notification + 'static>(
        &mut self,
        game_server_id: GameServerId,
        notification: N,
    ) {
        self.notification_queue.push_back(NotificationInfo {
            channel: game_server_id.to_string(),
            notification: Box::new(notification),
        });
    }

    //-------------------------------------------------------------------------------------------------
    pub fn queue_player_notification<N: Notification + 'static>(
        &mut self,
        profile_id: ProfileId,
        notification: N,
    ) {
        self.notification_queue.push_back(NotificationInfo {
            channel: profile_id.to_string(),
            notification: Box::new(notification),
        });
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn send_notifications(&mut self) -> Result<(), Error> {
        while let Some(notification) = self.notification_queue.pop_back() {
            self.notification_manager
                .send_notification(&notification.channel, notification.notification.as_ref())
                .await?;
        }

        Ok(())
    }
}
