# `json2tests` — Generate Rust Tests from a Given JSON File

The macro provided in this crate allows for generating test cases from a given JSON file following the schema defined in [schema.json](schema.json).

The [example](example) crate shows how to implement the macro in your source code. Notice how you can just run `cargo test` in this directory and the test from the JSON file is executed.

## Requirements

Add the following to your `Cargo.toml` (if not already in it):

```toml
[packages]
assert-json-diff = "2.0"
json_to_test = { version = "0.1.0", path = "YOUR_PATH" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

Please note that this macro is not yet available via `cargo add`, you need to clone the repository and specify the local path!

## Usage

Add the following to your source code:

```rs
#[cfg(test)]
mod test {
    use super::*;
    use json_to_test::generate_tests_from_json;

    generate_tests_from_json!(file_path)
}
```

Generate tests from JSON takes a file path as input and will invoke a `run()` function for each testcase contained in the JSON. Your `run` function needs to be defined as follows:

```rs
fn run(action: &str, args: serde_json::Value) -> Result<serde_json::Value, impl std::err::Error>;
```

> [!NOTE]
> Cargo will not detect if you change something on your test files. I therefore recommend to add a `build.rs` file declaring a dependency on your JSON file(s), to ensure a recompilation when you change your tests.
>
> Example:
>
> ```rs
> fn main() {
>     println!("cargo:rerun-if-changed=tests.json");
> }
> ```

## Example

This JSON script

```json
{
  "testcases": {
    "test_addition": {
      "action": "add",
      "arguments": {
        "a": 2,
        "b": 3
      },
      "result": 5
    }
  }
}
```

will generate the following code, when implemented as described in [Usage](#usage):

```rs
mod test {
    use super::*;
    use json_to_test::generate_tests_from_json;
    extern crate test;
    #[rustc_test_marker = "test::test_addition"]
    #[doc(hidden)]
    pub const test_addition: test::TestDescAndFn = test::TestDescAndFn {
        // Snip
    };
    fn test_addition() {
        let value = serde_json::from_str("{\"a\":2,\"b\":3}").unwrap();
        let result = run("add", value);
        if !result.is_ok() {
            ::core::panicking::panic("assertion failed: result.is_ok()")
        }
        let expected_result: serde_json::Value = serde_json::from_str("5").unwrap();
        {
            {
                if let Err(error) = ::assert_json_diff::assert_json_matches_no_panic(
                    &result.unwrap(),
                    &expected_result,
                    ::assert_json_diff::Config::new(
                        ::assert_json_diff::CompareMode::Strict,
                    ),
                ) {
                    {
                        ::std::rt::panic_fmt(format_args!("\n\n{0}\n\n", error));
                    };
                }
            }
        }
    }
}
```

## You (probably)

![Meme stating "I'm a high efficiency man for a high speed age"](img/efficient_meme.png)

(This shall include all Rustaceans equally, without regard to gender :crab:).

Happy coding (and testing)! :rocket:
