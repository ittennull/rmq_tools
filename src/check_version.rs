use anyhow::Result;
use axum::http::header::USER_AGENT;
use clap::crate_version;
use log::debug;
use serde::Deserialize;

pub async fn show_notification_if_new_version_available() {
    if let Ok(response) = get_latest_version().await {
        let current_version = crate_version!();
        if response.tag_name != current_version {
            println!(
                r#" _   _                                                     _
| \ | |   ___  __      __     __   __   ___   _ __   ___  (_)   ___    _ __
|  \| |  / _ \ \ \ /\ / /     \ \ / /  / _ \ | '__| / __| | |  / _ \  | '_ \
| |\  | |  __/  \ V  V /       \ V /  |  __/ | |    \__ \ | | | (_) | | | | |
|_| \_|  \___|   \_/\_/         \_/    \___| |_|    |___/ |_|  \___/  |_| |_|"#
            );
            println!(
                "Current version {}, new version {}",
                current_version, response.tag_name
            );
            println!("Grab the new one here: {}\n", response.html_url)
        }
    }
}

async fn get_latest_version() -> Result<GetLatestReleaseResponse> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/ittennull/rmq_tools/releases/latest")
        .header(USER_AGENT, "rust-web-api-client")
        .send()
        .await?;
    debug!("GitHub API call returned status {}", response.status());
    Ok(response.json().await?)
}

#[derive(Deserialize)]
struct GetLatestReleaseResponse {
    tag_name: String,
    html_url: String,
}
