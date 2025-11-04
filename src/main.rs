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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let rmq_client = Rabbitmq::new(&args.url, &args.vhost)?;
    let connection_info = rmq_client.get_connection_info();
    let database = Database::new(&connection_info.domain)?;

    let app = api::build_api(rmq_client, database);

    println!("Web interface is on http://localhost:{}", args.port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
