use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Testcase {
    pub day: u32,
    pub part: u32,
    pub args: Option<String>,
    pub solution: String,
    pub input: String,
}

pub fn read_all_testcases() -> Vec<Testcase> {
    let toml_str = include_str!("../testcases.toml");
    let testcases: HashMap<String, Vec<Testcase>> = toml::from_str(toml_str).unwrap();

    let testcases: Vec<Testcase> = testcases.get("testcases").unwrap().to_vec();
    testcases
}
