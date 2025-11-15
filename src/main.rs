mod api;
mod args;
mod database;
mod dtos;
mod rabbitmq;
mod rmq_background;
mod types;

use crate::args::Args;
use crate::database::Database;
use crate::rabbitmq::Rabbitmq;
use crate::rmq_background::RmqBackground;
use anyhow::Result;
use clap::Parser;
use log::{error, info, LevelFilter};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .format_timestamp(None)
        .format_target(false)
        .init();

    _ = run().await.inspect_err(|e| error!("{:?}", e));
}

async fn run() -> Result<()> {
    let args = Args::parse();
    let rmq_client = Arc::new(Rabbitmq::connect(&args.url, &args.vhost).await?);
    let rmq_background = RmqBackground::new(Arc::clone(&rmq_client));
    let connection_info = rmq_client.get_connection_info();
    let database = Database::new(&connection_info.domain, &connection_info.vhost)?;
    let wwwroot_dir = get_wwwroot_directory()?;

    let app = api::build_api(rmq_client, database, rmq_background, wwwroot_dir);

    info!("Web interface is on http://localhost:{}", args.port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

fn get_wwwroot_directory() -> Result<PathBuf> {
    const WWWROOT_DIR: &str = "wwwroot";

    let mut path = std::env::current_exe()?;
    path.pop();
    path.push(WWWROOT_DIR);

    match path.exists() {
        true => Ok(path),
        false => Err(anyhow::anyhow!(
            "Cannot find '{WWWROOT_DIR}' folder. If the executable was invoked through a symbolic link, some platforms will return the path of the symbolic link and other platforms will return the path of the symbolic link’s target. In such a case invoke the tool by its full path"
        )),
    }
}
