use std::{collections::HashMap, env, path::PathBuf, time::Duration};

use eyre::{Context as _, Result};
use jiff::{SignedDuration, Timestamp};
use tokio::fs::{create_dir_all, read, write};
use uuid::Uuid;

use crate::model::Doggo;

const KENNEL_PATH_ENV_VAR: &str = "KENNEL_PATH";

pub struct Kennel {
    path: PathBuf,
    pub doggos: HashMap<Uuid, Doggo>,
}

impl Kennel {
    pub async fn new() -> Result<Self> {
        let path: PathBuf = env::var(KENNEL_PATH_ENV_VAR)
            .wrap_err_with(|| {
                format!(
                    "getting path to kennel from the environment variable {KENNEL_PATH_ENV_VAR}"
                )
            })?
            .parse()?;

        if let Some(parent) = path.parent() {
            create_dir_all(parent).await.wrap_err_with(|| {
                format!(
                    "(recursively) creating directory at {} for kennel persistence",
                    parent.display()
                )
            })?;
        }

        let json = match read(&path).await {
            Ok(json) => json,
            Err(err) => {
                println!(
                    "Couldn't load doggos from {path:?}: {err:#?}. Initializing empty kennel."
                );
                return Ok(Self::empty(path));
            }
        };

        let parsed: Result<HashMap<Uuid, Doggo>, serde_json::Error> = serde_json::from_slice(&json);
        Ok(match parsed {
            Ok(doggos) => {
                println!("Loaded {} doggos from {:?}", doggos.len(), &path);
                Self { path, doggos }
            }
            Err(err) => {
                println!(
                    "Couldn't load doggos from {path:?}: {err:#?}. Initializing empty kennel."
                );
                Self::empty(path)
            }
        })
    }

    fn empty(path: PathBuf) -> Self {
        Self {
            path,
            doggos: HashMap::new(),
        }
    }

    pub async fn persist(&self) -> Result<()> {
        let json = serde_json::to_string(&self.doggos).wrap_err("serializing doggos")?;
        write(&self.path, json)
            .await
            .wrap_err_with(|| format!("persisting serialized doggos to {:?}", &self.path))
    }

    pub fn until_next_run(&self) -> Option<Duration> {
        let mut next_runs: Vec<Timestamp> =
            self.doggos.values().map(|doggo| doggo.next_run()).collect();
        next_runs.sort();

        let closest_next_run = next_runs.first();
        closest_next_run.map(|next| {
            Timestamp::now()
                .duration_until(*next)
                // Prevent busy loops, just in case.
                .max(SignedDuration::from_secs(1))
                // We're already positive thanks to max() above but we need to convert the type to
                // Duration to be accepted by sleep().
                .unsigned_abs()
        })
    }
}
