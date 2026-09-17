use anyhow::Context;
use rand::RngCore;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::KvStore;

pub const SESSION_COOKIE: &str = "session";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    #[serde(skip)]
    store_key: String,
    #[serde(skip)]
    raw_id: String,
    #[serde(skip)]
    is_new: bool,
    #[serde(default)]
    values: HashMap<String, serde_json::Value>,
}

impl Session {
    pub fn new() -> Self {
        let raw_id = generate_session_id();
        let store_key = hash_session_id(&raw_id);
        Self {
            store_key,
            raw_id,
            is_new: true,
            values: HashMap::new(),
        }
    }

    pub async fn load(pool: &SqlitePool, raw_id: &str) -> anyhow::Result<Option<Self>> {
        let store_key = hash_session_id(raw_id);
        let Some(mut session) = Self::try_get_ex(pool, store_key.clone()).await? else {
            return Ok(None);
        };
        session.store_key = store_key;
        session.raw_id = raw_id.to_string();
        session.is_new = false;
        Ok(Some(session))
    }

    pub fn raw_id(&self) -> &str {
        &self.raw_id
    }

    pub fn is_new(&self) -> bool {
        self.is_new
    }

    pub fn user_id(&self) -> Option<Uuid> {
        self.get("user_id")
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.values
            .get(key)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }

    pub async fn attach<T: Serialize>(
        &mut self,
        pool: &SqlitePool,
        key: impl Into<String>,
        value: &T,
    ) -> anyhow::Result<()> {
        let value = serde_json::to_value(value).context("failed to serialize session value")?;
        self.values.insert(key.into(), value);
        self.persist(pool).await
    }

    pub async fn remove(&mut self, pool: &SqlitePool, key: &str) -> anyhow::Result<()> {
        self.values.remove(key);
        self.persist(pool).await
    }

    pub async fn destroy(&mut self, pool: &SqlitePool) -> anyhow::Result<()> {
        Self::del(pool, self.store_key.clone()).await?;
        *self = Self::new();
        Ok(())
    }

    async fn persist(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        self.set_ex(pool, self.store_key.clone()).await
    }
}

impl KvStore for Session {
    const EXPIRE_IN: usize = 60 * 60 * 24 * 14;

    fn key_format(key: String) -> String {
        format!("session:{key}")
    }
}

fn generate_session_id() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn hash_session_id(raw_id: &str) -> String {
    hex::encode(Sha256::digest(raw_id.as_bytes()))
}
