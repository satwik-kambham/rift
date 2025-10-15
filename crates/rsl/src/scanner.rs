use crate::token::Token;

pub struct Scanner {
    source: String,
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
                    self.advance();
                }
                self.advance();
                let string = self
                    .source
                    .get(self.start + 1..self.current - 1)
                    .unwrap()
                    .to_string();
                let string = string.replace("\\n", "\n");
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
        self.current += 1;
        self.source.chars().nth(self.current - 1).unwrap()
    }

    fn match_token(&mut self, expected: char) -> bool {
        if self.is_at_eof() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn is_at_eof(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek(&self) -> char {
        if self.is_at_eof() {
            return '\0';
        }
        self.source.chars().nth(self.current).unwrap()
    }

    fn peek_n(&self, n: usize) -> char {
        if self.current + n >= self.source.len() {
            return '\0';
        }
        self.source.chars().nth(self.current + n).unwrap()
    }
}
