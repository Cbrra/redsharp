use super::lexer::Token;

/// The order here is important
#[derive(PartialOrd, PartialEq)]
pub enum Precedence {
    Lowest,
    Assign,
    OrAnd,
    Equals,
    LessGreater,
    Sum,
    Product,
    Power,
    Method,
    Call,
    Index,
}

impl Token<'_> {
    pub fn precedence(&self) -> Precedence {
        match self {
            Token::Assign => Precedence::Assign,
            Token::Or | Token::And => Precedence::OrAnd,
            Token::Lt | Token::Gt | Token::Lte | Token::Gte => Precedence::LessGreater,
            Token::Eq | Token::Ne => Precedence::Equals,
            Token::Plus | Token::Minus => Precedence::Sum,
            Token::Slash | Token::Star | Token::Percent => Precedence::Product,
            Token::Caret => Precedence::Power,
            Token::Dot => Precedence::Method,
            Token::OpenParenthese => Precedence::Call,
            Token::OpenBracket => Precedence::Index,
            _ => Precedence::Lowest,
        }
    }
}