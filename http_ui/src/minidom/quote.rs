///
/// Quotes text for HTML display
///
pub fn quote_text(text: &String) -> String {
    let mut result = String::new();

    for c in text.chars() {
        match c {
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '&' => result.push_str("&amp;"),
            '\'' => result.push_str("&quot;"),

            _ => result.push(c)
        }
    }

    result
}
