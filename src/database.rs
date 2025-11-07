use crate::dtos::Message;
use crate::types::db_types::LocalQueue;
use anyhow::Result;
use rusqlite::{Connection, OptionalExtension, Row, ToSql};
use serde_json::Map;
use thiserror::Error;

pub type QueueId = u64;
pub type MessageId = u64;

pub struct Database {
    connection: Connection,
    vhost: String,
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("{:?}", .0)]
    Database(#[from] rusqlite::Error),
    #[error("{:?}", .0)]
    Serialization(#[from] serde_json::Error),
}

impl Database {
    pub fn new(filename: &str, vhost: &str) -> Result<Database> {
        let connection = Connection::open(format!("{}.db", filename))?;

        connection.execute(
            "CREATE TABLE IF NOT EXISTS queues (
            id    INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            name  TEXT NOT NULL,
            vhost TEXT NOT NULL
        )",
            (),
        )?;

        connection.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_queue_name_vhost
            ON queues(name, vhost)",
            (),
        )?;

        connection.execute(
            "CREATE TABLE IF NOT EXISTS messages (
            id        INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            queue_id  TEXT NOT NULL,
            headers   TEXT NOT NULL,
            payload   TEXT NOT NULL,
            FOREIGN KEY(queue_id) REFERENCES queues(id)
        )",
            (),
        )?;

        connection.execute(
            "CREATE INDEX IF NOT EXISTS idx_queue_id
            ON messages(queue_id)",
            (),
        )?;

        Ok(Self {
            connection,
            vhost: vhost.to_string(),
        })
    }

    pub fn get_queues(&self) -> Result<Vec<LocalQueue>, DatabaseError> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT q.id, q.name, coalesce(m.count, 0) FROM queues q
            LEFT JOIN (
                SELECT queue_id, count(*) as count FROM messages
                GROUP BY queue_id
                ) m ON m.queue_id = q.id
            WHERE q.vhost=?
        "#,
        )?;
        let vec = stmt.query_map([&self.vhost], |row| {
            Ok(LocalQueue {
                id: row.get(0)?,
                name: row.get(1)?,
                message_count: row.get(2)?,
            })
        })?;
        Ok(vec.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn find_queue_by_name(&self, name: &str) -> Result<Option<QueueId>, DatabaseError> {
        let mut stmt = self
            .connection
            .prepare("SELECT id FROM queues WHERE name=? AND vhost=?")?;
        let result = stmt
            .query_one([name, &self.vhost], |row| row.get(0))
            .optional()?;
        Ok(result)
    }

    fn get_messages_in_queue(&self, queue_id: QueueId) -> Result<Vec<Message>, DatabaseError> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, payload, headers FROM messages WHERE queue_id = ? ORDER BY id")?;
        let vec = stmt.query_map([queue_id], message_from_row)?;
        Ok(vec.collect::<Result<Vec<_>, _>>()?)
    }

    fn get_messages_by_ids(&self, ids: &[MessageId]) -> Result<Vec<Message>, DatabaseError> {
        let vars = repeat_vars(ids.len());
        let mut stmt = self.connection.prepare(&format!(
            "SELECT id, payload, headers FROM messages WHERE id IN ({vars}) ORDER BY id"
        ))?;
        let vec = stmt.query_map(rusqlite::params_from_iter(ids), message_from_row)?;
        Ok(vec.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get_messages(&self, selector: &MessageSelector) -> Result<Vec<Message>, DatabaseError> {
        match selector {
            MessageSelector::AllInQueue(queue_id) => self.get_messages_in_queue(*queue_id),
            MessageSelector::WithIds(ids) => self.get_messages_by_ids(ids),
        }
    }

    pub fn create_queue(&self, name: &str) -> Result<QueueId, DatabaseError> {
        self.connection.execute(
            "INSERT INTO queues (name, vhost) VALUES (?, ?)",
            [name, &self.vhost],
        )?;
        let queue_id = self
            .find_queue_by_name(name)?
            .expect("Queue ID does not exist, but it was just created");
        Ok(queue_id)
    }

    pub fn set_message_payload(
        &self,
        queue_id: QueueId,
        message_id: MessageId,
        payload: &str,
    ) -> Result<bool, DatabaseError> {
        let num_changed = self.connection.execute(
            "UPDATE messages SET payload = ? WHERE id = ? AND queue_id = ?",
            (payload, message_id, queue_id),
        )?;
        Ok(num_changed == 1)
    }

    pub fn save_messages(
        &self,
        queue_id: QueueId,
        messages: &[(String, Map<String, serde_json::Value>)],
    ) -> Result<(), DatabaseError> {
        let vars = {
            let mut s = "(?,?,?),".repeat(messages.len());
            s.pop(); // Remove trailing comma
            s
        };

        let converted_messages = messages
            .iter()
            .map(|(msg, headers)| {
                let headers = serde_json::to_string(&headers);
                headers.map(|h| (msg, h))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut values: Vec<&dyn ToSql> = Vec::with_capacity(3 * messages.len());
        for message in &converted_messages {
            values.push(&queue_id);
            values.push(message.0);
            values.push(&message.1);
        }

        self.connection.execute(
            &format!("INSERT INTO messages (queue_id, payload, headers) VALUES {vars}"),
            &values[..],
        )?;
        Ok(())
    }

    fn delete_messages_by_ids(&self, ids: &[MessageId]) -> Result<(), DatabaseError> {
        let vars = repeat_vars(ids.len());
        self.connection.execute(
            &format!("DELETE FROM messages WHERE id IN ({vars})"),
            rusqlite::params_from_iter(ids),
        )?;
        Ok(())
    }

    fn delete_all_messages(&self, queue_id: QueueId) -> Result<(), DatabaseError> {
        self.connection
            .execute("DELETE FROM messages WHERE queue_id=?", [queue_id])?;
        Ok(())
    }

    pub fn delete_messages(&self, selector: &MessageSelector) -> Result<(), DatabaseError> {
        match selector {
            MessageSelector::AllInQueue(queue_id) => self.delete_all_messages(*queue_id),
            MessageSelector::WithIds(ids) => self.delete_messages_by_ids(ids),
        }
    }
}

fn repeat_vars(count: usize) -> String {
    assert_ne!(count, 0);
    let mut s = "?,".repeat(count);
    // Remove trailing comma
    s.pop();
    s
}

fn message_from_row(row: &Row) -> Result<Message, rusqlite::Error> {
    let headers: String = row.get(2)?;
    Ok(Message {
        id: row.get(0)?,
        payload: row.get(1)?,
        headers: serde_json::from_str(&headers).unwrap(),
    })
}

pub enum MessageSelector<'a> {
    AllInQueue(QueueId),
    WithIds(&'a [QueueId]),
}
