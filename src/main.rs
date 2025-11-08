mod api;
mod args;
mod database;
mod dtos;
mod rabbitmq;
mod types;

use crate::args::Args;
use crate::database::Database;
use crate::rabbitmq::Rabbitmq;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let rmq_client = Rabbitmq::connect(&args.url, &args.vhost).await?;
    let connection_info = rmq_client.get_connection_info();
    let database = Database::new(&connection_info.domain, &connection_info.vhost)?;
    let wwwroot_dir = get_wwwroot_directory()?;

    let app = api::build_api(rmq_client, database, wwwroot_dir);

    println!("Web interface is on http://localhost:{}", args.port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;
    axum::serve(listener, app).await?;
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
