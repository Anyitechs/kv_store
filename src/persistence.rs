use crate::store::KeyValueStore;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct PersistentData {
    data: Vec<(String, String)>,
}

impl KeyValueStore {
    pub async fn load(&self, path: &str) -> anyhow::Result<()> {
        if Path::new(path).exists() {
            let content = tokio::fs::read_to_string(path).await?;
            let persistent_data: PersistentData = serde_json::from_str(&content)?;
            let mut map = self.map.lock().await;
            map.extend(persistent_data.data);
        }
        Ok(())
    }

    pub async fn save(&self, path: &str) -> anyhow::Result<()> {
        let data = {
            let map = self.map.lock().await;
            PersistentData {
                data: map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            }
        };

        let content = serde_json::to_string(&data)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }
}
