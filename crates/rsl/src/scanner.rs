use crate::token::Token;

pub struct Scanner {
    source: String,
    // Byte offsets into source; we iterate via char_indices to stay UTF-8 safe.
    start: usize,
    current: usize,
    tokens: Vec<Token>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            tokens: vec![],
        }
    }

    pub fn scan(&mut self) -> Vec<Token> {
        while !self.is_at_eof() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::EOF);

        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '+' => self.tokens.push(Token::Plus),
            '-' => self.tokens.push(Token::Minus),
            '*' => self.tokens.push(Token::Asterisk),
            '/' => self.tokens.push(Token::Slash),
            '%' => self.tokens.push(Token::Percent),
            ';' => self.tokens.push(Token::Semicolon),
            ',' => self.tokens.push(Token::Comma),
            '(' => self.tokens.push(Token::LeftParentheses),
            ')' => self.tokens.push(Token::RightParentheses),
            '[' => self.tokens.push(Token::LeftSquareBracket),
            ']' => self.tokens.push(Token::RightSquareBracket),
            '{' => self.tokens.push(Token::LeftBrace),
            '}' => self.tokens.push(Token::RightBrace),
            '<' => {
                let token = if self.match_token('=') {
                    Token::LessThanEqual
                } else {
                    Token::LessThan
                };
                self.tokens.push(token)
            }
            '>' => {
                let token = if self.match_token('=') {
                    Token::GreaterThanEqual
                } else {
                    Token::GreaterThan
                };
                self.tokens.push(token)
            }
            '!' => {
                let token = if self.match_token('=') {
                    Token::NotEqual
                } else {
                    Token::Not
                };
                self.tokens.push(token)
            }
            '=' => {
                let token = if self.match_token('=') {
                    Token::IsEqual
                } else {
                    Token::Equals
                };
                self.tokens.push(token)
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
                self.advance();
                let string = self
                    .source
                    .get(self.start + 1..self.current - 1)
                    .unwrap()
                    .to_string();
                let string = unescaper::unescape(&string).unwrap();
                self.tokens.push(Token::String(string))
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
                    let number = self
                        .source
                        .get(self.start..self.current)
                        .unwrap()
                        .parse::<f32>()
                        .unwrap();
                    self.tokens.push(Token::Number(number))
                } else if c.is_ascii_alphanumeric() {
                    while self.peek().is_ascii_alphanumeric() {
                        self.advance();
                    }
                    let identifier = self.source.get(self.start..self.current).unwrap();
                    match identifier {
                        "and" => self.tokens.push(Token::And),
                        "or" => self.tokens.push(Token::Or),
                        "if" => self.tokens.push(Token::If),
                        "loop" => self.tokens.push(Token::Loop),
                        "fn" => self.tokens.push(Token::Fn),
                        "null" => self.tokens.push(Token::Null),
                        "true" => self.tokens.push(Token::True),
                        "false" => self.tokens.push(Token::False),
                        "break" => self.tokens.push(Token::Break),
                        "return" => self.tokens.push(Token::Return),
                        "local" => self.tokens.push(Token::Local),
                        "export" => self.tokens.push(Token::Export),
                        _ => self.tokens.push(Token::Identifier(identifier.to_string())),
                    }
                }
            }
        }
    }

    fn advance(&mut self) -> char {
        // Use char_indices to get the current char and next byte offset.
        let mut iter = self.source[self.current..].char_indices();
        let (_, ch) = iter.next().unwrap();
        if let Some((next_offset, _)) = iter.next() {
            self.current += next_offset;
        } else {
            // Reached the last char; move to end.
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
        let tokens = scanner.scan();

        let expected = vec![
            Token::Identifier("print".into()),
            Token::LeftParentheses,
            Token::String("héllo 😊".into()),
            Token::RightParentheses,
            Token::Number(1.0),
            Token::EOF,
        ];

        assert_eq!(tokens, expected);
    }
}
