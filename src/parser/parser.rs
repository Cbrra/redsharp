use super::ast::{BlockStatement, Expr, Operator, Statement};
use super::lexer::{Token, Tokenizer};
use super::precedence::Precedence;

type Error = String;

struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    current_token: Token<'a>,
}

impl<'a> Parser<'a> {
    #[inline]
    fn new(input: &str) -> Parser {
        let mut tokenizer = Tokenizer::new(input);
        let current_token = tokenizer.next().unwrap_or(Token::Unknown);

        Parser {
            tokenizer,
            current_token,
        }
    }

    #[inline(always)]
    fn advance(&mut self) {
        self.current_token = self.tokenizer.next().unwrap_or(Token::Unknown);
    }

    #[inline]
    fn skip(&mut self, t: Token) -> Result<(), Error> {
        if self.current_token != t {
            return Err(format!(
                "Expected token {t:?}, but got: {:?}",
                self.current_token
            ));
        }
        self.advance();
        Ok(())
    }

    #[inline]
    fn skip_optional(&mut self, t: Token) {
        if self.current_token == t {
            self.advance()
        }
    }

    fn parse_operator(&mut self) -> Operator {
        Operator::from(self.current_token)
    }

    #[inline]
    fn parse_expr(&mut self, precedence: Precedence) -> Result<Expr, Error> {
        let mut left = match self.current_token {
            Token::Int(s) => self.parse_int_expression(s),
            Token::True => self.parse_bool_expression(true),
            Token::False => self.parse_bool_expression(false),
            Token::OpenParenthese => {
                self.advance();
                let expr = self.parse_expr(Precedence::Lowest)?;
                self.skip(Token::CloseParenthese)?;
                expr
            }
            Token::If => self.parse_if_expr()?,
            Token::Not | Token::Minus => self.parse_prefix_expr()?,
            Token::Identifier(name) => self.parse_ident(name),
            Token::Func => self.parse_function_expr()?,
            Token::OpenBracket => self.parse_array_expr()?,
            _ => {
                return Err(format!(
                    "Unexpected token: Expected an expression but got {:?}",
                    self.current_token
                ))
            }
        };

        while self.current_token != Token::Semicolon && precedence < self.current_token.precedence()
        {
            left = match self.current_token {
                Token::Lt
                | Token::Lte
                | Token::Gt
                | Token::Gte
                | Token::Eq
                | Token::Ne
                | Token::Plus
                | Token::Minus
                | Token::Slash
                | Token::Caret
                | Token::Star
                | Token::And
                | Token::Or
                | Token::Percent => self.parse_infix_expr(left)?,
                Token::Dot => self.parse_prop_access_expr(left)?,
                Token::Assign => self.parse_assign_expr(left)?,
                Token::OpenParenthese => self.parse_call_expr(left)?,
                Token::OpenBracket => self.parse_index_expr(left)?,
                _ => return Ok(left),
            };
        }

        Ok(left)
    }

    #[inline]
    fn parse_statement(&mut self) -> Result<Statement, Error> {
        let statement = match self.current_token {
            Token::Let => self.parse_declare_statement()?,
            Token::OpenBrace => Statement::Block(self.parse_block_statement()?),
            Token::Return => self.parse_return_statement()?,
            _ => Statement::Expression(self.parse_expr(Precedence::Lowest)?),
        };

        self.skip_optional(Token::Semicolon);
        Ok(statement)
    }

    fn parse_declare_statement(&mut self) -> Result<Statement, Error> {
        self.advance();

        let identifier = match self.current_token {
            Token::Identifier(name) => Ok(name.to_owned()),
            _ => Err(format!(
                "Unexpected token. expected an identifier, got a {:?}",
                self.current_token
            )),
        }?;

        self.advance();

        self.skip(Token::Assign)?;

        let value = self.parse_expr(Precedence::Lowest)?;
        Ok(Statement::Let(identifier, value))
    }

    fn parse_op_assign_expression(
        &mut self,
        left: Expr,
        operator: Operator,
    ) -> Result<Expr, Error> {
        self.advance();
        let right = self.parse_expr(Precedence::Lowest)?;
        Ok(Expr::Assignment {
            left: Box::new(left.clone()),
            right: Box::new(Expr::Infix {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            }),
        })
    }

    fn parse_infix_expr(&mut self, left: Expr) -> Result<Expr, Error> {
        let operator = self.parse_operator();
        let precedence = self.current_token.precedence();
        self.advance();

        if self.current_token == Token::Assign && matches!(left, Expr::Identifier(_)) {
            return self.parse_op_assign_expression(left, operator);
        }

        let right = self.parse_expr(precedence)?;
        Ok(Expr::Infix {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    fn parse_prefix_expr(&mut self) -> Result<Expr, Error> {
        let operator = self.parse_operator();
        let precedence = self.current_token.precedence();

        self.advance();
        Ok(Expr::Prefix {
            operator,
            right: Box::new(self.parse_expr(precedence)?),
        })
    }

    fn parse_if_expr(&mut self) -> Result<Expr, Error> {
        self.advance();

        let condition = self.parse_expr(Precedence::Lowest)?;
        let consequence = self.parse_block_statement()?;
        let alternative = if self.current_token == Token::Else {
            self.advance();

            if self.current_token == Token::If {
                Some(vec![self.parse_statement()?])
            } else {
                Some(self.parse_block_statement()?)
            }
        } else {
            None
        };

        Ok(Expr::If {
            condition: Box::new(condition),
            consequence,
            alternative,
        })
    }

    fn parse_prop_access_expr(&mut self, left: Expr) -> Result<Expr, Error> {
        match left {
            Expr::Identifier(_) => (),
            _ => {
                return Err(format!(
                    "Cannot access properties of expressions of type {left:?}"
                ))
            }
        }

        self.advance();
        if self.current_token == Token::OpenBracket {
            self.advance(); // [
            let right = self.parse_expr(Precedence::Lowest)?; // Parse inside `[...]`
            self.skip(Token::CloseBracket)?; // ]
            Ok(Expr::Member {
                left: Box::new(left),
                right: Box::new(right),
                computed: true,
            })
        } else {
            // Parse dot-access (non-computed)
            let right = match self.current_token {
                Token::Identifier(name) => {
                    self.advance();
                    Expr::String {
                        value: name.to_string(),
                    }
                }
                _ => {
                    return Err(format!(
                        "Expected property name after '.', found {:?}",
                        self.current_token
                    ))
                }
            };

            Ok(Expr::Member {
                left: Box::new(left),
                right: Box::new(right),
                computed: false,
            })
        }
    }

    fn parse_assign_expr(&mut self, left: Expr) -> Result<Expr, Error> {
        match left {
            Expr::Identifier(_) | Expr::Index { .. } => (),
            _ => {
                return Err(format!(
                    "Cannot assign a value to expressions of type {left:?}"
                ))
            }
        }

        self.advance();
        let right = self.parse_expr(Precedence::Assign)?;

        Ok(Expr::Assignment {
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    #[inline]
    fn parse_int_expression(&mut self, strval: &str) -> Expr {
        self.advance();
        Expr::Int {
            value: strval.parse().unwrap(),
        }
    }

    #[inline]
    fn parse_bool_expression(&mut self, value: bool) -> Expr {
        self.advance();
        Expr::Bool { value }
    }

    fn parse_function_expr(&mut self) -> Result<Expr, Error> {
        self.advance();

        let name = match self.current_token {
            Token::Identifier(name) => {
                self.advance();
                name
            }
            _ => "",
        };

        let mut parameters = vec![];
        self.skip(Token::OpenParenthese)?;

        while self.current_token != Token::CloseParenthese {
            if let Token::Identifier(name) = self.current_token {
                parameters.push(name.to_owned());
                self.advance();
                self.skip_optional(Token::Comma);
            }
        }
        self.skip(Token::CloseParenthese)?;

        let body = self.parse_block_statement()?;
        Ok(Expr::Function {
            name: name.to_owned(),
            parameters,
            body,
        })
    }

    fn parse_ident(&mut self, name: &str) -> Expr {
        let expr = Expr::Identifier(name.to_owned());
        self.advance();
        expr
    }

    fn parse_call_expr(&mut self, left: Expr) -> Result<Expr, Error> {
        match left {
            Expr::Identifier(_) | Expr::Function { .. } => {}
            _ => return Err(format!("Expressions of type {left:?} are not callable")),
        }

        self.advance();

        let mut arguments = vec![];
        while self.current_token != Token::CloseParenthese {
            arguments.push(self.parse_expr(Precedence::Lowest)?);
            self.skip_optional(Token::Comma);
        }

        self.advance();

        Ok(Expr::Call {
            left: Box::new(left),
            arguments,
        })
    }

    fn parse_array_expr(&mut self) -> Result<Expr, Error> {
        self.advance();

        let mut values = Vec::new();
        while self.current_token != Token::CloseBracket {
            values.push(self.parse_expr(Precedence::Lowest)?);
            self.skip_optional(Token::Comma);
        }

        self.skip(Token::CloseBracket)?;
        Ok(Expr::Array { values })
    }

    fn parse_index_expr(&mut self, left: Expr) -> Result<Expr, Error> {
        match &left {
            Expr::Identifier(_) | Expr::Array { .. } | Expr::String { .. } => (),
            _ => return Err(format!("Cannot index in expressions of type {left:?}")),
        }

        self.advance();
        let index = self.parse_expr(Precedence::Lowest)?;
        self.skip(Token::CloseBracket)?;
        Ok(Expr::Index {
            left: Box::new(left),
            index: Box::new(index),
        })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, Error> {
        self.advance();

        let expr = self.parse_expr(Precedence::Lowest)?;
        Ok(Statement::Return(expr))
    }

    fn parse_block_statement(&mut self) -> Result<BlockStatement, Error> {
        let mut block = BlockStatement::with_capacity(8);
        self.skip(Token::OpenBrace)?;

        while self.current_token != Token::Unknown && self.current_token != Token::CloseBrace {
            block.push(self.parse_statement()?);
        }

        self.skip(Token::CloseBrace)?;
        Ok(block)
    }
}

/// Parses the program string into an AST representation
pub fn parse(program: &str) -> Result<BlockStatement, Error> {
    let mut parser = Parser::new(program);
    let mut block = BlockStatement::new();

    while parser.current_token != Token::Unknown {
        block.push(parser.parse_statement()?);
    }

    Ok(block)
}
