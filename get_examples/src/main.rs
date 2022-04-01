use serde_derive::Deserialize;
use std::io::Write;
use std::process::Command;
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

fn get_current_commit_string() -> String {
    let output = Command::new("git")
        .current_dir(std::fs::canonicalize("../bevy").unwrap())
        .args(["log", "--oneline", "-n", "1"])
        .output()
        .expect("failed to execute process");

    String::from_utf8(output.stdout).expect("Failed to parse git output")
}

fn main() {
    let ignore = ["android", "custom_loop"];

    let toml_str = fs::read_to_string("../bevy/Cargo.toml").unwrap();

    let decoded: Cargo = toml::from_str(&toml_str).unwrap();

    let mut table_lines = vec![];
    table_lines.push("## Commit Tested".to_string());
    table_lines.push(get_current_commit_string());
    table_lines.push("## Results ".to_string());
    table_lines.push("|example|status|".to_string());
    table_lines.push("|-|-|".to_string());

    for example in decoded.example.iter().take(3) {
        if ignore.iter().any(|i| example.path.contains(i)) {
            continue;
        }

        let example_config = format!("../config/{}.ron", example.name);
        let config = if std::path::Path::new(&example_config).exists() {
            example_config
        } else {
            "../config/default.ron".to_string()
        };

        let output = Command::new("xvfb-run")
            .current_dir(std::fs::canonicalize("../bevy").unwrap())
            .env("CI_TESTING_CONFIG", config)
            .args([
                "cargo",
                "run",
                "--example",
                &example.name,
                "--features=x11,bevy_ci_testing",
            ])
            .output()
            .expect("failed to execute process");

        println!("{} {:?}", example.name, output.status);
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();

        let status_string = if output.status.success() {
            ":white_check_mark:".to_string()
        } else {
            format!(":x: (Code {})", output.status.code().unwrap())
        };

        table_lines.push(format!("|{}|{}|", example.name, status_string));
    }

    std::fs::write("../README.md", table_lines.join("\n")).expect("Failed to write readme");
}
