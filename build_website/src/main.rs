use metadata::ExampleResult;
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    hash::BuildHasher,
};
use tera::{to_value, try_get_value, Context, Tera, Value};

#[derive(Serialize)]
enum FailureType {
    None,
    Run,
    Compile,
}
impl From<&ExampleResult> for FailureType {
    fn from(result: &ExampleResult) -> Self {
        if result.code != 0 {
            return Self::Compile;
        }

        if result.stdout.contains("ERROR\x1b[0m \x1b[2mbevy") {
            return Self::Run;
        }

        Self::None
    }
}

pub fn failure_type<S: BuildHasher>(
    value: &Value,
    _: &HashMap<String, Value, S>,
) -> tera::Result<Value> {
    let res = try_get_value!("failure_type", "value", ExampleResult, value);
    Ok(to_value(&FailureType::from(&res)).unwrap())
}

fn main() {
    let runs = metadata::load(30);

    let mut num_fails = vec![];

    let mut all_examples: HashSet<String> = HashSet::new();
    for run in runs.iter() {
        let mut fails = 0;

        for (example, result) in run.results.iter() {
            if !matches!(FailureType::from(result), FailureType::None) {
                fails += 1;
            }

            all_examples.insert(example.clone());
        }

        num_fails.push(fails);
    }

    let mut context = Context::new();
    context.insert("runs".to_string(), &runs);
    context.insert("num_fails".to_string(), &num_fails);
    context.insert("all_examples".to_string(), &all_examples);

    let mut tera = Tera::default();
    tera.register_filter("failure_type", failure_type);
    // TODO not clear why tera can't find the template
    tera.add_raw_template(
        "index.html",
        &std::fs::read_to_string("./build_website/templates/index.html").unwrap(),
    )
    .unwrap();
    let rendered = tera.render("index.html", &context).unwrap();

    std::fs::create_dir_all("./out").unwrap();

    std::fs::write("./out/index.html", &rendered).unwrap();
}
