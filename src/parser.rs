mod error;
mod escape;
#[cfg(test)]
mod test;
use crate::{
    ast::{
        BinaryExprKind::*, Expression, Expression::*, Node, Statement, Statement::*,
        UnaryExprKind::*,
    },
    lexer::{error::LexerError, Lexer},
    token::{Token, TokenKind},
};

use self::{
    error::{ParseError, ParseError::*},
    escape::escape,
};

type PResult<T> = Result<T, ParseError>;

pub struct Parser {
    lexer: Lexer,
    cur_token: Token,
}
impl Parser {
    pub fn new(lexer: Lexer) -> Parser {
        let mut p = Parser {
            lexer,
            cur_token: Token::default(),
        };

        p.next_token();
        p
    }

    fn is_eof(&self) -> bool {
        self.cur_token.kind() == TokenKind::EOF
    }

    fn next_token(&mut self) -> Option<LexerError> {
        self.cur_token = self.lexer.next_token().ok()?;

        None
    }

    fn consume(&mut self, kind: TokenKind) -> PResult<Token> {
        if self.cur_token.kind() != kind {
            return Err(ParseConsumeError {
                expected: kind,
                got: self.cur_token.kind(),
            });
        }

        let token = self.cur_token.clone();

        self.next_token()
            .map_or(Ok(()), |e| Err(ParseNextTokenError(e)))?;
        Ok(token)
    }

    fn is_cur(&self, kind: TokenKind) -> bool {
        self.cur_token.kind() == kind
    }

    pub fn parse(&mut self) -> PResult<Node> {
        let mut stmts = Vec::new();

        while !self.is_eof() {
            stmts.push(self.parse_stmt()?);
        }

        Ok(Node::Program(stmts))
    }

    fn parse_stmt(&mut self) -> PResult<Statement> {
        match self.cur_token.kind() {
            TokenKind::Let => self.parse_let_stmt(),
            TokenKind::Return => self.parse_return_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    /// let_stmt = "let" ident ( "(" ( ident ( "," ident )* )? ")" )? "=" expr ";"
    fn parse_let_stmt(&mut self) -> PResult<Statement> {
        self.consume(TokenKind::Let)?;

        let name = self.parse_ident()?;

        // ident "(" ( ident ( "," ident )* )? ")" "=" expr ";"
        if self.consume(TokenKind::LParen).is_ok() {
            let mut params = Vec::new();

            // ident "(" ")" "=" expr ";"
            if self.consume(TokenKind::RParen).is_ok() {
                self.consume(TokenKind::Assign)?;

                let body = self.parse_expr()?;

                self.consume(TokenKind::SemiColon)?;

                return Ok(LetFnStmt { name, params, body });
            }

            // ident "(" ident ")"
            params.push(self.parse_ident()?);

            // ident "(" ident ( "," ident )* ")" "=" expr ";"
            params.push(self.parse_expr()?);
            while !self.consume(TokenKind::RParen).is_ok() {
                if self.is_eof() {
                    return Err(ParseCallExprError);
                }

                self.consume(TokenKind::Comma)?;

                params.push(self.parse_ident()?);
            }

            self.consume(TokenKind::Assign)?;

            let body = self.parse_expr()?;

            self.consume(TokenKind::SemiColon)?;

            return Ok(LetFnStmt { name, params, body });
        };

        self.consume(TokenKind::Assign)?;

        let value = self.parse_expr()?;

        self.consume(TokenKind::SemiColon)?;

        Ok(LetStmt { name, value })
    }

    /// return_stmt = "return" expr ";"
    fn parse_return_stmt(&mut self) -> PResult<Statement> {
        self.consume(TokenKind::Return)?;

        let expr = self.parse_expr()?;

        self.consume(TokenKind::SemiColon)?;

        Ok(ReturnStmt(expr))
    }

    /// expr_stmt = expr ";"?
    fn parse_expr_stmt(&mut self) -> PResult<Statement> {
        let expr = self.parse_expr()?;

        self.consume(TokenKind::SemiColon);

        Ok(ExprStmt(expr))
    }

    fn parse_expr(&mut self) -> PResult<Expression> {
        if self.is_cur(TokenKind::If) {
            return self.parse_if_expr();
        }

        if self.is_cur(TokenKind::LBrace) {
            return self.parse_block_expr();
        }

        self.parse_comp()
    }

    /// if_expr = "if" expr expr ( "else" expr )?
    fn parse_if_expr(&mut self) -> PResult<Expression> {
        self.consume(TokenKind::If)?;

        let condition = Box::new(self.parse_expr()?);
        let consequence = Box::new(self.parse_expr()?);
        let alternative = if self.consume(TokenKind::Else).is_ok() {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        Ok(IfExpr {
            condition,
            consequence,
            alternative,
        })
    }

    /// block_expr = "{" stmt* "}"
    fn parse_block_expr(&mut self) -> PResult<Expression> {
        self.consume(TokenKind::LBrace)?;

        let mut stmts = Vec::new();

        while !self.consume(TokenKind::RBrace).is_ok() {
            if self.is_eof() {
                return Err(ParseBlockExprError);
            }

            stmts.push(self.parse_stmt()?);
        }

        Ok(BlockExpr(stmts))
    }

    /// comp = add ( "==" add | "!=" add | "<" add | "<=" add | ">" add | ">=" add )
    fn parse_comp(&mut self) -> PResult<Expression> {
        let mut node = self.parse_add()?;

        if self.consume(TokenKind::Eq).is_ok() {
            node = BinaryExpr {
                kind: Eq,
                lhs: Box::new(node),
                rhs: Box::new(self.parse_add()?),
            };
        } else if self.consume(TokenKind::Ne).is_ok() {
            node = BinaryExpr {
                kind: Ne,
                lhs: Box::new(node),
                rhs: Box::new(self.parse_add()?),
            };
        } else if self.consume(TokenKind::Lt).is_ok() {
            node = BinaryExpr {
                kind: Lt,
                lhs: Box::new(node),
                rhs: Box::new(self.parse_add()?),
            };
        } else if self.consume(TokenKind::Le).is_ok() {
            node = BinaryExpr {
                kind: Le,
                lhs: Box::new(node),
                rhs: Box::new(self.parse_add()?),
            };
        } else if self.consume(TokenKind::Gt).is_ok() {
            node = BinaryExpr {
                kind: Gt,
                lhs: Box::new(node),
                rhs: Box::new(self.parse_add()?),
            };
        } else if self.consume(TokenKind::Ge).is_ok() {
            node = BinaryExpr {
                kind: Ge,
                lhs: Box::new(node),
                rhs: Box::new(self.parse_add()?),
            };
        }

        Ok(node)
    }

    /// add = mul ( "+" mul | "-" mul )*
    fn parse_add(&mut self) -> PResult<Expression> {
        let mut node = self.parse_mul()?;

        loop {
            if self.consume(TokenKind::Plus).is_ok() {
                node = BinaryExpr {
                    kind: Add,
                    lhs: Box::new(node),
                    rhs: Box::new(self.parse_mul()?),
                };
            } else if self.consume(TokenKind::Minus).is_ok() {
                node = BinaryExpr {
                    kind: Sub,
                    lhs: Box::new(node),
                    rhs: Box::new(self.parse_mul()?),
                };
            } else {
                break;
            }
        }

        Ok(node)
    }

    /// mul = unary ( "*" unary | "/" unary | "%" unary )*
    fn parse_mul(&mut self) -> PResult<Expression> {
        let mut node = self.parse_unary()?;

        loop {
            if self.consume(TokenKind::Asterisk).is_ok() {
                node = BinaryExpr {
                    kind: Mul,
                    lhs: Box::new(node),
                    rhs: Box::new(self.parse_unary()?),
                };
            } else if self.consume(TokenKind::Slash).is_ok() {
                node = BinaryExpr {
                    kind: Div,
                    lhs: Box::new(node),
                    rhs: Box::new(self.parse_unary()?),
                };
            } else if self.consume(TokenKind::Percent).is_ok() {
                node = BinaryExpr {
                    kind: Mod,
                    lhs: Box::new(node),
                    rhs: Box::new(self.parse_unary()?),
                };
            } else {
                break;
            }
        }

        Ok(node)
    }

    /// unary = ( "-" | "!" )? primary
    fn parse_unary(&mut self) -> PResult<Expression> {
        // "-" primary
        if self.consume(TokenKind::Minus).is_ok() {
            return Ok(UnaryExpr {
                kind: Minus,
                expr: Box::new(self.parse_primary()?),
            });
        }

        // "!" primary
        if self.consume(TokenKind::Bang).is_ok() {
            return Ok(UnaryExpr {
                kind: Not,
                expr: Box::new(self.parse_primary()?),
            });
        }

        self.parse_primary()
    }

    /// primary = integer | ident ( "(" ( expr ( "," expr )* )? ")" )? | str | bool | "(" expr ")"
    fn parse_primary(&mut self) -> PResult<Expression> {
        // "(" expr ")"
        if self.consume(TokenKind::LParen).is_ok() {
            let expr = self.parse_expr()?;

            self.consume(TokenKind::RParen)?;
            return Ok(expr);
        }

        // ident ( "(" ( expr ( "," expr )* )? ")" )?
        if self.is_cur(TokenKind::Ident) {
            let name = self.parse_ident()?;

            if self.consume(TokenKind::LParen).is_ok() {
                let mut args = Vec::new();

                // ident "(" ")"
                if self.consume(TokenKind::RParen).is_ok() {
                    return Ok(CallExpr {
                        name: Box::new(name),
                        args,
                    });
                }

                // ident "(" expr ")"
                args.push(self.parse_expr()?);

                // ident "(" expr ( "," expr )* ")"
                args.push(self.parse_expr()?);
                while !self.consume(TokenKind::RParen).is_ok() {
                    if self.is_eof() {
                        return Err(ParseCallExprError);
                    }

                    self.consume(TokenKind::Comma)?;

                    args.push(self.parse_expr()?);
                }

                return Ok(CallExpr {
                    name: Box::new(name),
                    args,
                });
            }

            return Ok(name);
        }

        // str
        if self.is_cur(TokenKind::Str) {
            return self.parse_str();
        }

        // bool
        if self.is_cur(TokenKind::True) || self.is_cur(TokenKind::False) {
            return self.parse_bool();
        }

        return self.parse_integer();
    }

    /// integer = [0-9]*
    fn parse_integer(&mut self) -> PResult<Expression> {
        let token = self.consume(TokenKind::Number)?;
        let value = token
            .literal()
            .parse::<isize>()
            .or(Err(ParseIntegerError))?;

        Ok(Integer(value))
    }

    /// ident = [A-z] | "_" ( [A-z0-9] | "_" )*
    fn parse_ident(&mut self) -> PResult<Expression> {
        let token = self.consume(TokenKind::Ident)?;

        Ok(Ident(token.literal()))
    }

    /// str = '"' ( (char except double quote and backslash) | "\" . ) '"'
    fn parse_str(&mut self) -> PResult<Expression> {
        let str = self.consume(TokenKind::Str)?;
        let str = escape(&str.literal())?;

        Ok(Str(str))
    }

    /// bool = "true" | "false"
    fn parse_bool(&mut self) -> PResult<Expression> {
        if self.consume(TokenKind::True).is_ok() {
            return Ok(Boolean(true));
        }

        self.consume(TokenKind::False)?;
        Ok(Boolean(false))
    }
}
