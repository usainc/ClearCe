use crate::core::processing::{
    errors::{err, ErrorCode, Result},
    task::{Task, TaskStatus},
};
use rusqlite::{params, Connection};
use std::{path::Path, sync::Mutex};
pub struct History {
    conn: Mutex<Connection>,
}
impl History {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).map_err(|_| err(ErrorCode::StorageError))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS tasks (id TEXT PRIMARY KEY, created_at INTEGER NOT NULL, payload TEXT NOT NULL);").map_err(|_|err(ErrorCode::StorageError))?;
        let history = Self {
            conn: Mutex::new(conn),
        };
        history.conn.lock().map_err(|_|err(ErrorCode::StorageError))?.execute_batch("CREATE TABLE IF NOT EXISTS queue_state (id INTEGER PRIMARY KEY CHECK(id=1), payload TEXT NOT NULL);").map_err(|_|err(ErrorCode::StorageError))?;
        for mut task in history.list()? {
            if !task.status.terminal() {
                task.transition(TaskStatus::Failed, "Interrupted by app shutdown")?;
                task.error = Some(err(ErrorCode::InterruptedJob));
                log::info!("event=interrupted_job_recovered");
                history.save(&task)?;
            }
        }
        Ok(history)
    }
    pub fn save(&self, task: &Task) -> Result<()> {
        let payload = serde_json::to_string(task).map_err(|_| err(ErrorCode::StorageError))?;
        self.conn.lock().map_err(|_|err(ErrorCode::StorageError))?.execute("INSERT INTO tasks(id,created_at,payload) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload",params![task.id,task.created_at,payload]).map_err(|_|err(ErrorCode::StorageError))?;
        Ok(())
    }
    pub fn list(&self) -> Result<Vec<Task>> {
        let conn = self.conn.lock().map_err(|_| err(ErrorCode::StorageError))?;
        let mut statement = conn
            .prepare("SELECT payload FROM tasks ORDER BY created_at DESC LIMIT 500")
            .map_err(|_| err(ErrorCode::StorageError))?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|_| err(ErrorCode::StorageError))?;
        rows.map(|row| {
            serde_json::from_str(&row.map_err(|_| err(ErrorCode::StorageError))?)
                .map_err(|_| err(ErrorCode::StorageError))
        })
        .collect()
    }
    pub fn get(&self, id: &str) -> Result<Task> {
        let conn = self.conn.lock().map_err(|_| err(ErrorCode::StorageError))?;
        let payload: String = conn
            .query_row("SELECT payload FROM tasks WHERE id=?1", [id], |row| {
                row.get(0)
            })
            .map_err(|_| err(ErrorCode::TaskNotFound))?;
        serde_json::from_str(&payload).map_err(|_| err(ErrorCode::StorageError))
    }
    pub fn delete(&self, id: &str) -> Result<()> {
        self.conn
            .lock()
            .map_err(|_| err(ErrorCode::StorageError))?
            .execute("DELETE FROM tasks WHERE id=?1", [id])
            .map_err(|_| err(ErrorCode::StorageError))?;
        Ok(())
    }
    pub fn read_queue<T: serde::de::DeserializeOwned + Default>(&self) -> Result<T> {
        use rusqlite::OptionalExtension;
        let json: Option<String> = self
            .conn
            .lock()
            .map_err(|_| err(ErrorCode::StorageError))?
            .query_row("SELECT payload FROM queue_state WHERE id=1", [], |row| {
                row.get(0)
            })
            .optional()
            .map_err(|_| err(ErrorCode::StorageError))?;
        match json {
            Some(json) => serde_json::from_str(&json).map_err(|_| err(ErrorCode::StorageError)),
            None => Ok(T::default()),
        }
    }
    pub fn save_queue<T: serde::Serialize>(&self, value: &T) -> Result<()> {
        let json = serde_json::to_string(value).map_err(|_| err(ErrorCode::StorageError))?;
        self.conn.lock().map_err(|_|err(ErrorCode::StorageError))?.execute("INSERT INTO queue_state(id,payload) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload",[json]).map_err(|_|err(ErrorCode::StorageError))?;
        Ok(())
    }
}
