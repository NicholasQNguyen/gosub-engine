use crate::settings::Setting;
use crate::StorageAdapter;
use gosub_shared::types::Result;
use log::warn;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{LockResult, Mutex};

pub struct SqliteStorageAdapter {
    connection: Mutex<sqlite::Connection>,
}

impl TryFrom<&String> for SqliteStorageAdapter {
    type Error = anyhow::Error;

    fn try_from(path: &String) -> Result<Self> {
        let conn = sqlite::open(path).expect("cannot open db file");

        let query = "CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY,
            key TEXT NOT NULL,
            value TEXT NOT NULL
        )";
        conn.execute(query)?;

        Ok(SqliteStorageAdapter {
            connection: Mutex::new(conn),
        })
    }
}

impl StorageAdapter for SqliteStorageAdapter {
    fn get(&self, key: &str) -> Option<Setting> {
        let db_lock = match self.connection.lock() {
            Ok(l) => {l}
            Err(e) => {
                warn!("Poisoned mutex {e}");
                return None
            }
        };

        let query = "SELECT * FROM settings WHERE key = :key";
        // If any of these sqlite commands fail at any point,
        // Then we return a None
        let mut statement = match db_lock.prepare(query) {
            Ok(s) => {s}
            Err(_) => {
                warn!("problem preparing statement: {err}");
                return None
            }
        };
        match statement.bind((":key", key)) {
            Ok(_) => {}
            Err(_) => {
                warn!("problem binding statement: {err}");
                return None
            }
        };

        match Setting::from_str(key) {
            Ok(setting) => Some(setting),
            Err(err) => {
                warn!("problem reading from sqlite: {err}");
                None
            }
        }
    }

    fn set(&self, key: &str, value: Setting) -> Result<()> {
        let db_lock = self.connection.lock().expect("Poisoned");

        let query = "INSERT OR REPLACE INTO settings (key, value) VALUES (:key, :value)";
        let mut statement = db_lock.prepare(query)?;
        statement.bind((":key", key))?;
        statement.bind((":value", value.to_string().as_str()))?;

        statement.next()?;
        Ok(())
    }

    fn all(&self) -> Result<HashMap<String, Setting>> {
        let db_lock = self.connection.lock().expect("Poisoned");

        let query = "SELECT * FROM settings";
        let mut statement = db_lock.prepare(query)?;

        let mut settings = HashMap::new();
        while let sqlite::State::Row = statement.next()? {
            let key = statement.read::<String, _>(1)?;
            let value = statement.read::<String, _>(2)?;
            settings.insert(key, Setting::from_str(&value)?);
        }

        Ok(settings)
    }
}
