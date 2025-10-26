use crate::dtos::RemoteQueue;
use rabbitmq_http_client::api::{Client, HttpClientError};

pub struct Rabbitmq {
    client: Client<String, String, String>,
    vhost: String,
}

pub struct RabbitMQError(pub HttpClientError);

impl From<HttpClientError> for RabbitMQError {
    fn from(value: HttpClientError) -> Self {
        Self(value)
    }
}

impl Rabbitmq {
    pub fn new(client: Client<String, String, String>, vhost: String) -> Self {
        Self { client, vhost }
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
}
