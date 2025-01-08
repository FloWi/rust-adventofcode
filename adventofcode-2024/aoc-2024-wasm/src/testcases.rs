use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct Testcase {
    pub day: u32,
    pub part: u32,
    pub arg: Option<String>,
    pub solution: String,
    pub input: String,
}

pub fn read_all_testcases() -> Vec<Testcase> {
    let toml_str = include_str!("../testcases.toml");
    let testcases: HashMap<String, Vec<Testcase>> = toml::from_str(toml_str).unwrap();

    let testcases: Vec<Testcase> = testcases
        .get("testcases")
        .unwrap()
        .iter()
        .cloned()
        .collect();
    testcases
}
