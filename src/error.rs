//! Custom error types for the interpreter.

use crate::ast::Span;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InterpError {
    #[error("Syntax error at {span}: {message}")]
    Syntax { span: Span, message: String },

    #[error("Semantic error at {span}: {message}")]
    Semantic { span: Span, message: String },

    #[error("Runtime error at {span}: {message}")]
    Runtime { span: Span, message: String },

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Function '{name}' not found at {span}")]
    FuncNotFound { name: String, span: Span },

    #[error("Unclosed block '{block}' at {span}")]
    UnclosedBlock { block: String, span: Span },

    #[error("Cyclic import detected: {path}")]
    CyclicImport { path: String },

    #[error("Import error at {span} for '{}': {message}", path)]
    ImportError {
        path: String,
        message: String,
        span: Span,
    },
}

impl InterpError {
    pub fn line(&self) -> Option<usize> {
        match self {
            InterpError::Syntax { span, .. } => Some(span.start_line),
            InterpError::Semantic { span, .. } => Some(span.start_line),
            InterpError::Runtime { span, .. } => Some(span.start_line),
            InterpError::FuncNotFound { span, .. } => Some(span.start_line),
            InterpError::UnclosedBlock { span, .. } => Some(span.start_line),
            InterpError::ImportError { span, .. } => Some(span.start_line),
            _ => None,
        }
    }

    pub fn span(&self) -> Option<Span> {
        match self {
            InterpError::Syntax { span, .. } => Some(*span),
            InterpError::Semantic { span, .. } => Some(*span),
            InterpError::Runtime { span, .. } => Some(*span),
            InterpError::FuncNotFound { span, .. } => Some(*span),
            InterpError::UnclosedBlock { span, .. } => Some(*span),
            InterpError::ImportError { span, .. } => Some(*span),
            _ => None,
        }
    }
}

pub type InterpResult<T> = Result<T, InterpError>;

#[derive(Error, Debug, Clone, PartialEq)]
#[error("{message}")]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

#[derive(Error, Debug, Clone, PartialEq)]
#[error("{message}")]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl ParseError {
    pub fn line(&self) -> Option<usize> {
        Some(self.span.start_line)
    }

    pub fn span(&self) -> Option<Span> {
        Some(self.span)
    }
}

impl LexError {
    pub fn line(&self) -> Option<usize> {
        Some(self.span.start_line)
    }

    pub fn span(&self) -> Option<Span> {
        Some(self.span)
    }
}
impl From<LexError> for InterpError {
    fn from(e: LexError) -> Self {
        InterpError::Syntax {
            span: e.span,
            message: e.message,
        }
    }
}

impl From<ParseError> for InterpError {
    fn from(e: ParseError) -> Self {
        InterpError::Syntax {
            span: e.span,
            message: e.message,
        }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;
