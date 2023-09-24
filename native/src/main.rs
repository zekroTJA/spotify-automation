#[macro_use]
extern crate rocket;

mod config;
mod controllers;
mod errors;
mod guards;

use anyhow::Result;
use config::Config;
use controller::UnauthorizedController;
use controllers::{auto, oauth};
use persistence::redis::Redis;

#[rocket::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init()
        .expect("failed initializing logger");

    let cfg = Config::parse()?;
    debug!("Parsed config: {cfg:?}");

    let db = Redis::from_env(false)?;
    let controller = UnauthorizedController::from_env(db)?;

    rocket::build()
        .manage(controller)
        .manage(cfg)
        .mount("/oauth", oauth::routes())
        .mount("/auto", auto::routes())
        .launch()
        .await?;

    Ok(())
}
