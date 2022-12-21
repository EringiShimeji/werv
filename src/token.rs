#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
    /// =
    Assign,
    /// ,
    Comma,
    /// ;
    SemiColon,
    /// +
    Plus,
    /// -
    Minus,
    /// *
    Asterisk,
    /// /
    Slash,
    /// %
    Percent,
    /// !
    Bang,
    /// (
    LParen,
    /// )
    RParen,
    /// {
    LBrace,
    /// }
    RBrace,

    /// ==
    Eq,
    /// !=
    Ne,
    /// <
    Lt,
    /// <=
    Le,
    /// >
    Gt,
    /// >=
    Ge,

    // キーワード
    Let,
    Return,
    If,
    Else,
    True,
    False,

    /// 識別子
    Ident,
    /// 数字
    Number,
    /// 文字列
    Str,

    Unknown,
    EOF,
}
impl TokenKind {
    pub fn lookup_kind(literal: &str) -> TokenKind {
        match literal {
            "let" => TokenKind::Let,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            _ => TokenKind::Ident,
        }
    }
}
impl Default for TokenKind {
    fn default() -> Self {
        TokenKind::Unknown
    }
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Token {
    kind: TokenKind,
    literal: String,
}
impl Token {
    pub fn new(kind: TokenKind, literal: String) -> Token {
        Token { kind, literal }
    }
    pub fn kind(&self) -> TokenKind {
        self.kind
    }
    pub fn literal(&self) -> String {
        self.literal.clone()
    }
}
