//! Tokenizer: converts source lines into tokens and resolves values.

use crate::error::{InterpError, InterpResult};
use crate::utils;
use crate::value::Value;
use std::collections::HashMap;

pub struct Tokenizer {}

impl Tokenizer {
    /// Splits a line into tokens, handling strings and comments.
    /// Returns an error on unclosed string literals or invalid escapes.
    pub fn tokenize(line: &str, line_num: usize) -> InterpResult<Vec<String>> {
        let mut tokens: Vec<String> = Vec::new();
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch.is_whitespace() {
                continue;
            }
            if ch == '/' && chars.peek() == Some(&'/') {
                break;
            }
            if ch == '"' {
                let mut content = String::new();
                let mut closed = false;

                while let Some(c) = chars.next() {
                    if c == '"' {
                        let mut backslash_count = 0;
                        for ch in content.chars().rev() {
                            if ch == '\\' {
                                backslash_count += 1;
                            } else {
                                break;
                            }
                        }
                        if backslash_count % 2 == 0 {
                            closed = true;
                            break;
                        }
                        content.push(c);
                    } else {
                        content.push(c);
                    }
                }

                if !closed {
                    return Err(InterpError::Syntax {
                        line: line_num,
                        message: "Unclosed string literal".to_string(),
                    });
                }

                let unescaped = Self::unescape_string(&content, line_num)?;
                tokens.push(unescaped);
            } else {
                let mut token = String::new();
                token.push(ch);
                while let Some(&next) = chars.peek() {
                    if next.is_whitespace() || next == '"' {
                        break;
                    }
                    token.push(chars.next().unwrap());
                }
                tokens.push(token);
            }
        }
        Ok(tokens)
    }

    /// Resolves a token to a Value: SP (stack top), variable (local/global), or literal.
    pub fn resolve_value(
        token: &str,
        stack: &[Value],
        globals: &HashMap<String, Value>,
        locals: Option<&HashMap<String, Value>>,
    ) -> InterpResult<Value> {
        if token == "SP" {
            return stack
                .last()
                .cloned()
                .ok_or(InterpError::Internal("Stack is empty".to_string()));
        }
        if let Some(locals) = locals
            && let Some(v) = locals.get(token)
        {
            return Ok(v.clone());
        }
        if let Some(v) = globals.get(token) {
            return Ok(v.clone());
        }

        Ok(utils::parse(token))
    }

    /// Unescapes a string literal content (without surrounding quotes).
    /// Returns an error on invalid Unicode escape sequences.
    pub(crate) fn unescape_string(raw: &str, line: usize) -> InterpResult<String> {
        let mut result = String::new();
        let mut chars = raw.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(next) = chars.next() {
                    match next {
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        '\\' => result.push('\\'),
                        '"' => result.push('"'),
                        'u' => {
                            if chars.next() != Some('{') {
                                return Err(InterpError::Syntax {
                                    line,
                                    message: "Expected '{' after \\u".to_string(),
                                });
                            }
                            let mut hex = String::new();
                            let mut closed = false;
                            while let Some(&ch) = chars.peek() {
                                if ch == '}' {
                                    chars.next();
                                    closed = true;
                                    break;
                                }
                                if !ch.is_ascii_hexdigit() {
                                    return Err(InterpError::Syntax {
                                        line,
                                        message: format!(
                                            "Invalid hex digit in \\u{{...}}: '{}'",
                                            ch
                                        ),
                                    });
                                }
                                hex.push(ch);
                                chars.next();
                            }
                            if !closed {
                                return Err(InterpError::Syntax {
                                    line,
                                    message: "Unclosed \\u{{...}} sequence".to_string(),
                                });
                            }
                            if hex.is_empty() {
                                return Err(InterpError::Syntax {
                                    line,
                                    message: "Empty Unicode code point in \\u{{...}}".to_string(),
                                });
                            }
                            match u32::from_str_radix(&hex, 16) {
                                Ok(codepoint) => {
                                    if let Some(ch) = std::char::from_u32(codepoint) {
                                        result.push(ch);
                                    } else {
                                        return Err(InterpError::Syntax {
                                            line,
                                            message: format!(
                                                "Invalid Unicode code point: U+{}",
                                                hex
                                            ),
                                        });
                                    }
                                }
                                Err(_) => {
                                    return Err(InterpError::Syntax {
                                        line,
                                        message: format!("Invalid hex number: {}", hex),
                                    });
                                }
                            }
                        }
                        _ => {
                            result.push('\\');
                            result.push(next);
                        }
                    }
                } else {
                    return Err(InterpError::Syntax {
                        line,
                        message: "Unexpected end of escape sequence".to_string(),
                    });
                }
            } else {
                result.push(c);
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::InterpError;

    // ---------- tokenize ----------
    #[test]
    fn tokenize_simple_tokens() {
        let line = "PUSH 42 HELLO";
        let tokens = Tokenizer::tokenize(line, 1).unwrap();
        assert_eq!(tokens, vec!["PUSH", "42", "HELLO"]);
    }

    #[test]
    fn tokenize_comment() {
        let line = "PUSH 42 // комментарий";
        let tokens = Tokenizer::tokenize(line, 1).unwrap();
        assert_eq!(tokens, vec!["PUSH", "42"]);
    }

    #[test]
    fn tokenize_string_literal() {
        let line = r#"PRINT "Hello, world!""#;
        let tokens = Tokenizer::tokenize(line, 1).unwrap();
        assert_eq!(tokens, vec!["PRINT", "Hello, world!"]);
    }

    #[test]
    fn tokenize_string_with_escaped_quote() {
        let line = "PRINT \"She said \\\"Hi\\\"\"";
        let tokens = Tokenizer::tokenize(line, 1).unwrap();
        assert_eq!(tokens[0], "PRINT");
        assert_eq!(tokens[1], "She said \"Hi\"");
    }

    #[test]
    fn tokenize_string_with_escapes() {
        let line = "PRINT \"Line1\\nLine2\\tTab\"";
        let tokens = Tokenizer::tokenize(line, 1).unwrap();
        assert_eq!(tokens[0], "PRINT");
        assert_eq!(tokens[1].len(), "Line1\nLine2\tTab".len());
        assert!(tokens[1].contains('\n'));
        assert!(tokens[1].contains('\t'));
    }

    #[test]
    fn tokenize_unicode_escape() {
        let line = "PRINT \"\\u{1F600}\"";
        let tokens = Tokenizer::tokenize(line, 1).unwrap();
        assert_eq!(tokens[0], "PRINT");
        assert_eq!(tokens[1], "😀");
    }

    #[test]
    fn tokenize_unclosed_string() {
        let line = "PRINT \"Hello";
        let result = Tokenizer::tokenize(line, 1);
        assert!(result.is_err());
        match result.unwrap_err() {
            InterpError::Syntax { line, message } => {
                assert_eq!(line, 1);
                assert!(message.contains("Unclosed string literal"));
            }
            _ => panic!("Expected Syntax error"),
        }
    }

    #[test]
    fn tokenize_invalid_escape_sequence() {
        let line = "PRINT \"\\u{ZZZ}\"";
        let result = Tokenizer::tokenize(line, 1);
        assert!(result.is_err());
        match result.unwrap_err() {
            InterpError::Syntax { line, message } => {
                assert_eq!(line, 1);
                assert!(message.contains("Invalid hex digit"));
            }
            _ => panic!("Expected Syntax error"),
        }
    }

    #[test]
    fn tokenize_invalid_unicode_code_point() {
        let line = "PRINT \"\\u{110000}\"";
        let result = Tokenizer::tokenize(line, 1);
        assert!(result.is_err());
        match result.unwrap_err() {
            InterpError::Syntax { line, message } => {
                assert_eq!(line, 1);
                assert!(message.contains("Invalid Unicode code point"));
            }
            _ => panic!("Expected Syntax error"),
        }
    }

    // ---------- unescape_string ----------
    #[test]
    fn unescape_string_basic() {
        let raw = "Hello\\nWorld\\t!";
        let result = Tokenizer::unescape_string(raw, 1).unwrap();
        assert_eq!(result, "Hello\nWorld\t!");
    }

    #[test]
    fn unescape_string_escaped_slash() {
        let raw = "C:\\\\Users\\\\name";
        let result = Tokenizer::unescape_string(raw, 1).unwrap();
        assert_eq!(result, r"C:\Users\name");
    }

    #[test]
    fn unescape_string_escaped_quote() {
        let raw = "She said \\\"Hi\\\"";
        let result = Tokenizer::unescape_string(raw, 1).unwrap();
        assert_eq!(result, "She said \"Hi\"");
    }

    #[test]
    fn unescape_string_unicode_simple() {
        let raw = "Smile \\u{1F600}!";
        let result = Tokenizer::unescape_string(raw, 1).unwrap();
        assert_eq!(result, "Smile 😀!");
    }

    #[test]
    fn unescape_string_unicode_invalid_hex() {
        let raw = "Bad \\u{G}!";
        let result = Tokenizer::unescape_string(raw, 1);
        assert!(result.is_err());
        match result.unwrap_err() {
            InterpError::Syntax { line, message } => {
                assert_eq!(line, 1);
                assert!(message.contains("Invalid hex digit"));
            }
            _ => panic!("Expected Syntax error"),
        }
    }

    #[test]
    fn unescape_string_unicode_unclosed() {
        let raw = "Unclosed \\u{123";
        let result = Tokenizer::unescape_string(raw, 1);
        assert!(result.is_err());
        match result.unwrap_err() {
            InterpError::Syntax { line, message } => {
                assert_eq!(line, 1);
                assert!(message.contains("Unclosed"));
            }
            _ => panic!("Expected Syntax error"),
        }
    }

    #[test]
    fn unescape_string_unicode_empty() {
        let raw = "Empty \\u{}";
        let result = Tokenizer::unescape_string(raw, 1);
        assert!(result.is_err());
        match result.unwrap_err() {
            InterpError::Syntax { line, message } => {
                assert_eq!(line, 1);
                assert!(message.contains("Empty Unicode code point"));
            }
            _ => panic!("Expected Syntax error"),
        }
    }
}
