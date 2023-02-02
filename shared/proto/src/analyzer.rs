use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Diagnostic {
    pub message: String,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct CompletionItem {
    pub label: String,
    pub kind: i64,
    pub detail: String,
    pub insertText: String,
    pub insertTextRules: u32,
    pub filterText: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Diagnostics(String),
    Completion(u32, u32),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Diagnostics(Vec<Diagnostic>),
    Completion(Vec<CompletionItem>),
}
