#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    Equals,
    IsEqual,
    NotEqual,

    Semicolon,
    Comma,
    LeftParentheses,
    RightParentheses,
    LeftSquareBracket,
    RightSquareBracket,
    LeftBrace,
    RightBrace,

    And,
    Or,
    Not,
    If,
    Loop,
    Fn,
    Break,
    Return,
    Local,
    Export,

    Hash,

    Null,
    True,
    False,
    Number(f32),
    String(String),

    Identifier(String),

    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,
}

impl Token {
    pub fn new(token_type: TokenType, start: usize, end: usize) -> Self {
        Self {
            token_type,
            span: Span { start, end },
        }
    }
}
