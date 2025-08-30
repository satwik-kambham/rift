#[derive(Debug)]
pub enum Token {
    // Operators
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,

    Equals,

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
