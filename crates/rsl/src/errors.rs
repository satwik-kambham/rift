use thiserror::Error;

use crate::token::Span;

#[derive(Error, Debug)]
pub enum RSLError {
    #[error(transparent)]
    ScanError(#[from] ScanError),
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    RuntimeError(#[from] RuntimeError),
}

#[derive(Error, Debug)]
#[error("scan error \"{}\" on line:{}", self.message, self.at.line)]
pub struct ScanError {
    message: String,
    at: Span,
}

impl ScanError {
    pub fn new(message: String, start: usize, end: usize, line: usize) -> Self {
        Self {
            message,
            at: Span {
                start_byte: start,
                end_byte: end,
                line,
            },
        }
    }
}

#[derive(Error, Debug)]
#[error("parse error \"{}\" on line:{}", self.message, self.at.line)]
pub struct ParseError {
    message: String,
    at: Span,
}

impl ParseError {
    pub fn new(message: String, at: Span) -> Self {
        Self { message, at }
    }
}

#[derive(Error, Debug)]
#[error("runtime error \"{}\" on line:{}", self.message, self.at.line)]
pub struct RuntimeError {
    message: String,
    at: Span,
}

impl RuntimeError {
    pub fn new(message: String, at: Span) -> Self {
        Self { message, at }
    }
}
