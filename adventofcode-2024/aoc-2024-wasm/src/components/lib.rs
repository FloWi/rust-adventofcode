use regex::Regex;
use send_wrapper::SendWrapper;
use serde::{Deserialize, Serialize};
use web_sys::File;

#[derive(Default, Debug, Deserialize, Serialize, Eq, PartialEq, Clone, Hash)]
pub struct AocDayInput {
    pub day: u32,
    pub input: String,
}

#[derive(Default, Debug, Deserialize, Serialize, Eq, PartialEq, Clone)]
pub struct AocInput {
    pub(crate) days: Vec<AocDayInput>,
}

pub async fn read_file_content(file: &SendWrapper<File>) -> String {
    let text_blob = file.text();
    (async move { wasm_bindgen_futures::JsFuture::from(text_blob).await.unwrap().as_string().unwrap() }).await
}

pub fn parse_day_from_str(filename: &str) -> Option<u32> {
    let re = Regex::new(r"\d+").unwrap();
    re.find(filename)?.as_str().parse().ok()
}
