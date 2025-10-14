use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct TestCase {
    pub action: String,
    pub arguments: Value,
    pub result: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct TestCases {
    pub testcases: std::collections::HashMap<String, TestCase>,
}
