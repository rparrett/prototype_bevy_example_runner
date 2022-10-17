use std::collections::HashSet;
use tera::{Context, Tera};

fn main() {
    let runs = metadata::load(30);

    let mut num_fails = vec![];

    let mut all_examples: HashSet<String> = HashSet::new();
    for run in runs.iter() {
        let mut fails = 0;

        for (example, result) in run.results.iter() {
            if result.code != 0 {
                fails += 1;
            }

            all_examples.insert(example.clone());
        }

        num_fails.push(fails);
    }

    let mut context = Context::new();
    context.insert("runs".to_string(), &runs);
    context.insert("num_fails".to_string(), &num_fails);
    context.insert("examples".to_string(), &all_examples);

    let rendered = Tera::one_off(
        &std::fs::read_to_string("./build_website/templates/index.html").unwrap(),
        &context,
        true,
    )
    .unwrap();

    std::fs::create_dir_all("./out").unwrap();

    std::fs::write("./out/index.html", &rendered).unwrap();
}
