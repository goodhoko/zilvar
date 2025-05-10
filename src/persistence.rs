use std::{collections::HashMap, path::PathBuf, time::Duration};

use eyre::{Context as _, Result};
use jiff::{SignedDuration, Timestamp};
use tokio::fs::{read, write};
use uuid::Uuid;

use crate::model::Doggo;

pub struct Kennel {
    path: PathBuf,
    pub doggos: HashMap<Uuid, Doggo>,
}

impl Kennel {
    pub async fn new(path: PathBuf) -> Self {
        let json = match read(&path).await {
            Ok(json) => json,
            Err(err) => {
                println!(
                    "Couldn't load doggos from {path:?}: {err:#?}. Initializing empty kennel."
                );
                return Self::empty(path);
            }
        };

        let parsed: Result<HashMap<Uuid, Doggo>, serde_json::Error> = serde_json::from_slice(&json);
        match parsed {
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
        }
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
