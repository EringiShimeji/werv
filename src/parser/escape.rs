use super::error::ParseError;

pub fn escape(str: &str) -> Result<String, ParseError> {
    let mut result = String::new();
    let mut chars = str.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(c) = chars.next() {
                if c == 'n' {
                    result.push('\n');
                } else {
                    result.push(c);
                }

                continue;
            }
            return Err(ParseError::ParseEscapeError);
        }

        result.push(c);
    }

    Ok(result)
}
