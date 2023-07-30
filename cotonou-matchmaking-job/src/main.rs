use crate::{
    error::*, game_server_manager::*, item_cache::*, matchmaking_assembler::*, matchmaking_dal::*,
    matchmaking_job::*, matchmaking_master_job::*, matchmaking_waiting_time_cache::*,
    notification_cache::*, queue_map::*, util::*,
};
use tokio::sync::watch;

mod error;
mod game_server_manager;
mod item_cache;
mod match_functions;
mod matchmaker;
mod matchmaking_assembler;
mod matchmaking_dal;
mod matchmaking_job;
mod matchmaking_master_job;
mod matchmaking_waiting_time_cache;
mod notification_cache;
mod queue_map;
mod util;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = env_logger::builder()
        .format_target(false)
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Trace)
        .target(env_logger::Target::Stdout)
        .try_init();

    log::info!("Starting cotonou-matchmaking-job...");

    let (shutdown_sender, shutown_receiver) = watch::channel(());
    let matchmaking_master_job = MatchmakingMasterJob::new(shutown_receiver).await?;

    log::info!("cotonou-matchmaking-job started!");

    let join_handle = tokio::spawn(async move { matchmaking_master_job.initialize().await });

    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            log::info!("shutdown requested");
            shutdown_sender.send(())?;
        }
        Err(err) => {
            log::error!("Unable to listen for shutdown signal: {}", err);
            shutdown_sender.send(())?;
        }
    }

    join_handle.await??;

    Ok(())
}
