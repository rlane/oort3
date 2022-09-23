pub fn format(input: &str) -> String {
    match syn::parse_file(input) {
        Ok(syntax_tree) => prettyplease::unparse(&syntax_tree),
        Err(e) => {
            log::error!("Failed to format code: {:?}", e);
            input.to_string()
        }
    }
}
