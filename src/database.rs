use std::time::Duration;

use anyhow::Context;
use fred::{prelude::*, types::Scanner};
use futures::stream::TryStreamExt;
use log::{debug, info};

use crate::types::Mirror;

#[derive(Debug, Clone)]
pub struct Database {
    pool: RedisPool,
}

impl Database {
    /// Try to initialize the database connection.
    pub async fn try_init(
        url: &str,
        poolsz: usize,
    ) -> anyhow::Result<Self> {
        let config = RedisConfig::from_url(url).context("Failed to build database config")?;

        let pool = Builder::from_config(config)
            .with_connection_config(|c| c.connection_timeout = Duration::from_secs(10))
            .with_performance_config(|c| c.auto_pipeline = true)
            .set_policy(ReconnectPolicy::new_exponential(0, 100, 30_000, 2))
            .build_pool(poolsz)
            .context("Failed to create database connection pool")?;
        info!("Initializing and pinging database connection pool...");
        pool.init().await.context("Failed to initialize database connection pool")?;
        pool.ping().await.context("Failed to ping database")?;

        info!("Initialized and pinged database connection pool at {url}");
        Ok(Self { pool })
    }

    pub async fn list_all_mirrors(&self) -> Result<Vec<u64>, RedisError> {
        debug!("scanning mirror list");

        let mut keys = vec![];
        let mut scanner = self
            .pool
            .next()
            .scan("xmirror:mirror:*", Some(100), None);

        while let Some(mut page) = scanner.try_next().await? {
            if let Some(page) = page.take_results() {
                keys.extend(page.iter().filter_map(|k| {
                    k.as_str()?
                        .trim_start_matches("xmirror:mirror:")
                        .parse::<u64>()
                        .ok()
                }))
            }
            let _ = page.next();
        }
        keys.sort();
        Ok(keys)
    }

    pub async fn get_all_mirrors(&self) -> Result<Vec<Mirror>, RedisError> {
        debug!("getting all mirrors");
        let mut mirrors = vec![];
        for k in self.list_all_mirrors().await? {
            mirrors.push(self.get_mirror(k).await?)
        }
        Ok(mirrors)
    }

    pub async fn get_mirror(&self, id: u64) -> Result<Mirror, RedisError> {
        debug!("getting mirror {id}");
        let mut mirror: Mirror = self.pool.hgetall(format!("xmirror:mirror:{id}")).await?;
        mirror.id = id;
        Ok(mirror)
    }

    pub async fn add_mirror(&self) -> Result<(), RedisError> {
        todo!()
    }

    pub async fn update_mirror(&self) -> Result<(), RedisError> {
        todo!()
    }

    pub async fn delete_mirror(&self) -> Result<(), RedisError> {
        todo!()
    }
}
