#[derive(Clone, Debug)]
pub enum Operator {
    Or,
    And,
    IsEqual,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Not,
}
