pub fn clean_column_name(name: &str) -> String {
    let mut cleaned_name = String::new();
    let mut prev_char_is_upper = false;

    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_char_is_upper {
                cleaned_name.push('_');
            }
            cleaned_name.push(c.to_ascii_lowercase());
        } else if c.is_alphanumeric() {
            cleaned_name.push(c);
        } else {
            cleaned_name.push('_');
        }
        prev_char_is_upper = c.is_uppercase();
    }

    cleaned_name
        .trim_matches(|c| c == '_')
        .replace(r"__+", "_")
        .to_lowercase()
}