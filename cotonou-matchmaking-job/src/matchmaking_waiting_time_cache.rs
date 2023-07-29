use crate::error::Error;
use cotonou_common::{
    matchmaking::matchmaking_ticket::{MatchmakingPlayer, MatchmakingTicket},
    matchmaking_average_waiting_time_dal::MatchmakingWaitingTimeDAL,
    unix_now,
};
use futures_util::future;
use std::collections::VecDeque;

const MAX_WAITING_QUEUE_SIZE: usize = 10;

struct WaitingTimeInfo {
    pub game_mode: String,
    pub last_waiting_times: VecDeque<u64>,
    pub median_waiting_time: u64,
}

pub struct MatchmakingWaitingTimeCache {
    region_system_name: String,
    matchmaking_waiting_time_dal: MatchmakingWaitingTimeDAL,
    waiting_time_infos: Vec<WaitingTimeInfo>,
}

impl MatchmakingWaitingTimeCache {
    //-------------------------------------------------------------------------------------------------
    pub fn new(
        region_system_name: &str,
        matchmaking_waiting_time_dal: MatchmakingWaitingTimeDAL,
    ) -> Self {
        Self {
            region_system_name: region_system_name.to_owned(),
            matchmaking_waiting_time_dal,
            waiting_time_infos: Vec::new(),
        }
    }

    //-------------------------------------------------------------------------------------------------
    pub fn update_cache(&mut self, ticket: &MatchmakingTicket, _player: &MatchmakingPlayer) {
        let game_mode = ticket.game_mode.as_str();

        let info = self
            .waiting_time_infos
            .iter_mut()
            .find(|i| i.game_mode == game_mode);

        let info = if let Some(info) = info {
            info
        } else {
            let info = WaitingTimeInfo {
                game_mode: game_mode.to_owned(),
                last_waiting_times: VecDeque::new(),
                median_waiting_time: 0,
            };
            self.waiting_time_infos.push(info);
            self.waiting_time_infos.last_mut().unwrap()
        };

        while info.last_waiting_times.len() >= MAX_WAITING_QUEUE_SIZE {
            info.last_waiting_times.pop_back();
        }

        info.last_waiting_times
            .push_front(unix_now() - ticket.creation_time);

        let count = info.last_waiting_times.len();

        info.median_waiting_time = if count > 0 {
            let mut times = info.last_waiting_times.iter().copied().collect::<Vec<_>>();
            times.sort();
            times[count / 2]
        } else {
            0u64
        };
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn save_cache(&self) -> Result<(), Error> {
        let tasks = self
            .waiting_time_infos
            .iter()
            .map(|i| {
                self.matchmaking_waiting_time_dal.set_average_waiting_time(
                    &self.region_system_name,
                    &i.game_mode,
                    i.median_waiting_time,
                )
            })
            .collect::<Vec<_>>();
        let results = future::join_all(tasks).await;

        match results.into_iter().find_map(|r| r.err()) {
            Some(error) => Err(error.into()),
            None => Ok(()),
        }
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn reset(&mut self) -> Result<(), Error> {
        let tasks = self
            .waiting_time_infos
            .iter()
            .map(|i| {
                self.matchmaking_waiting_time_dal
                    .reset(&self.region_system_name, &i.game_mode)
            })
            .collect::<Vec<_>>();
        let results = future::join_all(tasks).await;

        if let Some(error) = results.into_iter().find_map(|r| r.err()) {
            Err(error.into())
        } else {
            self.waiting_time_infos.clear();
            Ok(())
        }
    }

    //-------------------------------------------------------------------------------------------------
    pub fn get_average_waiting_time(&self, game_mode: &str) -> u64 {
        self.waiting_time_infos
            .iter()
            .find(|i| i.game_mode == game_mode)
            .map(|i| i.median_waiting_time)
            .unwrap_or_default()
    }
}
