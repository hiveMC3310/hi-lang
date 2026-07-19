//! Tokenizer: converts source lines into tokens and resolves values.

use crate::error::{InterpError, InterpResult};
use crate::utils;
use crate::value::Value;
use std::collections::HashMap;

pub struct Tokenizer {}

impl Tokenizer {
    /// Splits a line into tokens, handling strings and comments.
    /// Returns an error on unclosed string literals.
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
                let mut s = String::new();
                let mut closed = false;
                while let Some(c) = chars.next() {
                    if c == '"' && !s.ends_with('\\') {
                        closed = true;
                        break;
                    }
                    s.push(c);
                }
                if !closed {
                    return Err(InterpError::Syntax {
                        line: line_num,
                        message: "Unclosed string literal".to_string(),
                    });
                }
                tokens.push(s);
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
}
