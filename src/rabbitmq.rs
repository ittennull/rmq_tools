use crate::dtos::{RemoteQueue, RmqConnectionInfo};
use anyhow::anyhow;
use rabbitmq_http_client::api::{Client, HttpClientError};
use rabbitmq_http_client::responses::GetMessage;
use url::Url;

pub struct Rabbitmq {
    client: Client<String, String, String>,
    domain: String,
    vhost: String,
}

#[derive(Debug)]
pub enum RabbitMQError {
    HttpClientError(HttpClientError),
    Other(anyhow::Error),
}

impl From<HttpClientError> for RabbitMQError {
    fn from(value: HttpClientError) -> Self {
        RabbitMQError::HttpClientError(value)
    }
}

impl Rabbitmq {
    pub fn new(url: &str, vhost: &str) -> Result<Self, anyhow::Error> {
        let url = Url::parse(&url)?;
        let domain = url.domain().expect("Domain is missing").to_string();
        let endpoint = format!(
            "{}://{}:{}{}",
            url.scheme(),
            domain,
            url.port().unwrap_or(443),
            url.path()
        );

        println!(
            "Connecting to endpoint '{}' and vhost '{}'",
            endpoint, vhost
        );
        let client = Client::new(
            endpoint,
            url.username().to_string(),
            url.password().expect("Password is missing").to_string(),
        );

        Ok(Self {
            client,
            domain,
            vhost: vhost.to_string(),
        })
    }

    pub fn get_connection_info(&self) -> RmqConnectionInfo {
        RmqConnectionInfo {
            domain: self.domain.clone(),
            vhost: self.vhost.clone(),
        }
    }

    pub async fn list_queues(&self) -> Result<Vec<RemoteQueue>, RabbitMQError> {
        let queues = self
            .client
            .list_queues_in(&self.vhost)
            .await?
            .into_iter()
            .map(|q| RemoteQueue {
                name: q.name,
                message_count: q.message_count,
                exclusive: q.exclusive,
            })
            .collect();

        Ok(queues)
    }

    pub async fn load_messages(&self, queue: &str) -> Result<Vec<GetMessage>, RabbitMQError> {
        let queue_info = self.client.get_queue_info(&self.vhost, queue).await?;
        if queue_info.exclusive {
            return Err(RabbitMQError::Other(anyhow!(
                "Queue {} is exclusive",
                queue
            )));
        }

        let messages = self
            .client
            .get_messages(
                &self.vhost,
                queue,
                queue_info.message_count as u32,
                "ack_requeue_false",
            )
            .await?;

        Ok(messages)
    }
}
