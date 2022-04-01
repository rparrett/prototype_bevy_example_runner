use serde_derive::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct Example {
    name: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct Cargo {
    example: Vec<Example>,
}

fn main() {
    let ignore = ["android"];

    let toml_str = fs::read_to_string("../bevy/Cargo.toml").unwrap();

    let decoded: Cargo = toml::from_str(&toml_str).unwrap();

    for example in decoded.example {
        if ignore.iter().any(|i| example.path.contains(i)) {
            continue;
        }

        println!("{}", example.name);
    }
}
