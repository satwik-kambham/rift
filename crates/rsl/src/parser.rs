use crate::expression;
use crate::operator;
use crate::primitive;
use crate::statement;
use crate::token::Token;

macro_rules! expect_token {
    ($parser:expr, $pattern:pat, $msg:expr) => {
        if matches!($parser.peek(), $pattern) {
            $parser.consume()
        } else {
            panic!("Parse error: expected {}, found {:?}", $msg, $parser.peek());
        }
    };
}

macro_rules! consume_token {
    ($parser:expr, $pattern:pat) => {
        if matches!($parser.peek(), $pattern) {
            $parser.consume();
            true
        } else {
            false
        }
    };
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Box<dyn statement::Statement>> {
        let mut statements = vec![];

        while !self.is_at_eof() {
            statements.push(self.declaration());
        }

        statements
    }

    fn declaration(&mut self) -> Box<dyn statement::Statement> {
        if consume_token!(self, Token::Fn) {
            return self.function_declaration();
        }
        self.statement()
    }

    fn block(&mut self) -> Vec<Box<dyn statement::Statement>> {
        let mut statements = vec![];

        while !matches!(self.peek(), Token::RightBrace) && !self.is_at_eof() {
            statements.push(self.declaration());
        }

        statements
    }

    fn function_declaration(&mut self) -> Box<dyn statement::Statement> {
        let identifier = self.expect_identifier();
        expect_token!(self, Token::LeftParentheses, "(");
        let mut parameters = vec![];
        if !matches!(self.peek(), Token::RightParentheses) {
            loop {
                parameters.push(self.expect_identifier());

                if !consume_token!(self, Token::Comma) {
                    break;
                }
            }
        }
        expect_token!(self, Token::RightParentheses, ")");
        expect_token!(self, Token::LeftBrace, "{");
        let body = self.block();
        expect_token!(self, Token::RightBrace, "}");
        Box::new(statement::FunctionDefinition::new(
            identifier, parameters, body,
        ))
    }

    fn statement(&mut self) -> Box<dyn statement::Statement> {
        if consume_token!(self, Token::Loop) {
            return self.loop_statement();
        }
        if consume_token!(self, Token::If) {
            return self.if_statement();
        }
        if consume_token!(self, Token::Break) {
            return self.break_statement();
        }
        if consume_token!(self, Token::Return) {
            return self.return_statement();
        }
        if matches!(self.peek(), Token::Identifier(_)) && matches!(self.peek_n(1), Token::Equals) {
            return self.assignment_statement();
        }
        self.expression_statement()
    }

    fn loop_statement(&mut self) -> Box<dyn statement::Statement> {
        expect_token!(self, Token::LeftBrace, "{");
        let body = self.block();
        expect_token!(self, Token::RightBrace, "}");
        Box::new(statement::LoopStatement::new(body))
    }

    fn if_statement(&mut self) -> Box<dyn statement::Statement> {
        let condition_expression = self.expression();
        expect_token!(self, Token::LeftBrace, "{");
        let body = self.block();
        expect_token!(self, Token::RightBrace, "}");
        Box::new(statement::IfStatement::new(condition_expression, body))
    }

    fn break_statement(&mut self) -> Box<dyn statement::Statement> {
        expect_token!(self, Token::Semicolon, ";");
        return Box::new(statement::BreakStatement::new());
    }

    fn return_statement(&mut self) -> Box<dyn statement::Statement> {
        let expression = self.expression();
        expect_token!(self, Token::Semicolon, ";");
        return Box::new(statement::ExpressionStatement::new(expression));
    }

    fn assignment_statement(&mut self) -> Box<dyn statement::Statement> {
        let identifier = self.expect_identifier();
        expect_token!(self, Token::Equals, "=");
        let expression = self.expression();
        expect_token!(self, Token::Semicolon, ";");
        return Box::new(statement::AssignmentStatement::new(identifier, expression));
    }

    fn expression_statement(&mut self) -> Box<dyn statement::Statement> {
        let expression = self.expression();
        expect_token!(self, Token::Semicolon, ";");
        Box::new(statement::ExpressionStatement::new(expression))
    }

    fn expression(&mut self) -> Box<dyn expression::Expression> {
        let expression = self.or_expression();
        expression
    }

    fn or_expression(&mut self) -> Box<dyn expression::Expression> {
        let mut expression = self.and_expression();

        while matches!(self.peek(), Token::Or) {
            let operator = match self.consume() {
                Token::Or => operator::Operator::Or,
                other => panic!("Parse error: expected identifier, found {:?}", other),
            };

            let right = self.and_expression();
            expression = Box::new(expression::BinaryExpression::new(
                expression, operator, right,
            ));
        }

        expression
    }

    fn and_expression(&mut self) -> Box<dyn expression::Expression> {
        let mut expression = self.equality_expression();

        while matches!(self.peek(), Token::And) {
            let operator = match self.consume() {
                Token::And => operator::Operator::And,
                other => panic!("Parse error: expected identifier, found {:?}", other),
            };

            let right = self.equality_expression();
            expression = Box::new(expression::BinaryExpression::new(
                expression, operator, right,
            ));
        }

        expression
    }

    fn equality_expression(&mut self) -> Box<dyn expression::Expression> {
        let mut expression = self.comparison_expression();

        while matches!(self.peek(), Token::IsEqual | Token::NotEqual) {
            let operator = match self.consume() {
                Token::IsEqual => operator::Operator::IsEqual,
                Token::NotEqual => operator::Operator::NotEqual,
                other => panic!("Parse error: expected identifier, found {:?}", other),
            };

            let right = self.comparison_expression();
            expression = Box::new(expression::BinaryExpression::new(
                expression, operator, right,
            ));
        }

        expression
    }

    fn comparison_expression(&mut self) -> Box<dyn expression::Expression> {
        let mut expression = self.term_expression();

        while matches!(
            self.peek(),
            Token::LessThan | Token::LessThanEqual | Token::GreaterThan | Token::GreaterThanEqual
        ) {
            let operator = match self.consume() {
                Token::LessThan => operator::Operator::LessThan,
                Token::LessThanEqual => operator::Operator::LessThanEqual,
                Token::GreaterThan => operator::Operator::GreaterThan,
                Token::GreaterThanEqual => operator::Operator::GreaterThanEqual,
                other => panic!("Parse error: expected identifier, found {:?}", other),
            };

            let right = self.term_expression();
            expression = Box::new(expression::BinaryExpression::new(
                expression, operator, right,
            ));
        }

        expression
    }

    fn term_expression(&mut self) -> Box<dyn expression::Expression> {
        let mut expression = self.factor_expression();

        while matches!(self.peek(), Token::Plus | Token::Minus) {
            let operator = match self.consume() {
                Token::Plus => operator::Operator::Plus,
                Token::Minus => operator::Operator::Minus,
                other => panic!("Parse error: expected identifier, found {:?}", other),
            };

            let right = self.factor_expression();
            expression = Box::new(expression::BinaryExpression::new(
                expression, operator, right,
            ));
        }

        expression
    }

    fn factor_expression(&mut self) -> Box<dyn expression::Expression> {
        let mut expression = self.unary_expression();

        while matches!(self.peek(), Token::Asterisk | Token::Slash | Token::Percent) {
            let operator = match self.consume() {
                Token::Asterisk => operator::Operator::Asterisk,
                Token::Slash => operator::Operator::Slash,
                Token::Percent => operator::Operator::Percent,
                other => panic!("Parse error: expected identifier, found {:?}", other),
            };

            let right = self.unary_expression();
            expression = Box::new(expression::BinaryExpression::new(
                expression, operator, right,
            ));
        }

        expression
    }

    fn unary_expression(&mut self) -> Box<dyn expression::Expression> {
        if matches!(self.peek(), Token::Not | Token::Minus) {
            let operator = match self.consume() {
                Token::Not => operator::Operator::Not,
                Token::Minus => operator::Operator::Minus,
                other => panic!("Parse error: expected identifier, found {:?}", other),
            };

            let right = self.unary_expression();
            return Box::new(expression::UnaryExpression::new(operator, right));
        }

        self.literal_expression()
    }

    fn literal_expression(&mut self) -> Box<dyn expression::Expression> {
        if consume_token!(self, Token::Null) {
            return Box::new(expression::LiteralExpression::new(
                primitive::Primitive::Null,
            ));
        }
        if consume_token!(self, Token::True) {
            return Box::new(expression::LiteralExpression::new(
                primitive::Primitive::Boolean(true),
            ));
        }
        if consume_token!(self, Token::False) {
            return Box::new(expression::LiteralExpression::new(
                primitive::Primitive::Boolean(false),
            ));
        }
        if let Token::Number(number) = self.peek() {
            let number = *number;
            expect_token!(self, Token::Number(_), "number");
            return Box::new(expression::LiteralExpression::new(
                primitive::Primitive::Number(number),
            ));
        }
        if let Token::String(string) = self.peek() {
            let string = string.to_string();
            expect_token!(self, Token::String(_), "string");
            return Box::new(expression::LiteralExpression::new(
                primitive::Primitive::String(string),
            ));
        }
        if let Token::Identifier(_) = self.peek() {
            let identifier = self.expect_identifier();
            if consume_token!(self, Token::LeftParentheses) {
                let mut parameters = vec![];
                if !matches!(self.peek(), Token::RightParentheses) {
                    loop {
                        parameters.push(self.expression());

                        if !consume_token!(self, Token::Comma) {
                            break;
                        }
                    }
                }
                expect_token!(self, Token::RightParentheses, ")");
                return Box::new(expression::FunctionCallExpression::new(
                    identifier, parameters,
                ));
            }
            return Box::new(expression::VariableExpression::new(identifier));
        }

        if consume_token!(self, Token::LeftParentheses) {
            let expression = self.expression();
            expect_token!(self, Token::RightParentheses, ")");
            return Box::new(expression::GroupingExpression::new(expression));
        }

        panic!("Parse error: expected expression, found {:?}", self.peek());
    }

    fn expect_identifier(&mut self) -> String {
        match self.peek() {
            Token::Identifier(identifier) => {
                let identifier = identifier.clone();
                self.consume();
                return identifier;
            }
            other => {
                panic!("Parse error: expected identifier, found {:?}", other);
            }
        }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn peek_n(&self, n: usize) -> &Token {
        self.tokens.get(self.current + n).unwrap()
    }

    fn consume(&mut self) -> &Token {
        self.current += 1;
        self.tokens.get(self.current - 1).unwrap()
    }

    fn is_at_eof(&self) -> bool {
        matches!(self.peek(), Token::EOF)
    }
}
