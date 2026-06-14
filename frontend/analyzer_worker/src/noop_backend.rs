use crate::{CodeAnalyzer, CompletionItem, Diagnostic};

pub struct NoopBackend;

impl NoopBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoopBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeAnalyzer for NoopBackend {
    fn update_file(&mut self, _text: String) -> Vec<Diagnostic> {
        Vec::new()
    }

    fn completions(&self, _line: u32, _col: u32) -> Vec<CompletionItem> {
        Vec::new()
    }
}
