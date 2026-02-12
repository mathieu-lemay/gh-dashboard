use std::collections::HashMap;
use std::env;
use std::process::Command;

use config::{Value, ValueKind};
use exn::{Result, ResultExt, bail};
use gh_dashboard::Error;
use log::debug;
use secrecy::SecretString;
use serde::Deserialize;

use crate::models::Repository;

#[derive(Debug, Error)]
pub struct AuthError(String);

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub host: String,
    auth_token: Option<SecretString>,
    pub repos: Vec<Repository>,
}

impl Settings {
    pub fn token(&self) -> Result<SecretString, AuthError> {
        if let Some(token) = self.auth_token.as_ref() {
            debug!("Using github token from config");
            return Ok(token.clone());
        }

        if let Ok(t) = env::var("GITHUB_TOKEN") {
            debug!("Using github token from GITHUB_TOKEN environment variable");
            return Ok(SecretString::from(t));
        }

        let gh_cli = env::var("GH_PATH").unwrap_or("gh".to_string());
        let cmd = Command::new(gh_cli)
            .args(["auth", "token", "--hostname", &self.host])
            .output();

        match cmd {
            Ok(output) => {
                if output.status.success() {
                    debug!("Using github token from GH cli");
                    return Ok(SecretString::from(
                        String::from_utf8_lossy(&output.stdout).trim().to_string(),
                    ));
                }
                debug!("No valid token from GH cli");
            }
            Err(e) => {
                debug!("Error getting auth token from GH cli: {}", e);
            }
        }

        bail!(AuthError::from("Unable to find GitHub token"));
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            host: "github.com".to_string(),
            auth_token: None,
            repos: vec![],
        }
    }
}

impl From<Settings> for ValueKind {
    fn from(value: Settings) -> Self {
        let mut table = HashMap::new();

        table.insert(
            "host".to_string(),
            Value::new(None, ValueKind::String(value.host)),
        );

        ValueKind::Table(table)
    }
}

#[derive(Debug, Error)]
pub struct ConfigError(String);

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let default = Settings::default();

    let make_err = || ConfigError::from("error initializing configuration");

    let mut config_files =
        vec![config::File::new("config.toml", config::FileFormat::Toml).required(false)];

    if let Some(dir) = dirs::config_dir() {
        let f = config::File::new(
            dir.join("gh-dashboard")
                .join("config.toml")
                .to_str()
                .unwrap(),
            config::FileFormat::Toml,
        )
        .required(false);

        config_files.push(f);
    }

    config_files.push(
        config::File::new("/etc/gh-dashboard.toml", config::FileFormat::Toml).required(false),
    );

    let mut builder = config::Config::builder();

    for file in config_files {
        builder = builder.add_source(file);
    }

    let settings = builder
        .add_source(config::Environment::with_prefix("GH_DASHBOARD"))
        .set_default("host", default.host)
        .or_raise(make_err)?
        .build()
        .or_raise(make_err)?;

    settings.try_deserialize::<Settings>().or_raise(make_err)
}
