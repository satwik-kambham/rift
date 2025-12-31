use crate::environment::VariableType;
use crate::expression;
use crate::operator;
use crate::primitive;
use crate::statement;
use crate::token::Token;
use crate::token::TokenType;

macro_rules! expect_token {
    ($parser:expr, $pattern:pat, $msg:expr) => {
        if matches!($parser.peek().token_type, $pattern) {
            $parser.consume()
        } else {
            panic!("Parse error: expected {}, found {:?}", $msg, $parser.peek());
        }
    };
}

macro_rules! consume_token {
    ($parser:expr, $pattern:pat) => {
        if matches!($parser.peek().token_type, $pattern) {
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
        if consume_token!(self, TokenType::Fn) {
            return self.function_declaration();
        }
        self.statement()
    }

    fn block(&mut self) -> Vec<Box<dyn statement::Statement>> {
        let mut statements = vec![];

        while !matches!(self.peek().token_type, TokenType::RightBrace) && !self.is_at_eof() {
            statements.push(self.declaration());
        }

        statements
    }

    fn function_declaration(&mut self) -> Box<dyn statement::Statement> {
        let export_function = consume_token!(self, TokenType::Export);

        let identifier = self.expect_identifier();
        expect_token!(self, TokenType::LeftParentheses, "(");
        let mut parameters = vec![];
        if !matches!(self.peek().token_type, TokenType::RightParentheses) {
            loop {
                parameters.push(self.expect_identifier());

                if !consume_token!(self, TokenType::Comma) {
                    break;
                }
            }
        }
        expect_token!(self, TokenType::RightParentheses, ")");
        expect_token!(self, TokenType::LeftBrace, "{");
        let body = self.block();
        expect_token!(self, TokenType::RightBrace, "}");
        Box::new(statement::FunctionDefinitionStatement::new(
            identifier,
            parameters,
            body,
            export_function,
        ))
    }

    fn statement(&mut self) -> Box<dyn statement::Statement> {
        if consume_token!(self, TokenType::Loop) {
            return self.loop_statement();
        }
        if consume_token!(self, TokenType::If) {
            return self.if_statement();
        }
        if consume_token!(self, TokenType::Break) {
            return self.break_statement();
        }
        if consume_token!(self, TokenType::Return) {
            return self.return_statement();
        }
        if matches!(self.peek().token_type, TokenType::Local | TokenType::Export) {
            return self.assignment_statement();
        }
        if matches!(self.peek().token_type, TokenType::Identifier(_))
            && matches!(self.peek_n(1).token_type, TokenType::Equals)
        {
            return self.assignment_statement();
        }
        self.expression_statement()
    }

    fn loop_statement(&mut self) -> Box<dyn statement::Statement> {
        expect_token!(self, TokenType::LeftBrace, "{");
        let body = self.block();
        expect_token!(self, TokenType::RightBrace, "}");
        Box::new(statement::LoopStatement::new(body))
    }

    fn if_statement(&mut self) -> Box<dyn statement::Statement> {
        let condition_expression = self.expression();
        expect_token!(self, TokenType::LeftBrace, "{");
        let body = self.block();
        expect_token!(self, TokenType::RightBrace, "}");
        Box::new(statement::IfStatement::new(condition_expression, body))
    }

    fn break_statement(&mut self) -> Box<dyn statement::Statement> {
        Box::new(statement::BreakStatement::new())
    }

    fn return_statement(&mut self) -> Box<dyn statement::Statement> {
        let expression = self.expression();
        Box::new(statement::ReturnStatement::new(expression))
    }

    fn assignment_statement(&mut self) -> Box<dyn statement::Statement> {
        let variable_type = if consume_token!(self, TokenType::Local) {
            VariableType::Local
        } else if consume_token!(self, TokenType::Export) {
            VariableType::Export
        } else {
            VariableType::Default
        };
        let identifier = self.expect_identifier();
        expect_token!(self, TokenType::Equals, "=");
        let expression = self.expression();
        Box::new(statement::AssignmentStatement::new(
            identifier,
            expression,
            variable_type,
        ))
    }

    fn expression_statement(&mut self) -> Box<dyn statement::Statement> {
        let expression = self.expression();
        Box::new(statement::ExpressionStatement::new(expression))
    }

    fn expression(&mut self) -> Box<dyn expression::Expression> {
        self.or_expression()
    }

    fn or_expression(&mut self) -> Box<dyn expression::Expression> {
        let mut expression = self.and_expression();

        while matches!(self.peek().token_type, TokenType::Or) {
            let operator = match &self.consume().token_type {
                TokenType::Or => operator::Operator::Or,
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

        while matches!(self.peek().token_type, TokenType::And) {
            let operator = match &self.consume().token_type {
                TokenType::And => operator::Operator::And,
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

        while matches!(
            self.peek().token_type,
            TokenType::IsEqual | TokenType::NotEqual
        ) {
            let operator = match &self.consume().token_type {
                TokenType::IsEqual => operator::Operator::IsEqual,
                TokenType::NotEqual => operator::Operator::NotEqual,
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
            self.peek().token_type,
            TokenType::LessThan
                | TokenType::LessThanEqual
                | TokenType::GreaterThan
                | TokenType::GreaterThanEqual
        ) {
            let operator = match &self.consume().token_type {
                TokenType::LessThan => operator::Operator::LessThan,
                TokenType::LessThanEqual => operator::Operator::LessThanEqual,
                TokenType::GreaterThan => operator::Operator::GreaterThan,
                TokenType::GreaterThanEqual => operator::Operator::GreaterThanEqual,
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

        while matches!(self.peek().token_type, TokenType::Plus | TokenType::Minus) {
            let operator = match &self.consume().token_type {
                TokenType::Plus => operator::Operator::Plus,
                TokenType::Minus => operator::Operator::Minus,
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

        while matches!(
            self.peek().token_type,
            TokenType::Asterisk | TokenType::Slash | TokenType::Percent
        ) {
            let operator = match &self.consume().token_type {
                TokenType::Asterisk => operator::Operator::Asterisk,
                TokenType::Slash => operator::Operator::Slash,
                TokenType::Percent => operator::Operator::Percent,
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
        if matches!(self.peek().token_type, TokenType::Not | TokenType::Minus) {
            let operator = match &self.consume().token_type {
                TokenType::Not => operator::Operator::Not,
                TokenType::Minus => operator::Operator::Minus,
                other => panic!("Parse error: expected identifier, found {:?}", other),
            };

            let right = self.unary_expression();
            return Box::new(expression::UnaryExpression::new(operator, right));
        }

        self.literal_expression()
    }

    fn literal_expression(&mut self) -> Box<dyn expression::Expression> {
        if consume_token!(self, TokenType::Null) {
            return Box::new(expression::LiteralExpression::new(
                primitive::Primitive::Null,
            ));
        }
        if consume_token!(self, TokenType::True) {
            return Box::new(expression::LiteralExpression::new(
                primitive::Primitive::Boolean(true),
            ));
        }
        if consume_token!(self, TokenType::False) {
            return Box::new(expression::LiteralExpression::new(
                primitive::Primitive::Boolean(false),
            ));
        }
        if let TokenType::Number(number) = &self.peek().token_type {
            let number = *number;
            expect_token!(self, TokenType::Number(_), "number");
            return Box::new(expression::LiteralExpression::new(
                primitive::Primitive::Number(number),
            ));
        }
        if let TokenType::String(string) = &self.peek().token_type {
            let string = string.to_string();
            expect_token!(self, TokenType::String(_), "string");
            return Box::new(expression::LiteralExpression::new(
                primitive::Primitive::String(string),
            ));
        }
        if let TokenType::Identifier(_) = &self.peek().token_type {
            let identifier = self.expect_identifier();
            if consume_token!(self, TokenType::LeftParentheses) {
                let mut parameters = vec![];
                if !matches!(self.peek().token_type, TokenType::RightParentheses) {
                    loop {
                        parameters.push(self.expression());

                        if !consume_token!(self, TokenType::Comma) {
                            break;
                        }
                    }
                }
                expect_token!(self, TokenType::RightParentheses, ")");
                return Box::new(expression::FunctionCallExpression::new(
                    identifier, parameters,
                ));
            }
            return Box::new(expression::VariableExpression::new(identifier));
        }

        if consume_token!(self, TokenType::LeftParentheses) {
            let expression = self.expression();
            expect_token!(self, TokenType::RightParentheses, ")");
            return Box::new(expression::GroupingExpression::new(expression));
        }

        panic!("Parse error: expected expression, found {:?}", self.peek());
    }

    fn expect_identifier(&mut self) -> String {
        match &self.peek().token_type {
            TokenType::Identifier(identifier) => {
                let identifier = identifier.clone();
                self.consume();
                identifier
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
        matches!(self.peek().token_type, TokenType::EOF)
    }
}
