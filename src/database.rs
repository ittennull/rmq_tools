use crate::dtos::{LocalQueue, Message};
use anyhow::Result;
use rusqlite::{Connection, Error, OptionalExtension, ToSql};

pub type QueueId = u64;

pub struct Database {
    connection: Connection,
}

pub struct DatabaseError(pub rusqlite::Error);

impl From<rusqlite::Error> for DatabaseError {
    fn from(value: Error) -> Self {
        Self(value)
    }
}

impl Database {
    pub fn new() -> Result<Database> {
        let connection = Connection::open("rmq_tools.db")?;

        connection.execute(
            "CREATE TABLE IF NOT EXISTS queues (
            id   INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        )",
            (), // empty list of parameters.
        )?;

        connection.execute(
            "CREATE TABLE IF NOT EXISTS messages (
            id   INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            queue_id TEXT NOT NULL,
            payload TEXT NOT NULL,
            FOREIGN KEY(queue_id) REFERENCES queues(id)
        )",
            (), // empty list of parameters.
        )?;

        Ok(Self { connection })
    }

    pub fn get_queues(&self) -> Result<Vec<LocalQueue>, DatabaseError> {
        let mut stmt = self.connection.prepare("SELECT id, name FROM queues")?;
        let vec = stmt.query_map([], |row| {
            Ok(LocalQueue {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        Ok(vec.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn find_queue_by_name(&self, name: &str) -> Result<Option<QueueId>, DatabaseError> {
        let mut stmt = self
            .connection
            .prepare("SELECT id FROM queues WHERE name=?")?;
        let result = stmt.query_one([name], |row| row.get(0)).optional()?;
        Ok(result)
    }

    pub fn get_messages(
        &self,
        queue_id: QueueId,
        start: u32,
        take: u32,
    ) -> Result<Vec<Message>, DatabaseError> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, payload FROM messages WHERE queue_id = ?")?;
        let vec = stmt.query_map([queue_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                payload: row.get(1)?,
            })
        })?;
        Ok(vec.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn create_queue(&self, name: &str) -> Result<QueueId, DatabaseError> {
        self.connection
            .execute("INSERT INTO queues (name) VALUES (?)", [name])?;
        let queue_id = self
            .find_queue_by_name(name)?
            .expect("Queue ID does not exist, but it was just created");
        Ok(queue_id)
    }

    pub fn save_messages(
        &self,
        queue_id: QueueId,
        messages: &Vec<String>,
    ) -> Result<(), DatabaseError> {
        let vars = {
            let mut s = String::new();
            for i in (1..=2 * messages.len()).step_by(2) {
                s.push_str(&format!("(?{},?{}),", i, i + 1));
            }
            // Remove trailing comma
            s.pop();
            s
        };

        let mut values: Vec<&dyn ToSql>=Vec::with_capacity(2*messages.len());
        for message in messages {
            values.push(&queue_id);
            values.push(message);
        }

        let items: Vec<_> = messages.into_iter().map(|x| (queue_id, x)).collect();
        self.connection.execute(
            &format!("INSERT INTO messages (queue_id, payload) VALUES {vars}"),
            &values[..],
        )?;
        Ok(())
    }

    pub fn delete_messages(&self, ids: &Vec<u64>) -> Result<(), DatabaseError> {
        let vars = repeat_vars(ids.len());
        self.connection.execute(
            &format!("DELETE FROM messages WHERE id IN ({vars})"),
            rusqlite::params_from_iter(ids),
        )?;
        Ok(())
    }
}

fn repeat_vars(count: usize) -> String {
    assert_ne!(count, 0);
    let mut s = "?,".repeat(count);
    // Remove trailing comma
    s.pop();
    s
}
