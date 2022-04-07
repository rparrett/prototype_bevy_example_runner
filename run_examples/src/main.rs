use clap::Parser;
use metadata::*;
use serde_derive::Deserialize;
use std::collections::VecDeque;
use std::fs::remove_dir_all;
use std::io::Write;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use std::{fs, io};

#[derive(Debug, Deserialize)]
struct Example {
    name: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct Cargo {
    example: Vec<Example>,
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(long)]
    xvfb: bool,
    #[clap(short, long)]
    num_examples: Option<usize>,
}

fn get_current_commit() -> (String, String) {
    let output = Command::new("git")
        .current_dir(std::fs::canonicalize("./bevy").unwrap())
        .args(["log", "--pretty=oneline", "-n", "1"])
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8(output.stdout).expect("Failed to parse git output");

    let split = stdout.split_once(' ').unwrap();

    (split.0.to_string(), split.1.to_string())
}

fn main() {
    let args = Args::parse();

    let ignore = ["android", "custom_loop"];

    let toml_str = fs::read_to_string("./bevy/Cargo.toml").unwrap();

    let decoded: Cargo = toml::from_str(&toml_str).unwrap();

    let mut run = Run::default();

    let commit = get_current_commit();
    run.commit_hash = commit.0;
    run.commit_message = commit.1;

    let mut n = 0;

    for example in decoded.example.iter() {
        if ignore.iter().any(|i| example.path.contains(i)) {
            continue;
        }

        let example_config = format!("../config/{}.ron", example.name);
        let config = if std::path::Path::new(&example_config).exists() {
            example_config
        } else {
            "../config/default.ron".to_string()
        };

        let mut cmd_args = VecDeque::from([
            "run",
            "--example",
            &example.name,
            "--features=x11,bevy_ci_testing",
        ]);
        if args.xvfb {
            cmd_args.push_front("cargo");
        }
        let command = if args.xvfb { "xvfb-run" } else { "cargo" };

        let output = Command::new(command)
            .current_dir(std::fs::canonicalize("./bevy").unwrap())
            .env("CI_TESTING_CONFIG", config)
            .args(cmd_args)
            .output()
            .expect(&format!("failed to execute {}", command));

        println!("{} {:?}", example.name, output.status);

        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();

        // Mysterious linker errors after 30 or so examples run. I suspect this is a disk space thing,
        // because if we clean up after ourselves, the issue seems to go away.
        remove_dir_all("./bevy/target/debug/examples").expect("Failed to clean up after ourselves");

        run.results.insert(
            example.name.clone(),
            ExampleResult {
                code: output.status.code().unwrap(),
                stdout: String::from_utf8(output.stdout.into()).unwrap_or_else(|_| "".to_string()),
                stderr: String::from_utf8(output.stderr.into()).unwrap_or_else(|_| "".to_string()),
            },
        );

        // xvfb needs some time to shut down properly, or we get intermittent
        // failures
        sleep(Duration::from_secs(10));

        n += 1;

        if let Some(num) = args.num_examples {
            if n >= num {
                break;
            }
        }
    }

    run.save().unwrap();
}
