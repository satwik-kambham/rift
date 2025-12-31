use thiserror::Error;

use crate::token::Span;

#[derive(Error, Debug)]
pub enum RSLError {
    #[error(transparent)]
    ScanError(#[from] ScanError),
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
