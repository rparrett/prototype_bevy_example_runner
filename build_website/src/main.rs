use std::collections::HashSet;
use tera::{Context, Tera};

fn main() {
    let runs = metadata::load(10);

    let mut all_examples: HashSet<String> = HashSet::new();
    for run in runs.iter() {
        for (example, _) in run.results.iter() {
            all_examples.insert(example.clone());
        }
    }

    let mut context = Context::new();
    context.insert("runs".to_string(), &runs);
    context.insert("examples".to_string(), &all_examples);

    let rendered = Tera::one_off(
        &std::fs::read_to_string("./build_website/templates/index.html").unwrap(),
        &context,
        true,
    )
    .unwrap();

    std::fs::write("index.html", &rendered).unwrap();
}
