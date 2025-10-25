mod api;
mod args;
mod dtos;

use crate::api::{list_queues, AppState, RmqClient};
use crate::args::Args;
use anyhow::Result;
use axum::{routing::get, Router};
use clap::Parser;
use rabbitmq_http_client::api::Client;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let rmq_client = connect_to_rmq(&args)?;

    let app = build_api(rmq_client);

    println!("Web interface is on http://localhost:{}", args.port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn connect_to_rmq(args: &Args) -> Result<RmqClient> {
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

    Ok(RmqClient::new(client, args.vhost.clone()))
}

fn build_api(rmq_client: RmqClient) -> Router {
    let state = AppState::new(rmq_client);

    Router::new()
        .route("/queues", get(list_queues))
        .with_state(state)
}
