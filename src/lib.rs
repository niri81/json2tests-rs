use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::{
    fs::{self},
    path::PathBuf,
};
use syn::{LitStr, parse_macro_input};
use uuid::Uuid;

mod types;

/// Generates tests from JSON file specified as argument (relative to
/// `Cargo.toml`). The JSON must match the [predefined schema](https://raw.githubusercontent.com/niri81/json2tests-rs/refs/heads/main/schema.json).
///
/// This macro will always invoke a `run` function requiring the following
/// specification, passing the action and all provided arguments:
/// ```ignore
/// fn run(action: &str, args: serde_json::Value) -> Result<serde_json::Value, impl std::error::Error>;
/// ```
/// The returned JSON value must then be `Ok` and match the expected result from
/// the provided JSON file.
///
/// # Usage
///
/// ```ignore
/// #[cfg(test)]
/// mod test {
///     use super::*;
///     use json2tests::json2tests;
///
///     json2tests!("examples/default_tests.json");
/// }
/// ```
///
/// For further information, check the [GitHub repository](https://github.com/niri81/json2tests-rs/).
#[proc_macro]
pub fn json2tests(input: TokenStream) -> TokenStream {
    let json_path = PathBuf::from(parse_macro_input!(input as LitStr).value());

    if !json_path.is_file() {
        let error_msg = format!("Provided path {} is no file", json_path.display());
        return quote! {compile_error!(#error_msg)}.into();
    }

    let mut generated_tests = Vec::new();

    let data = match fs::read_to_string(&json_path) {
        Ok(v) => v,
        Err(e) => {
            let error_msg = format!(
                "Could not read JSON file {}. Error: {e}",
                json_path.display()
            );
            return quote! {compile_error!(#error_msg)}.into();
        }
    };

    let json: types::TestCases = match serde_json::from_str(&data) {
        Ok(v) => v,
        Err(e) => {
            let error_msg = format!(
                "Could not parse JSON file {}. Error: {e}",
                json_path.display()
            );
            return quote! {compile_error!(#error_msg)}.into();
        }
    };

    for (test_name, test_data) in json.testcases {
        let test_ident = if Uuid::parse_str(&test_name).is_ok() {
            format_ident!("uuid_{}", test_name.replace('-', "_"))
        } else {
            format_ident!("{}", test_name.replace('-', "_"))
        };

        let action = LitStr::new(&test_data.action, test_ident.span());
        let arguments = LitStr::new(&test_data.arguments.to_string(), test_ident.span());
        let compare_results = test_data.result.map(|r| {
            let result_lit = LitStr::new(&r.to_string(), test_ident.span());

            quote! {
                let expected_result: serde_json::Value = serde_json::from_str(#result_lit).unwrap();
                ::assert_json_diff::assert_json_eq!(result.unwrap(), expected_result)
            }
        });
        let should_panic = test_data.panic.map(|v| {
            if let Some(b) = v.as_bool()
                && b
            {
                quote! {
                    #[should_panic]
                }
            } else if let Some(s) = v.as_str() {
                let msg = LitStr::new(s, test_ident.span());

                quote! {
                    #[should_panic(expected = #msg)]
                }
            } else {
                quote! {}
            }
        });

        let test_code = quote! {
            #[test]
            #should_panic
            fn #test_ident() {
                let value = serde_json::from_str(#arguments).unwrap();

                let result = run(#action, value);
                assert!(result.is_ok());

                #compare_results
            }
        };

        generated_tests.push(test_code);
    }

    let output = quote! {
        #(#generated_tests)*
    };

    output.into()
}
