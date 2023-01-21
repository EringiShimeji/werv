#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Token {
    kind: TokenKind,
    literal: String,
}
impl Token {
    pub fn new(kind: TokenKind, literal: impl ToString) -> Token {
        Token {
            kind,
            literal: literal.to_string(),
        }
    }
    pub fn kind(&self) -> TokenKind {
        self.kind
    }
    pub fn literal(&self) -> &str {
        &self.literal
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
    Unknown,
    EOF,

    Number,
    Ident,

    Assign,
    Plus,
    Minus,
    Asterisk,
    Slash,

    LParen,
    RParen,
    SemiColon,

    Let,
}

impl Default for TokenKind {
    fn default() -> Self {
        TokenKind::EOF
    }
}

impl TokenKind {
    pub fn lookup_ident(literal: &str) -> TokenKind {
        match literal {
            "let" => Self::Let,
            _ => Self::Ident,
        }
    }
}
