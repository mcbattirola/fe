use rusqlite::{Connection, Result as SqliteResult};
use std::{ffi::OsString, fs, path::PathBuf};

use crate::utils::dir::QuickAccessEntry;

pub struct Storage {
    db: Connection,
}

const TABLE_NAME: &str = "quick_access";

impl Storage {
    pub fn new(mut db_path: PathBuf) -> SqliteResult<Self> {
        db_path.push("db");
        // create dir if it doesn't exist
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).expect("can't data dir");
        }

        let db = Connection::open_with_flags(
            db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE,
        )?;

        db.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {} (path TEXT PRIMARY KEY, name TEXT)",
                TABLE_NAME
            ),
            [],
        )?;
        Ok(Self { db })
    }

    pub fn list_quick_access(&self) -> SqliteResult<Vec<QuickAccessEntry>> {
        let mut stmt = self
            .db
            .prepare(&format!("SELECT path, name FROM {}", TABLE_NAME))?;
        let rows = stmt.query_map([], |row| {
            let path: String = row.get(0)?;
            let name: OsString = row.get::<usize, String>(1)?.into();
            Ok(QuickAccessEntry {
                path: PathBuf::from(path),
                name: name,
            })
        })?;

        let mut entries = Vec::new();
        for result in rows {
            let entry: QuickAccessEntry = result?;
            entries.push(entry);
        }
        Ok(entries)
    }

    pub fn save_quick_access(&self, entry: &QuickAccessEntry) -> SqliteResult<()> {
        self.db.execute(
            &format!(
                "INSERT OR REPLACE INTO {} (path, name) VALUES (?, ?)",
                TABLE_NAME
            ),
            &[
                &entry.path.to_str().unwrap().to_string(),
                &entry.name.to_string_lossy().to_string(),
            ],
        )?;

        Ok(())
    }

    pub fn remove_quick_access(&self, path: &PathBuf) -> SqliteResult<()> {
        self.db.execute(
            &format!("DELETE FROM {} WHERE path = ?", TABLE_NAME),
            &[path.to_str().unwrap()],
        )?;

        Ok(())
    }
}
