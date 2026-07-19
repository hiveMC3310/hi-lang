//! Custom error types for the interpreter.

use thiserror::Error;

/// All errors that can occur during interpretation.
#[derive(Error, Debug)]
pub enum InterpError {
    #[error("Syntax error at line {line}: {message}")]
    Syntax { line: usize, message: String },

    #[error("Semantic error at line {line}: {message}")]
    Semantic { line: usize, message: String },

    #[error("Runtime error at line {line}: {message}")]
    Runtime { line: usize, message: String },

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Function '{name}' not found at line {line}")]
    FuncNotFound { name: String, line: usize },

    #[error("Unclosed block: {block}")]
    UnclosedBlock { block: String },
}

impl InterpError {
    /// Returns the source line number if present.
    pub fn line(&self) -> Option<usize> {
        match self {
            InterpError::Syntax { line, .. } => Some(*line),
            InterpError::Semantic { line, .. } => Some(*line),
            InterpError::Runtime { line, .. } => Some(*line),
            _ => None,
        }
    }
}

pub type InterpResult<T> = Result<T, InterpError>;
