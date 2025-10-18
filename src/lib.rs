use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::{
    fs::{self},
    path::PathBuf,
};
use syn::{LitStr, parse_macro_input};
use uuid::Uuid;

mod types;

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
        let test_ident: syn::Ident;
        if Uuid::parse_str(&test_name).is_ok() {
            test_ident = format_ident!("uuid_{}", test_name.replace("-", "_"));
        } else {
            test_ident = format_ident!("{}", test_name);
        }

        let action = LitStr::new(&test_data.action, test_ident.span());
        let arguments = LitStr::new(&test_data.arguments.to_string(), test_ident.span());
        let compare_results = test_data.result.map(|r| {
            let result_lit = LitStr::new(&r.to_string(), test_ident.span());

            quote! {
                let expected_result: serde_json::Value = serde_json::from_str(#result_lit).unwrap();
                ::assert_json_diff::assert_json_eq!(result.unwrap(), expected_result)
            }
        });

        let test_code = quote! {
            #[test]
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
