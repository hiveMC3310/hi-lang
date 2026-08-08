//! Semantic analysis for the Hi language.
//! Provides AST traversal, symbol resolution, and scoping.

pub mod analysis;
pub mod scope;
pub mod symbol;

// Re-export commonly used types
pub use hi_interpreter::ast::{Expr, Program, Span, Stmt};
pub use hi_interpreter::error::{LexError, ParseError};
pub use hi_interpreter::parser::{Parser, lexer::Lexer};
