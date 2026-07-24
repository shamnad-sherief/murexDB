use std::{collections::HashMap, sync::Arc};

use murex_common::{Key, Value};
use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct Database {
    db: Arc<RwLock<HashMap<Key, Value>>>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            db: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &[u8]) -> Option<Value> {
        let guard = self.db.read().await;
        guard.get(key).cloned()
    }

    pub async fn set(&self, key: Key, val: Value) {
        let mut guard = self.db.write().await;
        guard.insert(key, val);
    }

    pub async fn delete(&self, key: &[u8]) -> bool {
        let mut guard = self.db.write().await;
        guard.remove(key).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_db_set_get_delete() {
        let db = Database::new();

        // 1. Get non-existent key
        assert_eq!(db.get(b"key1").await, None);

        // 2. Set key
        db.set(b"key1".to_vec(), b"val1".to_vec()).await;
        assert_eq!(db.get(b"key1").await, Some(b"val1".to_vec()));

        // 3. Delete key
        let deleted = db.delete(b"key1").await;
        assert!(deleted);
        assert_eq!(db.get(b"key1").await, None);
    }
}
