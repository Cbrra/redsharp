use std::str::Chars;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Token<'a> {
    Identifier(&'a str),

    // Types
    Int(&'a str),

    // Keywords
    If,
    Let,
    Else,
    Return,
    Func,
    True,
    False,

    // Operators
    Lte,
    Gte,
    Eq,
    Ne,
    And,
    Or,
    Lt,
    Gt,
    /// "!"
    Not,
    /// "-"
    Minus,
    /// "+"
    Plus,
    /// "*"
    Star,
    /// "/"
    Slash,
    /// "^"
    Caret,
    /// "%"
    Percent,
    /// "="
    Assign,

    // Punctuations
    Semicolon,
    Colon,
    Comma,
    Dot,

    // Parentheses
    OpenParenthese,
    CloseParenthese,
    /// "{"
    OpenBrace,
    /// "}"
    CloseBrace,
    /// "["
    OpenBracket,
    /// "]"
    CloseBracket,

    /// Unknown token
    Unknown,
}

impl<'a> From<&'a str> for Token<'a> {
    fn from(value: &'a str) -> Self {
        match value {
            "if" => Token::If,
            "let" => Token::Let,
            "return" => Token::Return,
            "else" => Token::Else,
            "fn" => Token::Func,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Identifier(value),
        }
    }
}

/// The language tokenizer
pub struct Tokenizer<'a> {
    pos: usize,
    input: &'a str,
    chars: Chars<'a>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &str) -> Tokenizer {
        Tokenizer {
            pos: 0,
            input,
            chars: input.chars(),
        }
    }

    #[inline]
    fn eat(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    #[inline]
    fn peek(&self) -> Option<char> {
        // `.next()` optimizes better than `.nth(0)`
        self.chars.clone().next()
    }

    #[inline]
    fn is_eof(&self) -> bool {
        self.offset() >= self.input.len()
    }

    /// Current offset
    #[inline]
    fn offset(&self) -> usize {
        self.pos
    }

    /// Takes a string slice from the input
    #[inline]
    fn read_str(&self, from: usize, to: usize) -> &'a str {
        &self.input[from..to]
    }

    #[inline]
    fn skip_while(&mut self, mut predicate: impl FnMut(char, bool) -> bool) {
        // It was tried making optimized version of this for eg. line comments, but
        // LLVM can inline all of this and compile it down to fast iteration over bytes.
        let mut escaped = false;
        while !self.is_eof() && predicate(self.peek().unwrap(), escaped) {
            escaped = self.eat() == Some('\\');
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.offset();
        let token = match self.eat()? {
            // Identifiers
            c if c.is_alphabetic() || c == '_' => {
                // first char must be alphabetic, but consecutive chars can have integers
                self.skip_while(|c, _| c.is_alphanumeric() || c == '_');
                let ident = self.read_str(start, self.offset());
                ident.into()
            }

            // Integers & Floats
            '0'..='9' => {
                let mut decimal = false;
                self.skip_while(|c, _| {
                    if c.is_ascii_digit() {
                        return true;
                    }

                    if !decimal && c == '.' {
                        decimal = true;
                        return true;
                    }

                    false
                });
                let val = self.read_str(start, self.offset());
                if decimal {
                    // Token::Float(val)
                    panic!("Float are not yet implemented");
                } else {
                    Token::Int(val)
                }
            }

            // String values
            '"' => {
                self.skip_while(|c, esc| c != '"' || esc);

                // skip closing "
                self.eat()?;

                // Token::String(self.read_str(start + 1, self.offset() - 1))
                panic!("String are not yet implemented");
            }

            c if c.is_whitespace() => return self.next(),

            // Multi-char tokens:
            '=' => {
                if self.peek() == Some('=') {
                    Token::Eq
                } else {
                    Token::Assign
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    Token::Ne
                } else {
                    Token::Not
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    Token::Lte
                } else {
                    Token::Lt
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    Token::Gte
                } else {
                    Token::Gt
                }
            }
            '/' => {
                if self.peek() == Some('/') {
                    self.skip_while(|c, _| c != '\n');
                    return self.next();
                } else {
                    Token::Slash
                }
            }
            '&' if self.peek() == Some('&') => Token::And,
            '|' if self.peek() == Some('|') => Token::Or,
            ';' => Token::Semicolon,
            ':' => Token::Colon,
            ',' => Token::Comma,
            '.' => Token::Dot,
            '(' => Token::OpenParenthese,
            ')' => Token::CloseParenthese,
            '{' => Token::OpenBrace,
            '}' => Token::CloseBrace,
            '[' => Token::OpenBracket,
            ']' => Token::CloseBracket,
            '-' => Token::Minus,
            '+' => Token::Plus,
            '*' => Token::Star,
            '^' => Token::Caret,
            '%' => Token::Percent,
            _ => Token::Unknown,
        };

        // If we parsed a multi-char token,
        // eat iterator appropriate number of times
        match token {
            Token::Eq | Token::Ne | Token::Gte | Token::Lte | Token::And | Token::Or => {
                self.eat()
            }
            _ => None,
        };

        Some(token)
    }
}
