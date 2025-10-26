mod api;
mod args;
mod database;
mod dtos;
mod rabbitmq;

use crate::args::Args;
use crate::database::Database;
use crate::rabbitmq::Rabbitmq;
use anyhow::Result;
use clap::Parser;
use rabbitmq_http_client::api::Client;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let rmq_client = connect_to_rmq(&args)?;
    let database = Database::new()?;

    let app = api::build_api(rmq_client, database);

    println!("Web interface is on http://localhost:{}", args.port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn connect_to_rmq(args: &Args) -> Result<Rabbitmq> {
    let url = Url::parse(&args.url)?;
    let endpoint = format!(
        "{}://{}:{}{}",
        url.scheme(),
        url.domain().expect("Domain is missing"),
        url.port().unwrap_or(443),
        url.path()
    );

    println!(
        "Connecting to endpoint '{}' and vhost '{}'",
        endpoint, args.vhost
    );
    let client = Client::new(
        endpoint,
        url.username().to_string(),
        url.password().expect("Password is missing").to_string(),
    );

    Ok(Rabbitmq::new(client, args.vhost.clone()))
}
