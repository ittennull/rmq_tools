use crate::dtos::{LocalQueue, Message};
use anyhow::Result;
use rusqlite::{Connection, Error};

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
            queue_id TEXT NOT NULL UNIQUE,
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

    pub fn find_queue_by_name(&self, name: &str) -> Result<u64, DatabaseError> {
        let mut stmt = self
            .connection
            .prepare("SELECT id FROM queues WHERE name=?")?;
        let result = stmt.query_one([name], |row| row.get(0))?;
        Ok(result)
    }

    pub fn get_messages(
        &self,
        queue_id: u64,
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
