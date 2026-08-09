//! Lexer for the Hi language, tokenizes source code.

use crate::ast::Span;
use crate::error::LexError;
use hi_common::Symbol;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Let,
    Input,
    If,
    Then,
    Else,
    End,
    While,
    For,
    To,
    Next,
    Do,
    Func,
    Ret,
    Break,
    Print,
    Import,
    As,
    True,
    False,

    Ident(Symbol),

    Int(i64),
    Float(f64),

    String(String),

    Plus,          // +
    Minus,         // -
    Star,          // *
    Slash,         // /
    Percent,       // %
    Caret,         // ^
    PlusAssign,    // +=
    MinusAssign,   // -=
    StarAssign,    // *=
    SlashAssign,   // /=
    PercentAssign, // %=
    CaretAssign,   // ^=
    EqEq,          // ==
    Neq,           // !=
    Gt,            // >
    Ge,            // >=
    Lt,            // <
    Le,            // <=
    And,           // AND
    Or,            // OR
    Not,           // NOT or !
    Assign,        // =
    LParen,        // (
    RParen,        // )
    LBrace,        // {
    RBrace,        // }
    LBracket,      // [
    RBracket,      // ]
    Comma,         // ,
    Colon,         // :

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub doc: Option<String>,
}

pub struct Lexer {
    pending_doc: Option<String>,
}

impl Lexer {
    pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
        let mut lexer = Lexer { pending_doc: None };

        let mut tokens = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let mut pos = 0;
        let mut line = 1;
        let mut col = 1;

        while pos < chars.len() {
            let ch = chars[pos];
            let start_col = col;
            let start_line = line;

            // Whitespace
            if ch.is_whitespace() {
                if ch == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
                pos += 1;
                continue;
            }

            // Comments
            if ch == '/' && pos + 1 < chars.len() && chars[pos + 1] == '/' {
                if pos + 2 < chars.len() && chars[pos + 2] == '/' {
                    // doc
                    let mut doc_line = String::new();
                    pos += 3;
                    col += 3;
                    while pos < chars.len() && chars[pos] != '\n' {
                        doc_line.push(chars[pos]);
                        pos += 1;
                        col += 1;
                    }
                    if let Some(ref mut existing) = lexer.pending_doc {
                        existing.push('\n');
                        existing.push_str(&doc_line);
                    } else {
                        lexer.pending_doc = Some(doc_line);
                    }
                    continue;
                } else {
                    while pos < chars.len() && chars[pos] != '\n' {
                        pos += 1;
                        col += 1;
                    }
                    continue;
                }
            }

            // Strings
            if ch == '"' {
                let mut content = String::new();
                pos += 1;
                col += 1;
                let mut closed = false;
                while pos < chars.len() {
                    let c = chars[pos];
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
                            pos += 1;
                            col += 1;
                            break;
                        }
                        content.push(c);
                        pos += 1;
                        col += 1;
                    } else {
                        content.push(c);
                        pos += 1;
                        col += 1;
                    }
                }
                if !closed {
                    return Err(LexError {
                        message: "Unclosed string literal".to_string(),
                        span: Span {
                            start_line,
                            start_col,
                            end_line: line,
                            end_col: col,
                        },
                    });
                }

                let string_span = Span {
                    start_line,
                    start_col,
                    end_line: line,
                    end_col: col,
                };
                let unescaped = Lexer::unescape_string(&content, string_span)?;
                tokens.push(lexer.token(
                    TokenKind::String(unescaped),
                    start_line,
                    start_col,
                    line,
                    col,
                ));
                continue;
            }

            // Numbers
            if ch.is_ascii_digit()
                || (ch == '.' && pos + 1 < chars.len() && chars[pos + 1].is_ascii_digit())
            {
                let mut num_str = String::new();
                let mut is_float = false;

                if ch == '.' {
                    num_str.push('0');
                    num_str.push('.');
                    pos += 1;
                    col += 1;
                    is_float = true;
                }

                while pos < chars.len() && chars[pos].is_ascii_digit() {
                    num_str.push(chars[pos]);
                    pos += 1;
                    col += 1;
                }

                if pos < chars.len() && chars[pos] == '.' {
                    if is_float {
                        return Err(LexError {
                            message: "Multiple decimal points in number".to_string(),
                            span: Span {
                                start_line,
                                start_col,
                                end_line: line,
                                end_col: col + 1,
                            },
                        });
                    }
                    is_float = true;
                    num_str.push('.');
                    pos += 1;
                    col += 1;

                    let mut frac_digits = 0;
                    while pos < chars.len() && chars[pos].is_ascii_digit() {
                        num_str.push(chars[pos]);
                        frac_digits += 1;
                        pos += 1;
                        col += 1;
                    }

                    if frac_digits == 0 {
                        num_str.push('0');
                    }
                }

                // Check for invalid trailing characters (letters or underscore)
                if pos < chars.len() && (chars[pos].is_ascii_alphabetic() || chars[pos] == '_') {
                    return Err(LexError {
                        message: "Invalid number: trailing characters after number".to_string(),
                        span: Span {
                            start_line,
                            start_col,
                            end_line: line,
                            end_col: col + 1,
                        },
                    });
                }

                // Disallow a second decimal point (e.g. "1.2.3")
                if pos < chars.len() && chars[pos] == '.' {
                    return Err(LexError {
                        message: "Multiple decimal points in number".to_string(),
                        span: Span {
                            start_line,
                            start_col,
                            end_line: line,
                            end_col: col + 1,
                        },
                    });
                }

                if is_float {
                    match num_str.parse::<f64>() {
                        Ok(f) => tokens.push(lexer.token(
                            TokenKind::Float(f),
                            start_line,
                            start_col,
                            line,
                            col,
                        )),
                        Err(_) => {
                            return Err(LexError {
                                message: format!("Invalid float literal: {}", num_str),
                                span: Span {
                                    start_line,
                                    start_col,
                                    end_line: line,
                                    end_col: col,
                                },
                            });
                        }
                    }
                } else {
                    match num_str.parse::<i64>() {
                        Ok(i) => tokens.push(lexer.token(
                            TokenKind::Int(i),
                            start_line,
                            start_col,
                            line,
                            col,
                        )),
                        Err(_) => {
                            return Err(LexError {
                                message: format!("Invalid integer literal: {}", num_str),
                                span: Span {
                                    start_line,
                                    start_col,
                                    end_line: line,
                                    end_col: col,
                                },
                            });
                        }
                    }
                }
                continue;
            }

            // Identifiers and keywords
            if ch.is_ascii_alphabetic() || ch == '_' {
                let mut ident = String::new();
                while pos < chars.len() && (chars[pos].is_ascii_alphanumeric() || chars[pos] == '_')
                {
                    ident.push(chars[pos]);
                    pos += 1;
                    col += 1;
                }
                let kind = match ident.as_str() {
                    "LET" => TokenKind::Let,
                    "INPUT" => TokenKind::Input,
                    "IF" => TokenKind::If,
                    "THEN" => TokenKind::Then,
                    "ELSE" => TokenKind::Else,
                    "END" => TokenKind::End,
                    "WHILE" => TokenKind::While,
                    "FOR" => TokenKind::For,
                    "TO" => TokenKind::To,
                    "NEXT" => TokenKind::Next,
                    "DO" => TokenKind::Do,
                    "FUNC" => TokenKind::Func,
                    "RET" => TokenKind::Ret,
                    "BREAK" => TokenKind::Break,
                    "PRINT" => TokenKind::Print,
                    "IMPORT" => TokenKind::Import,
                    "AS" => TokenKind::As,
                    "TRUE" => TokenKind::True,
                    "FALSE" => TokenKind::False,
                    "AND" => TokenKind::And,
                    "OR" => TokenKind::Or,
                    "NOT" => TokenKind::Not,
                    _ => TokenKind::Ident(hi_common::intern(&ident)),
                };
                tokens.push(lexer.token(kind, start_line, start_col, line, col));
                continue;
            }

            // Operators and punctuation
            match ch {
                '+' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        tokens.push(lexer.token(
                            TokenKind::PlusAssign,
                            start_line,
                            start_col,
                            line,
                            col + 1,
                        ));
                        pos += 2;
                        col += 2;
                    } else {
                        tokens.push(lexer.token(TokenKind::Plus, start_line, start_col, line, col));
                        pos += 1;
                        col += 1;
                    }
                    continue;
                }
                '-' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        tokens.push(lexer.token(
                            TokenKind::MinusAssign,
                            start_line,
                            start_col,
                            line,
                            col + 1,
                        ));
                        pos += 2;
                        col += 2;
                    } else {
                        tokens.push(lexer.token(
                            TokenKind::Minus,
                            start_line,
                            start_col,
                            line,
                            col,
                        ));
                        pos += 1;
                        col += 1;
                    }
                    continue;
                }
                '*' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        tokens.push(lexer.token(
                            TokenKind::StarAssign,
                            start_line,
                            start_col,
                            line,
                            col + 1,
                        ));
                        pos += 2;
                        col += 2;
                    } else {
                        tokens.push(lexer.token(TokenKind::Star, start_line, start_col, line, col));
                        pos += 1;
                        col += 1;
                    }
                    continue;
                }
                '/' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        tokens.push(lexer.token(
                            TokenKind::SlashAssign,
                            start_line,
                            start_col,
                            line,
                            col + 1,
                        ));
                        pos += 2;
                        col += 2;
                    } else {
                        tokens.push(lexer.token(
                            TokenKind::Slash,
                            start_line,
                            start_col,
                            line,
                            col,
                        ));
                        pos += 1;
                        col += 1;
                    }
                    continue;
                }
                '%' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        tokens.push(lexer.token(
                            TokenKind::PercentAssign,
                            start_line,
                            start_col,
                            line,
                            col + 1,
                        ));
                        pos += 2;
                        col += 2;
                    } else {
                        tokens.push(lexer.token(
                            TokenKind::Percent,
                            start_line,
                            start_col,
                            line,
                            col,
                        ));
                        pos += 1;
                        col += 1;
                    }
                    continue;
                }
                '^' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        tokens.push(lexer.token(
                            TokenKind::CaretAssign,
                            start_line,
                            start_col,
                            line,
                            col + 1,
                        ));
                        pos += 2;
                        col += 2;
                    } else {
                        tokens.push(lexer.token(
                            TokenKind::Caret,
                            start_line,
                            start_col,
                            line,
                            col,
                        ));
                        pos += 1;
                        col += 1;
                    }
                    continue;
                }
                '(' => {
                    tokens.push(lexer.token(TokenKind::LParen, start_line, start_col, line, col))
                }
                ')' => {
                    tokens.push(lexer.token(TokenKind::RParen, start_line, start_col, line, col))
                }
                '{' => {
                    tokens.push(lexer.token(TokenKind::LBrace, start_line, start_col, line, col))
                }
                '}' => {
                    tokens.push(lexer.token(TokenKind::RBrace, start_line, start_col, line, col))
                }
                '[' => {
                    tokens.push(lexer.token(TokenKind::LBracket, start_line, start_col, line, col))
                }
                ']' => {
                    tokens.push(lexer.token(TokenKind::RBracket, start_line, start_col, line, col))
                }
                ',' => tokens.push(lexer.token(TokenKind::Comma, start_line, start_col, line, col)),
                ':' => tokens.push(lexer.token(TokenKind::Colon, start_line, start_col, line, col)),
                '=' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        tokens.push(lexer.token(
                            TokenKind::EqEq,
                            start_line,
                            start_col,
                            line,
                            col + 1,
                        ));
                        pos += 2;
                        col += 2;
                    } else {
                        tokens.push(lexer.token(
                            TokenKind::Assign,
                            start_line,
                            start_col,
                            line,
                            col,
                        ));
                        pos += 1;
                        col += 1;
                    }
                    continue;
                }
                '!' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        tokens.push(lexer.token(
                            TokenKind::Neq,
                            start_line,
                            start_col,
                            line,
                            col + 1,
                        ));
                        pos += 2;
                        col += 2;
                    } else {
                        tokens.push(lexer.token(TokenKind::Not, start_line, start_col, line, col));
                        pos += 1;
                        col += 1;
                    }
                    continue;
                }
                '>' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        tokens.push(lexer.token(
                            TokenKind::Ge,
                            start_line,
                            start_col,
                            line,
                            col + 1,
                        ));
                        pos += 2;
                        col += 2;
                    } else {
                        tokens.push(lexer.token(TokenKind::Gt, start_line, start_col, line, col));
                        pos += 1;
                        col += 1;
                    }
                    continue;
                }
                '<' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        tokens.push(lexer.token(
                            TokenKind::Le,
                            start_line,
                            start_col,
                            line,
                            col + 1,
                        ));
                        pos += 2;
                        col += 2;
                    } else {
                        tokens.push(lexer.token(TokenKind::Lt, start_line, start_col, line, col));
                        pos += 1;
                        col += 1;
                    }
                    continue;
                }
                _ => {
                    return Err(LexError {
                        message: format!("Unexpected character '{}'", ch),
                        span: Span {
                            start_line,
                            start_col,
                            end_line: line,
                            end_col: col,
                        },
                    });
                }
            }
            pos += 1;
            col += 1;
        }

        tokens.push(lexer.token(TokenKind::Eof, line, col, line, col));

        Ok(tokens)
    }

    fn token(&mut self, kind: TokenKind, sl: usize, sc: usize, el: usize, ec: usize) -> Token {
        let doc = self.pending_doc.take();
        Token {
            kind,
            span: Span {
                start_line: sl,
                start_col: sc,
                end_line: el,
                end_col: ec,
            },
            doc,
        }
    }

    /// Unescapes a string literal content (without surrounding quotes).
    /// Returns an error on invalid Unicode escape sequences.
    fn unescape_string(raw: &str, span: Span) -> Result<String, LexError> {
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
                                return Err(LexError {
                                    message: "Expected '{' after \\u".to_string(),
                                    span,
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
                                    return Err(LexError {
                                        message: format!(
                                            "Invalid hex digit in \\u{{...}}: '{}'",
                                            ch
                                        ),
                                        span,
                                    });
                                }
                                hex.push(ch);
                                chars.next();
                            }
                            if !closed {
                                return Err(LexError {
                                    message: "Unclosed \\u{...} sequence".to_string(),
                                    span,
                                });
                            }
                            if hex.is_empty() {
                                return Err(LexError {
                                    message: "Empty Unicode code point in \\u{{...}}".to_string(),
                                    span,
                                });
                            }
                            match u32::from_str_radix(&hex, 16) {
                                Ok(codepoint) => {
                                    if let Some(ch) = std::char::from_u32(codepoint) {
                                        result.push(ch);
                                    } else {
                                        return Err(LexError {
                                            message: format!(
                                                "Invalid Unicode code point: U+{}",
                                                hex
                                            ),
                                            span,
                                        });
                                    }
                                }
                                Err(_) => {
                                    return Err(LexError {
                                        message: format!("Invalid hex number: {}", hex),
                                        span,
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
                    return Err(LexError {
                        message: "Unexpected end of escape sequence".to_string(),
                        span,
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

    // Helper: extracts only token kinds for simplified comparison
    fn tokens_kinds(tokens: &[Token]) -> Vec<TokenKind> {
        tokens.iter().map(|t| t.kind.clone()).collect()
    }

    // Helper: checks that tokenization succeeds and returns expected kinds
    fn assert_tokens(input: &str, expected_kinds: Vec<TokenKind>) {
        let tokens = Lexer::tokenize(input).expect("tokenization failed");
        let kinds = tokens_kinds(&tokens);
        assert_eq!(kinds, expected_kinds);
    }

    // Helper: checks error with message content
    fn assert_error(input: &str, expected_msg_contains: &str) {
        let result = Lexer::tokenize(input);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(
                e.message.contains(expected_msg_contains),
                "Expected error containing '{}', got '{}'",
                expected_msg_contains,
                e.message
            );
        }
    }

    // ---------- Numbers ----------
    #[test]
    fn test_int() {
        assert_tokens("123", vec![TokenKind::Int(123), TokenKind::Eof]);
        assert_tokens("0", vec![TokenKind::Int(0), TokenKind::Eof]);
        assert_tokens("42", vec![TokenKind::Int(42), TokenKind::Eof]);
    }

    #[test]
    fn test_float_standard() {
        assert_tokens("3.14", vec![TokenKind::Float(3.14), TokenKind::Eof]);
        assert_tokens("0.5", vec![TokenKind::Float(0.5), TokenKind::Eof]);
        assert_tokens("10.0", vec![TokenKind::Float(10.0), TokenKind::Eof]);
    }

    #[test]
    fn test_float_leading_dot() {
        assert_tokens(".5", vec![TokenKind::Float(0.5), TokenKind::Eof]);
        assert_tokens(".123", vec![TokenKind::Float(0.123), TokenKind::Eof]);
        assert_tokens(".0", vec![TokenKind::Float(0.0), TokenKind::Eof]);
    }

    #[test]
    fn test_float_trailing_dot() {
        assert_tokens("1.", vec![TokenKind::Float(1.0), TokenKind::Eof]);
        assert_tokens("42.", vec![TokenKind::Float(42.0), TokenKind::Eof]);
        assert_tokens("0.", vec![TokenKind::Float(0.0), TokenKind::Eof]);
    }

    #[test]
    fn test_float_multiple_dots_error() {
        assert_error("1.2.3", "Multiple decimal points in number");
        assert_error(".5.6", "Multiple decimal points in number");
        assert_error("1..2", "Multiple decimal points in number");
    }

    #[test]
    fn test_number_trailing_letters_error() {
        assert_error("123abc", "trailing characters");
        assert_error("3.14xyz", "trailing characters");
        assert_error("1._", "trailing characters");
    }

    #[test]
    fn test_number_with_underscore_error() {
        assert_error("123_456", "trailing characters");
    }

    // ---------- Identifiers and keywords ----------
    #[test]
    fn test_keywords() {
        let input = "LET IF THEN ELSE END WHILE DO FUNC RET BREAK PRINT TRUE FALSE AND OR NOT";
        let expected = vec![
            TokenKind::Let,
            TokenKind::If,
            TokenKind::Then,
            TokenKind::Else,
            TokenKind::End,
            TokenKind::While,
            TokenKind::Do,
            TokenKind::Func,
            TokenKind::Ret,
            TokenKind::Break,
            TokenKind::Print,
            TokenKind::True,
            TokenKind::False,
            TokenKind::And,
            TokenKind::Or,
            TokenKind::Not,
            TokenKind::Eof,
        ];
        assert_tokens(input, expected);
    }

    #[test]
    fn test_identifiers() {
        assert_tokens(
            "foo",
            vec![TokenKind::Ident(hi_common::intern("foo")), TokenKind::Eof],
        );
        assert_tokens(
            "bar123",
            vec![
                TokenKind::Ident(hi_common::intern("bar123")),
                TokenKind::Eof,
            ],
        );
        assert_tokens(
            "_var",
            vec![TokenKind::Ident(hi_common::intern("_var")), TokenKind::Eof],
        );
        assert_tokens(
            "my_var",
            vec![
                TokenKind::Ident(hi_common::intern("my_var")),
                TokenKind::Eof,
            ],
        );
    }

    // ---------- Strings ----------
    #[test]
    fn test_string_basic() {
        assert_tokens(
            "\"hello\"",
            vec![TokenKind::String("hello".to_string()), TokenKind::Eof],
        );
        assert_tokens(
            "\"\"",
            vec![TokenKind::String("".to_string()), TokenKind::Eof],
        );
    }

    #[test]
    fn test_string_escapes() {
        assert_tokens(
            "\"\\n\"",
            vec![TokenKind::String("\n".to_string()), TokenKind::Eof],
        );
        assert_tokens(
            "\"\\r\"",
            vec![TokenKind::String("\r".to_string()), TokenKind::Eof],
        );
        assert_tokens(
            "\"\\t\"",
            vec![TokenKind::String("\t".to_string()), TokenKind::Eof],
        );
        assert_tokens(
            "\"\\\\\"",
            vec![TokenKind::String("\\".to_string()), TokenKind::Eof],
        );
        assert_tokens(
            "\"\\\"\"",
            vec![TokenKind::String("\"".to_string()), TokenKind::Eof],
        );
    }

    #[test]
    fn test_string_unicode_escape() {
        assert_tokens(
            "\"\\u{1F600}\"",
            vec![TokenKind::String("😀".to_string()), TokenKind::Eof],
        );
        assert_tokens(
            "\"\\u{41}\"",
            vec![TokenKind::String("A".to_string()), TokenKind::Eof],
        );
    }

    #[test]
    fn test_string_unclosed_error() {
        assert_error("\"hello", "Unclosed string literal");
        assert_error("\"", "Unclosed string literal");
    }

    #[test]
    fn test_string_invalid_escape() {
        assert_error("\"\\u{ZZZ}\"", "Invalid hex digit");
        assert_error("\"\\u{110000}\"", "Invalid Unicode code point");
        assert_error("\"\\u{123\"", "Unclosed \\u{...} sequence");
    }

    // ---------- Operators and punctuation ----------
    #[test]
    fn test_operators() {
        let input = "+ - * / % ^ == != > >= < <= = ! ( ) { } ,";
        let expected = vec![
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
            TokenKind::Caret,
            TokenKind::EqEq,
            TokenKind::Neq,
            TokenKind::Gt,
            TokenKind::Ge,
            TokenKind::Lt,
            TokenKind::Le,
            TokenKind::Assign,
            TokenKind::Not,
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::Comma,
            TokenKind::Eof,
        ];
        assert_tokens(input, expected);
    }

    // ---------- Comments and whitespace ----------
    #[test]
    fn test_comments() {
        let input = "LET x = 5 // this is a comment\nPRINT x";
        let expected = vec![
            TokenKind::Let,
            TokenKind::Ident(hi_common::intern("x")),
            TokenKind::Assign,
            TokenKind::Int(5),
            TokenKind::Print,
            TokenKind::Ident(hi_common::intern("x")),
            TokenKind::Eof,
        ];
        assert_tokens(input, expected);
    }

    #[test]
    fn test_whitespace() {
        let input = "   LET    x  =   5   \n\n";
        let expected = vec![
            TokenKind::Let,
            TokenKind::Ident(hi_common::intern("x")),
            TokenKind::Assign,
            TokenKind::Int(5),
            TokenKind::Eof,
        ];
        assert_tokens(input, expected);
    }

    // ---------- Unknown character ----------
    #[test]
    fn test_unknown_char_error() {
        assert_error("@", "Unexpected character '@'");
        assert_error("#", "Unexpected character '#'");
    }

    // ---------- Mixed tokens ----------
    #[test]
    fn test_mixed_expression() {
        let input = "LET x = 3 + 4 * (2 - 1) / .5";
        let expected = vec![
            TokenKind::Let,
            TokenKind::Ident(hi_common::intern("x")),
            TokenKind::Assign,
            TokenKind::Int(3),
            TokenKind::Plus,
            TokenKind::Int(4),
            TokenKind::Star,
            TokenKind::LParen,
            TokenKind::Int(2),
            TokenKind::Minus,
            TokenKind::Int(1),
            TokenKind::RParen,
            TokenKind::Slash,
            TokenKind::Float(0.5),
            TokenKind::Eof,
        ];
        assert_tokens(input, expected);
    }

    #[test]
    fn test_func_definition() {
        let input = "FUNC greet(name)\n    PRINT \"Hello, \" + name\nRET\nEND";
        let tokens = Lexer::tokenize(input).unwrap();
        let kinds = tokens_kinds(&tokens);
        assert_eq!(
            kinds,
            vec![
                TokenKind::Func,
                TokenKind::Ident(hi_common::intern("greet")),
                TokenKind::LParen,
                TokenKind::Ident(hi_common::intern("name")),
                TokenKind::RParen,
                TokenKind::Print,
                TokenKind::String("Hello, ".to_string()),
                TokenKind::Plus,
                TokenKind::Ident(hi_common::intern("name")),
                TokenKind::Ret,
                TokenKind::End,
                TokenKind::Eof,
            ]
        );
    }

    // ---------- Edge cases ----------
    #[test]
    fn test_empty_input() {
        assert_tokens("", vec![TokenKind::Eof]);
    }

    #[test]
    fn test_only_comment() {
        assert_tokens("// comment", vec![TokenKind::Eof]);
    }

    #[test]
    fn test_float_with_plus_sign() {
        let input = "+.5";
        let expected = vec![TokenKind::Plus, TokenKind::Float(0.5), TokenKind::Eof];
        assert_tokens(input, expected);
    }

    #[test]
    fn test_float_with_minus_sign() {
        let input = "-.5";
        let expected = vec![TokenKind::Minus, TokenKind::Float(0.5), TokenKind::Eof];
        assert_tokens(input, expected);
    }
}
