use std::path::PathBuf;

use crate::utils::dir::QuickAccessEntry;
use sled::Db;

pub struct Storage {
    db: Db,
}

const KEY_QUICK_ACCESS: &str = "quick_access";

impl Storage {
    pub fn new(path: String) -> Result<Self, sled::Error> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    pub fn list_quick_access(&self) -> Vec<QuickAccessEntry> {
        match self.db.get(KEY_QUICK_ACCESS) {
            Ok(Some(value)) => bincode::deserialize(&value).unwrap_or_default(),
            Ok(None) => vec![],
            Err(_) => vec![], // TODO handle error
        }
    }

    pub fn save_quick_access(&self, entry: QuickAccessEntry) -> Option<()> {
        // Retrieve the current list
        let mut entries = self.list_quick_access();

        // Append the new entry
        entries.push(entry);

        // Serialize the updated list and store it
        let serialized = bincode::serialize(&entries).ok()?;
        self.db.insert(KEY_QUICK_ACCESS, serialized).ok()?;
        Some(())
    }

    pub fn remove_quick_access(&self, path: &PathBuf) -> Option<()> {
        // Retrieve the current list
        let mut entries = self.list_quick_access();

        // remove the path
        entries.retain(|entry| entry.path != *path);

        // Serialize the updated list and store it
        let serialized = bincode::serialize(&entries).ok()?;
        self.db.insert(KEY_QUICK_ACCESS, serialized).ok()?;
        Some(())
    }
}
