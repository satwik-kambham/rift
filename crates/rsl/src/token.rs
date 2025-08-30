#[derive(Debug, Clone)]
pub enum Token {
    // Operators
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

    // Delimiters
    Semicolon,
    Comma,
    LeftParentheses,
    RightParentheses,
    LeftSquareBracket,
    RightSquareBracket,
    LeftBrace,
    RightBrace,

    // Keywords
    And,
    Or,
    Not,
    If,
    Else,
    While,
    Fn,

    // Comment
    Hash,

    // Literals
    Null,
    True,
    False,
    Number(f32),
    String(String),

    Identifier(String),

    EOF,
}
