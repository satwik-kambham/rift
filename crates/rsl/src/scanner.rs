use crate::{
    errors::ScanError,
    token::{Token, TokenType},
};

pub struct Scanner {
    source: String,
    start: usize,
    current: usize,
    line: usize,
    start_line: usize,
    tokens: Vec<Token>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
            start_line: 1,
            tokens: vec![],
        }
    }

    pub fn scan(&mut self) -> Result<Vec<Token>, ScanError> {
        while !self.is_at_eof() {
            self.start = self.current;
            self.start_line = self.line;
            self.scan_token()?;
        }

        self.start = self.current;
        self.start_line = self.line;
        self.add_token(TokenType::EOF);

        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> Result<(), ScanError> {
        let c = self.advance();
        match c {
            '+' => self.add_token(TokenType::Plus),
            '-' => self.add_token(TokenType::Minus),
            '*' => self.add_token(TokenType::Asterisk),
            '/' => self.add_token(TokenType::Slash),
            '%' => self.add_token(TokenType::Percent),
            ';' => self.add_token(TokenType::Semicolon),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '(' => self.add_token(TokenType::LeftParentheses),
            ')' => self.add_token(TokenType::RightParentheses),
            '[' => self.add_token(TokenType::LeftSquareBracket),
            ']' => self.add_token(TokenType::RightSquareBracket),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            '<' => {
                let token_type = if self.match_token('=') {
                    TokenType::LessThanEqual
                } else {
                    TokenType::LessThan
                };
                self.add_token(token_type)
            }
            '>' => {
                let token_type = if self.match_token('=') {
                    TokenType::GreaterThanEqual
                } else {
                    TokenType::GreaterThan
                };
                self.add_token(token_type)
            }
            '!' => {
                let token_type = if self.match_token('=') {
                    TokenType::NotEqual
                } else {
                    TokenType::Not
                };
                self.add_token(token_type)
            }
            '=' => {
                let token_type = if self.match_token('=') {
                    TokenType::IsEqual
                } else {
                    TokenType::Equals
                };
                self.add_token(token_type)
            }
            '#' => {
                while self.peek() != '\n' && !self.is_at_eof() {
                    self.advance();
                }
            }
            ' ' | '\r' | '\t' | '\n' => {}
            '"' => {
                while self.peek() != '"' && !self.is_at_eof() {
                    if self.peek() == '\\' {
                        self.advance();
                    }
                    self.advance();
                }
                if self.is_at_eof() {
                    return Err(ScanError::new(
                        "Unterminated string".into(),
                        self.start,
                        self.current,
                        self.start_line,
                    ));
                }
                self.advance();
                let string = self
                    .source
                    .get(self.start + 1..self.current - 1)
                    .unwrap()
                    .to_string();
                let string = unescaper::unescape(&string).unwrap();
                self.add_token(TokenType::String(string))
            }
            _ => {
                if c.is_ascii_digit() {
                    while self.peek().is_ascii_digit() {
                        self.advance();
                    }
                    if self.peek() == '.' && self.peek_n(1).is_ascii_digit() {
                        self.advance();
                    }
                    while self.peek().is_ascii_digit() {
                        self.advance();
                    }
                    let number = match self
                        .source
                        .get(self.start..self.current)
                        .unwrap()
                        .parse::<f32>()
                    {
                        Ok(n) => n,
                        Err(_) => {
                            return Err(ScanError::new(
                                "Invalid number".into(),
                                self.start,
                                self.current,
                                self.start_line,
                            ));
                        }
                    };
                    self.add_token(TokenType::Number(number))
                } else if c.is_ascii_alphanumeric() {
                    while self.peek().is_ascii_alphanumeric() {
                        self.advance();
                    }
                    let identifier = self.source.get(self.start..self.current).unwrap();
                    let token_type = match identifier {
                        "and" => TokenType::And,
                        "or" => TokenType::Or,
                        "if" => TokenType::If,
                        "else" => TokenType::Else,
                        "loop" => TokenType::Loop,
                        "fn" => TokenType::Fn,
                        "null" => TokenType::Null,
                        "true" => TokenType::True,
                        "false" => TokenType::False,
                        "break" => TokenType::Break,
                        "return" => TokenType::Return,
                        "let" => TokenType::Let,
                        "export" => TokenType::Export,
                        _ => TokenType::Identifier(identifier.to_string()),
                    };
                    self.add_token(token_type)
                } else {
                    return Err(ScanError::new(
                        format!("Unknown character: {}", c),
                        self.start,
                        self.start + c.len_utf8(),
                        self.line,
                    ));
                }
            }
        }
        Ok(())
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.tokens.push(Token::new(
            token_type,
            self.start,
            self.current,
            self.start_line,
        ))
    }

    fn advance(&mut self) -> char {
        let mut iter = self.source[self.current..].char_indices();
        let (_, ch) = iter.next().unwrap();
        if ch == '\n' {
            self.line += 1;
        }
        if let Some((next_offset, _)) = iter.next() {
            self.current += next_offset;
        } else {
            self.current = self.source.len();
        }
        ch
    }

    fn match_token(&mut self, expected: char) -> bool {
        if self.is_at_eof() {
            return false;
        }

        let mut iter = self.source[self.current..].char_indices();
        let (_, ch) = iter.next().unwrap();
        if ch != expected {
            return false;
        }

        if let Some((next_offset, _)) = iter.next() {
            self.current += next_offset;
        } else {
            self.current = self.source.len();
        }

        true
    }

    fn is_at_eof(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek(&self) -> char {
        if self.is_at_eof() {
            return '\0';
        }
        self.source[self.current..].chars().next().unwrap()
    }

    fn peek_n(&self, n: usize) -> char {
        let mut iter = self.source[self.current..].chars();
        for _ in 0..n {
            iter.next();
        }
        iter.next().unwrap_or('\0')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_utf8_in_strings_and_comments() {
        let mut scanner = Scanner::new("print(\"héllo 😊\") #🙂\n1".to_string());
        let tokens = scanner.scan().unwrap();

        let expected = vec![
            Token::new(TokenType::Identifier("print".into()), 0, 5, 1),
            Token::new(TokenType::LeftParentheses, 5, 6, 1),
            Token::new(TokenType::String("héllo 😊".into()), 6, 19, 1),
            Token::new(TokenType::RightParentheses, 19, 20, 1),
            Token::new(TokenType::Number(1.0), 27, 28, 2),
            Token::new(TokenType::EOF, 28, 28, 2),
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn scans_multiline_string() {
        let mut scanner = Scanner::new("\"line1\nline2\nline3\"".to_string());
        let tokens = scanner.scan().unwrap();

        let expected = vec![
            Token::new(TokenType::String("line1\nline2\nline3".into()), 0, 19, 1),
            Token::new(TokenType::EOF, 19, 19, 3),
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn scans_dot_token() {
        let mut scanner = Scanner::new("table.fun(1)".to_string());
        let tokens = scanner.scan().unwrap();

        let expected = vec![
            Token::new(TokenType::Identifier("table".into()), 0, 5, 1),
            Token::new(TokenType::Dot, 5, 6, 1),
            Token::new(TokenType::Identifier("fun".into()), 6, 9, 1),
            Token::new(TokenType::LeftParentheses, 9, 10, 1),
            Token::new(TokenType::Number(1.0), 10, 11, 1),
            Token::new(TokenType::RightParentheses, 11, 12, 1),
            Token::new(TokenType::EOF, 12, 12, 1),
        ];

        assert_eq!(tokens, expected);
    }
}
