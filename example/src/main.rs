use anyhow::Result;
use serde_json::{Value, json};

fn add(args: Value) -> Value {
    let res = args.get("a").unwrap().as_i64().unwrap() + args.get("b").unwrap().as_i64().unwrap();

    json!(res)
}

fn main() {
    println!("Hello, world!");
}

#[allow(unused)]
fn run(action: &str, args: Value) -> Result<Value> {
    match action {
        "add" => Ok(add(args)),
        _ => panic!("Unimplemented"),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use json_to_test::generate_tests_from_json;

    json2tests!("tests.json");
}
