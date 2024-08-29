use std::{env, path::PathBuf};

use anyhow::Context;
use confik::{Configuration, EnvSource, FileSource, Source};

#[derive(Debug, PartialEq, Configuration)]
pub struct Config {
    /// address to bind the application to
    #[confik(default = "localhost:8080")]
    pub bind_addr: String,
    /// database connection url
    #[confik(default = "redis://localhost:6379")]
    pub database_url: String,
    /// number of database connections to pool
    #[confik(default = 8_usize)]
    pub database_pool_size: usize,
    /// directory containing static files (static/) and templates (template/)
    #[confik(default = ".")]
    pub files_dir: String,
}

/// A configuration source for [`confik`]. It is the same as [`confik::FileSource`]
/// but supports loading from multiple files in a fallback manner, and does not error
/// if none of the provided options are found.
#[derive(Debug)]
struct MultiFileSource {
    paths: Vec<PathBuf>,
    allow_secrets: bool,
}

impl MultiFileSource {
    /// Create the source with paths to try.
    pub fn new(paths: Vec<impl Into<PathBuf>>) -> Self {
        Self {
            paths: paths.into_iter().map(|p| p.into()).collect(),
            allow_secrets: false,
        }
    }

    /// Allow secrets to come from this source.
    #[allow(dead_code)]
    pub fn allow_secrets(mut self) -> Self {
        self.allow_secrets = true;
        self
    }
}

impl Source for MultiFileSource {
    fn provide<T: confik::ConfigurationBuilder>(
        &self,
    ) -> Result<T, Box<dyn std::error::Error + Sync + Send>> {
        for p in &self.paths {
            if p.exists() && p.is_file() {
                return FileSource::new(p.to_path_buf()).provide();
            }
        }
        Ok(T::default())
    }

    fn allows_secrets(&self) -> bool {
        self.allow_secrets
    }
}

/// Try to load xmirrord's configuration into [`Config`].
///
/// Precedence:
///
/// 1. environment variables (with the `XMIRRORD_` prefix)
/// 2. configuration file (TOML or JSON) named in environment variable `XMIRRORD_CONFIG`
/// 3. `/etc/xmirror/xmirrord.toml`
/// 4. `/etc/xmirror/xmirrord.json`
pub fn try_load() -> anyhow::Result<Config> {
    let paths = vec![
        env::var("XMIRRORD_CONFIG").unwrap_or_default(),
        "/etc/xmirror/xmirrord.toml".into(),
        "/etc/xmirror/xmirrord.json".into(),
    ];
    Config::builder()
        .override_with(MultiFileSource::new(paths))
        .override_with(EnvSource::new().with_prefix("XMIRRORD_"))
        .try_build()
        .context("Failed to parse configuration")
}
