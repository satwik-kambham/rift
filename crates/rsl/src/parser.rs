use crate::environment::DeclarationType;
use crate::errors::ParseError;
use crate::expression;
use crate::operator;
use crate::primitive;
use crate::statement;
use crate::token::Span;
use crate::token::Token;
use crate::token::TokenType;

macro_rules! expect_token {
    ($parser:expr, $pattern:pat, $msg:expr) => {
        if matches!($parser.peek().token_type, $pattern) {
            $parser.consume()
        } else {
            return Err(ParseError::new(
                format!("expected {}, found {:?}", $msg, $parser.peek().token_type),
                $parser.peek().span.clone(),
            ));
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

    pub fn parse(&mut self) -> Result<Vec<Box<dyn statement::Statement>>, ParseError> {
        let mut statements = vec![];

        while !self.is_at_eof() {
            statements.push(self.declaration()?);
        }

        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Box<dyn statement::Statement>, ParseError> {
        if consume_token!(self, TokenType::Fn) {
            return self.function_declaration();
        }
        self.statement()
    }

    fn block(&mut self) -> Result<Vec<Box<dyn statement::Statement>>, ParseError> {
        let mut statements = vec![];

        while !matches!(self.peek().token_type, TokenType::RightBrace) && !self.is_at_eof() {
            statements.push(self.declaration()?);
        }

        Ok(statements)
    }

    fn function_declaration(&mut self) -> Result<Box<dyn statement::Statement>, ParseError> {
        let export_function = consume_token!(self, TokenType::Export);

        let identifier = self.expect_identifier()?;
        expect_token!(self, TokenType::LeftParentheses, "(");
        let mut parameters = vec![];
        if !matches!(self.peek().token_type, TokenType::RightParentheses) {
            loop {
                parameters.push(self.expect_identifier()?);

                if !consume_token!(self, TokenType::Comma) {
                    break;
                }
            }
        }
        expect_token!(self, TokenType::RightParentheses, ")");
        expect_token!(self, TokenType::LeftBrace, "{");
        let body = self.block()?;
        expect_token!(self, TokenType::RightBrace, "}");
        Ok(Box::new(statement::FunctionDefinitionStatement::new(
            identifier,
            parameters,
            body,
            export_function,
        )))
    }

    fn statement(&mut self) -> Result<Box<dyn statement::Statement>, ParseError> {
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
        if matches!(self.peek().token_type, TokenType::Let | TokenType::Export) {
            return self.assignment_statement();
        }
        if self.looks_like_index_assignment() {
            return self.index_assignment_statement();
        }
        if matches!(self.peek().token_type, TokenType::Identifier(_))
            && matches!(self.peek_n(1).token_type, TokenType::Equals)
        {
            return self.assignment_statement();
        }
        self.expression_statement()
    }

    fn loop_statement(&mut self) -> Result<Box<dyn statement::Statement>, ParseError> {
        expect_token!(self, TokenType::LeftBrace, "{");
        let body = self.block()?;
        expect_token!(self, TokenType::RightBrace, "}");
        Ok(Box::new(statement::LoopStatement::new(body)))
    }

    fn if_statement(&mut self) -> Result<Box<dyn statement::Statement>, ParseError> {
        let start_span = self.peek().span.clone();
        let condition_expression = self.expression()?;
        expect_token!(self, TokenType::LeftBrace, "{");
        let body = self.block()?;
        expect_token!(self, TokenType::RightBrace, "}");

        let else_body = if consume_token!(self, TokenType::Else) {
            if consume_token!(self, TokenType::If) {
                let first_condition = self.expression()?;
                expect_token!(self, TokenType::LeftBrace, "{");
                let first_body = self.block()?;
                expect_token!(self, TokenType::RightBrace, "}");

                let mut else_if_branches: Vec<(
                    Box<dyn expression::Expression>,
                    Vec<Box<dyn statement::Statement>>,
                )> = vec![(first_condition, first_body)];

                let mut final_else: Option<Vec<Box<dyn statement::Statement>>> = None;

                while consume_token!(self, TokenType::Else) {
                    if consume_token!(self, TokenType::If) {
                        let cond = self.expression()?;
                        expect_token!(self, TokenType::LeftBrace, "{");
                        let body = self.block()?;
                        expect_token!(self, TokenType::RightBrace, "}");
                        else_if_branches.push((cond, body));
                    } else {
                        expect_token!(self, TokenType::LeftBrace, "{");
                        final_else = Some(self.block()?);
                        expect_token!(self, TokenType::RightBrace, "}");
                        break;
                    }
                }

                let mut chain_else_body = final_else;
                for (cond, body) in else_if_branches.into_iter().rev() {
                    let node = Box::new(statement::IfStatement::new(
                        cond,
                        body,
                        chain_else_body,
                        start_span.clone(),
                    ));
                    chain_else_body = Some(vec![node]);
                }

                chain_else_body
            } else {
                expect_token!(self, TokenType::LeftBrace, "{");
                let else_body = self.block()?;
                expect_token!(self, TokenType::RightBrace, "}");
                Some(else_body)
            }
        } else {
            None
        };

        Ok(Box::new(statement::IfStatement::new(
            condition_expression,
            body,
            else_body,
            start_span,
        )))
    }

    fn break_statement(&mut self) -> Result<Box<dyn statement::Statement>, ParseError> {
        Ok(Box::new(statement::BreakStatement::new()))
    }

    fn return_statement(&mut self) -> Result<Box<dyn statement::Statement>, ParseError> {
        let expression = self.expression()?;
        Ok(Box::new(statement::ReturnStatement::new(expression)))
    }

    fn assignment_statement(&mut self) -> Result<Box<dyn statement::Statement>, ParseError> {
        let declaration_type = if consume_token!(self, TokenType::Let) {
            DeclarationType::Definition
        } else if consume_token!(self, TokenType::Export) {
            DeclarationType::Export
        } else {
            DeclarationType::Assignment
        };
        let (identifier, identifier_span) = self.expect_identifier_with_span()?;
        expect_token!(self, TokenType::Equals, "=");
        let expression = self.expression()?;
        Ok(Box::new(statement::AssignmentStatement::new(
            identifier,
            expression,
            declaration_type,
            identifier_span,
        )))
    }

    fn expression_statement(&mut self) -> Result<Box<dyn statement::Statement>, ParseError> {
        let expression = self.expression()?;
        Ok(Box::new(statement::ExpressionStatement::new(expression)))
    }

    fn expression(&mut self) -> Result<Box<dyn expression::Expression>, ParseError> {
        self.or_expression()
    }

    fn or_expression(&mut self) -> Result<Box<dyn expression::Expression>, ParseError> {
        let start_span = self.peek().span.clone();
        let mut expression = self.and_expression()?;

        while matches!(self.peek().token_type, TokenType::Or) {
            let operator = match &self.consume().token_type {
                TokenType::Or => operator::Operator::Or,
                other => {
                    return Err(ParseError::new(
                        format!("expected identifier, found {:?}", other),
                        self.peek().span.clone(),
                    ));
                }
            };

            let right = self.and_expression()?;
            expression = Box::new(expression::BinaryExpression::new(
                expression,
                operator,
                right,
                start_span.clone(),
            ));
        }

        Ok(expression)
    }

    fn and_expression(&mut self) -> Result<Box<dyn expression::Expression>, ParseError> {
        let start_span = self.peek().span.clone();
        let mut expression = self.equality_expression()?;

        while matches!(self.peek().token_type, TokenType::And) {
            let operator = match &self.consume().token_type {
                TokenType::And => operator::Operator::And,
                other => {
                    return Err(ParseError::new(
                        format!("expected identifier, found {:?}", other),
                        self.peek().span.clone(),
                    ));
                }
            };

            let right = self.equality_expression()?;
            expression = Box::new(expression::BinaryExpression::new(
                expression,
                operator,
                right,
                start_span.clone(),
            ));
        }

        Ok(expression)
    }

    fn equality_expression(&mut self) -> Result<Box<dyn expression::Expression>, ParseError> {
        let start_span = self.peek().span.clone();
        let mut expression = self.comparison_expression()?;

        while matches!(
            self.peek().token_type,
            TokenType::IsEqual | TokenType::NotEqual
        ) {
            let operator = match &self.consume().token_type {
                TokenType::IsEqual => operator::Operator::IsEqual,
                TokenType::NotEqual => operator::Operator::NotEqual,
                other => {
                    return Err(ParseError::new(
                        format!("expected identifier, found {:?}", other),
                        self.peek().span.clone(),
                    ));
                }
            };

            let right = self.comparison_expression()?;
            expression = Box::new(expression::BinaryExpression::new(
                expression,
                operator,
                right,
                start_span.clone(),
            ));
        }

        Ok(expression)
    }

    fn comparison_expression(&mut self) -> Result<Box<dyn expression::Expression>, ParseError> {
        let start_span = self.peek().span.clone();
        let mut expression = self.term_expression()?;

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
                other => {
                    return Err(ParseError::new(
                        format!("expected identifier, found {:?}", other),
                        self.peek().span.clone(),
                    ));
                }
            };

            let right = self.term_expression()?;
            expression = Box::new(expression::BinaryExpression::new(
                expression,
                operator,
                right,
                start_span.clone(),
            ));
        }

        Ok(expression)
    }

    fn term_expression(&mut self) -> Result<Box<dyn expression::Expression>, ParseError> {
        let start_span = self.peek().span.clone();
        let mut expression = self.factor_expression()?;

        while matches!(self.peek().token_type, TokenType::Plus | TokenType::Minus) {
            let operator = match &self.consume().token_type {
                TokenType::Plus => operator::Operator::Plus,
                TokenType::Minus => operator::Operator::Minus,
                other => {
                    return Err(ParseError::new(
                        format!("expected identifier, found {:?}", other),
                        self.peek().span.clone(),
                    ));
                }
            };

            let right = self.factor_expression()?;
            expression = Box::new(expression::BinaryExpression::new(
                expression,
                operator,
                right,
                start_span.clone(),
            ));
        }

        Ok(expression)
    }

    fn factor_expression(&mut self) -> Result<Box<dyn expression::Expression>, ParseError> {
        let start_span = self.peek().span.clone();
        let mut expression = self.unary_expression()?;

        while matches!(
            self.peek().token_type,
            TokenType::Asterisk | TokenType::Slash | TokenType::Percent
        ) {
            let operator = match &self.consume().token_type {
                TokenType::Asterisk => operator::Operator::Asterisk,
                TokenType::Slash => operator::Operator::Slash,
                TokenType::Percent => operator::Operator::Percent,
                other => {
                    return Err(ParseError::new(
                        format!("expected identifier, found {:?}", other),
                        self.peek().span.clone(),
                    ));
                }
            };

            let right = self.unary_expression()?;
            expression = Box::new(expression::BinaryExpression::new(
                expression,
                operator,
                right,
                start_span.clone(),
            ));
        }

        Ok(expression)
    }

    fn unary_expression(&mut self) -> Result<Box<dyn expression::Expression>, ParseError> {
        if matches!(self.peek().token_type, TokenType::Not | TokenType::Minus) {
            let start_span = self.peek().span.clone();
            let operator = match &self.consume().token_type {
                TokenType::Not => operator::Operator::Not,
                TokenType::Minus => operator::Operator::Minus,
                other => {
                    return Err(ParseError::new(
                        format!("expected identifier, found {:?}", other),
                        self.peek().span.clone(),
                    ));
                }
            };

            let right = self.unary_expression()?;
            return Ok(Box::new(expression::UnaryExpression::new(
                operator, right, start_span,
            )));
        }

        self.postfix_expression()
    }

    fn postfix_expression(&mut self) -> Result<Box<dyn expression::Expression>, ParseError> {
        let mut expression = self.primary_expression()?;

        loop {
            if matches!(self.peek().token_type, TokenType::LeftSquareBracket) {
                let span = self.peek().span.clone();
                self.consume();
                let index_expression = self.expression()?;
                expect_token!(self, TokenType::RightSquareBracket, "]");
                expression = Box::new(expression::IndexExpression::new(
                    expression,
                    index_expression,
                    span,
                ));
                continue;
            }

            if matches!(self.peek().token_type, TokenType::Dot) {
                let span = self.peek().span.clone();
                self.consume();
                let key = self.expect_identifier()?;
                expect_token!(self, TokenType::LeftParentheses, "(");
                let mut parameters = vec![];
                if !matches!(self.peek().token_type, TokenType::RightParentheses) {
                    loop {
                        parameters.push(self.expression()?);
                        if !consume_token!(self, TokenType::Comma) {
                            break;
                        }
                    }
                }
                expect_token!(self, TokenType::RightParentheses, ")");
                expression = Box::new(expression::TableMethodCallExpression::new(
                    expression, key, parameters, span,
                ));
                continue;
            }

            break;
        }

        Ok(expression)
    }

    fn primary_expression(&mut self) -> Result<Box<dyn expression::Expression>, ParseError> {
        if consume_token!(self, TokenType::Null) {
            return Ok(Box::new(expression::LiteralExpression::new(
                primitive::Primitive::Null,
            )));
        }
        if consume_token!(self, TokenType::True) {
            return Ok(Box::new(expression::LiteralExpression::new(
                primitive::Primitive::Boolean(true),
            )));
        }
        if consume_token!(self, TokenType::False) {
            return Ok(Box::new(expression::LiteralExpression::new(
                primitive::Primitive::Boolean(false),
            )));
        }
        if let TokenType::Number(number) = &self.peek().token_type {
            let number = *number;
            expect_token!(self, TokenType::Number(_), "number");
            return Ok(Box::new(expression::LiteralExpression::new(
                primitive::Primitive::Number(number),
            )));
        }
        if let TokenType::String(string) = &self.peek().token_type {
            let string = string.to_string();
            expect_token!(self, TokenType::String(_), "string");
            return Ok(Box::new(expression::LiteralExpression::new(
                primitive::Primitive::String(string),
            )));
        }
        if let TokenType::Identifier(_) = &self.peek().token_type {
            let identifier_span = self.peek().span.clone();
            let identifier = self.expect_identifier()?;
            if consume_token!(self, TokenType::LeftParentheses) {
                let mut parameters = vec![];
                if !matches!(self.peek().token_type, TokenType::RightParentheses) {
                    loop {
                        parameters.push(self.expression()?);

                        if !consume_token!(self, TokenType::Comma) {
                            break;
                        }
                    }
                }
                expect_token!(self, TokenType::RightParentheses, ")");
                return Ok(Box::new(expression::FunctionCallExpression::new(
                    identifier,
                    parameters,
                    identifier_span,
                )));
            }
            return Ok(Box::new(expression::VariableExpression::new(identifier)));
        }

        if consume_token!(self, TokenType::LeftParentheses) {
            let expression = self.expression()?;
            expect_token!(self, TokenType::RightParentheses, ")");
            return Ok(Box::new(expression::GroupingExpression::new(expression)));
        }

        Err(ParseError::new(
            format!("expected expression, found {:?}", self.peek().token_type),
            self.peek().span.clone(),
        ))
    }

    fn looks_like_index_assignment(&self) -> bool {
        if !matches!(self.peek().token_type, TokenType::Identifier(_)) {
            return false;
        }
        if !matches!(self.peek_n(1).token_type, TokenType::LeftSquareBracket) {
            return false;
        }

        let mut depth = 0usize;
        let mut i = self.current + 1;

        while i < self.tokens.len() {
            match self.tokens[i].token_type {
                TokenType::LeftSquareBracket => depth += 1,
                TokenType::RightSquareBracket => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                    if depth == 0 {
                        let next = self.tokens.get(i + 1);
                        match next.map(|token| &token.token_type) {
                            Some(TokenType::Equals) => return true,
                            Some(TokenType::LeftSquareBracket) => {}
                            _ => return false,
                        }
                    }
                }
                TokenType::EOF => return false,
                _ => {}
            }

            i += 1;
        }

        false
    }

    fn index_assignment_statement(&mut self) -> Result<Box<dyn statement::Statement>, ParseError> {
        let start_span = self.peek().span.clone();
        let mut target_expression = self.primary_expression()?;
        let mut indices: Vec<(Box<dyn expression::Expression>, Span)> = Vec::new();

        while matches!(self.peek().token_type, TokenType::LeftSquareBracket) {
            let span = self.peek().span.clone();
            self.consume();
            let index_expression = self.expression()?;
            expect_token!(self, TokenType::RightSquareBracket, "]");
            indices.push((index_expression, span));
        }

        if indices.is_empty() {
            return Err(ParseError::new(
                "expected index expression before assignment".to_string(),
                start_span,
            ));
        }

        let (last_index, _) = indices.pop().unwrap();
        for (index_expression, span) in indices {
            target_expression = Box::new(expression::IndexExpression::new(
                target_expression,
                index_expression,
                span,
            ));
        }

        expect_token!(self, TokenType::Equals, "=");
        let value_expression = self.expression()?;

        Ok(Box::new(statement::IndexAssignmentStatement::new(
            target_expression,
            last_index,
            value_expression,
            start_span,
        )))
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        match &self.peek().token_type {
            TokenType::Identifier(identifier) => {
                let identifier = identifier.clone();
                self.consume();
                Ok(identifier)
            }
            other => Err(ParseError::new(
                format!("expected identifier, found {:?}", other),
                self.peek().span.clone(),
            )),
        }
    }

    fn expect_identifier_with_span(&mut self) -> Result<(String, Span), ParseError> {
        match &self.peek().token_type {
            TokenType::Identifier(identifier) => {
                let identifier = identifier.clone();
                let span = self.peek().span.clone();
                self.consume();
                Ok((identifier, span))
            }
            other => Err(ParseError::new(
                format!("expected identifier, found {:?}", other),
                self.peek().span.clone(),
            )),
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
