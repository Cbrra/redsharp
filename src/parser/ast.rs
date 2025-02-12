use super::lexer::Token;

#[derive(PartialEq, Debug, PartialOrd, Clone)]
pub enum Statement {
    Let(String, Expr),
    Expression(Expr),
    Block(BlockStatement),
    Return(Expr),
}

pub type BlockStatement = Vec<Statement>;

#[derive(PartialEq, Debug, PartialOrd, Clone)]
pub enum Expr {
    Identifier(String),
    Int {
        value: u32,
    },
    Bool {
        value: bool,
    },
    Prefix {
        operator: Operator,
        right: Box<Expr>,
    },
    Infix {
        left: Box<Expr>,
        operator: Operator,
        right: Box<Expr>,
    },
    Break,
    Loop {
        body: BlockStatement,
    },
    If {
        condition: Box<Expr>,
        consequence: BlockStatement,
        alternative: Option<BlockStatement>,
    },
    Function {
        name: String,
        parameters: Vec<String>,
        body: BlockStatement,
    },
    Call {
        left: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Assignment {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Member {
        left: Box<Expr>,
        right: Box<Expr>,
        computed: bool,
    },
    String {
        value: String,
    },
    Array {
        values: Vec<Expr>,
    },
    Index {
        left: Box<Expr>,
        index: Box<Expr>,
    },
}

#[derive(PartialEq, Eq, Debug, PartialOrd, Clone)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Gt,
    Gte,
    Lt,
    Lte,
    Eq,
    Ne,
    Not,
    Negate,
    And,
    Or,
    Modulo,
    Assign,
}

impl From<Token<'_>> for Operator {
    fn from(value: Token) -> Self {
        match value {
            Token::Plus => Operator::Add,
            Token::Minus => Operator::Subtract,
            Token::Slash => Operator::Divide,
            Token::Star => Operator::Multiply,
            Token::Caret => Operator::Power,
            Token::Percent => Operator::Modulo,
            Token::And => Operator::And,
            Token::Or => Operator::Or,
            Token::Gt => Operator::Gt,
            Token::Gte => Operator::Gte,
            Token::Lt => Operator::Lt,
            Token::Lte => Operator::Lte,
            Token::Eq => Operator::Eq,
            Token::Ne => Operator::Ne,
            Token::Not => Operator::Not,
            Token::Assign => Operator::Assign,
            _ => unimplemented!(
                "Parsing token {:?} into operator is not implemented.",
                value
            ),
        }
    }
}
