use chrono::prelude::*;
use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ExampleResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}
#[derive(Serialize, Deserialize, Default, Debug)]

pub struct Run {
    pub commit_hash: String,
    pub commit_message: String,
    pub results: HashMap<String, ExampleResult>,
}

impl Run {
    pub fn save(&self) -> std::io::Result<()> {
        std::fs::create_dir_all("./results")?;

        let now: DateTime<Utc> = Utc::now();
        let filename = format!("./results/{}.json", now.format("%Y%m%d%H%I%S"));

        std::fs::write(
            filename,
            serde_json::to_string(&self).expect("Failed to serialize results."),
        )
    }
}

pub fn load(num: usize) -> Vec<Run> {
    let paths = std::fs::read_dir("./results").unwrap();

    let mut paths: Vec<_> = paths
        .into_iter()
        .filter_map(|maybe_entry| maybe_entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map_or(false, |ext| ext == "json"))
        .collect();

    // sort by newest first
    paths.sort_by(|a, b| a.cmp(&b));

    paths
        .iter()
        .take(num)
        .map(|p| {
            let json = std::fs::read_to_string(p).unwrap();
            serde_json::from_str(&json).expect(&format!("Failed to deserialize run data: {:?}", p))
        })
        .collect()
}
