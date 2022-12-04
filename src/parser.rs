use crate::{
    ast::{BinaryExprKind::*, Expression, Expression::*, Statement, Statement::*},
    lexer::Lexer,
    token::{Token, TokenKind},
};
#[cfg(test)]
mod test;

type PResult<T> = Result<T, ()>;

pub struct Parser {
    lexer: Lexer,
    cur_token: Token,
    peek_token: Token,
}
impl Parser {
    pub fn new(lexer: Lexer) -> Parser {
        let mut p = Parser {
            lexer,
            cur_token: Token::default(),
            peek_token: Token::default(),
        };

        p.next_token();
        p.next_token();
        p
    }

    pub fn parse(&mut self) -> PResult<Vec<Statement>> {
        let mut stmts = Vec::new();

        while !self.is_eof() {
            stmts.push(self.parse_statement()?);
        }

        Ok(stmts)
    }

    fn parse_statement(&mut self) -> PResult<Statement> {
        let stmt = match self.cur_token.kind() {
            TokenKind::Let => self.parse_let_statement()?,
            _ => return Err(()),
        };

        self.consume(TokenKind::SemiColon)?;
        Ok(stmt)
    }

    fn parse_let_statement(&mut self) -> PResult<Statement> {
        self.consume(TokenKind::Let)?;

        let name = self.parse_ident()?;

        self.consume(TokenKind::Assign)?;

        let value = self.parse_expression()?;

        Ok(LetStatement { name, value })
    }

    fn parse_expression(&mut self) -> PResult<Expression> {
        self.parse_add()
    }

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

    fn parse_mul(&mut self) -> PResult<Expression> {
        let mut node = self.parse_primary()?;

        loop {
            if self.consume(TokenKind::Asterisk).is_ok() {
                node = BinaryExpr {
                    kind: Mul,
                    lhs: Box::new(node),
                    rhs: Box::new(self.parse_primary()?),
                };
            } else if self.consume(TokenKind::Slash).is_ok() {
                node = BinaryExpr {
                    kind: Div,
                    lhs: Box::new(node),
                    rhs: Box::new(self.parse_primary()?),
                };
            } else {
                break;
            }
        }

        Ok(node)
    }

    fn parse_primary(&mut self) -> PResult<Expression> {
        if self.consume(TokenKind::LParen).is_ok() {
            let expr = self.parse_expression()?;

            self.consume(TokenKind::RParen)?;
            return Ok(expr);
        }

        if self.is_cur(TokenKind::Ident) {
            return self.parse_ident();
        }

        return self.parse_integer();
    }

    fn parse_integer(&mut self) -> PResult<Expression> {
        let token = self.consume(TokenKind::Number)?;
        let value = token.literal().parse::<isize>().or(Err(()))?;

        Ok(Integer(value))
    }

    fn parse_ident(&mut self) -> PResult<Expression> {
        let token = self.consume(TokenKind::Ident)?;
        Ok(Ident(token.literal()))
    }

    fn is_eof(&self) -> bool {
        self.cur_token.kind() == TokenKind::EOF
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn consume(&mut self, kind: TokenKind) -> PResult<Token> {
        if self.cur_token.kind() != kind {
            return Err(());
        }

        let token = self.cur_token.clone();

        self.next_token();
        Ok(token)
    }

    fn is_cur(&self, kind: TokenKind) -> bool {
        self.cur_token.kind() == kind
    }

    fn is_peek(&self, kind: TokenKind) -> bool {
        self.peek_token.kind() == kind
    }
}
