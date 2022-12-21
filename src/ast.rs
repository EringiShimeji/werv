#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Program(Vec<Statement>),
    Statement(Statement),
    Expression(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    LetStmt {
        name: Expression,
        value: Expression,
    },
    LetFnStmt {
        name: Expression,
        params: Vec<Expression>,
        body: Expression,
    },
    ReturnStmt(Expression),
    BlockReturnStmt(Expression),
    ExprStmt(Expression),
}
impl Into<Node> for Statement {
    fn into(self) -> Node {
        Node::Statement(self)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    UnaryExpr {
        kind: UnaryExprKind,
        expr: Box<Expression>,
    },
    IfExpr {
        condition: Box<Expression>,
        consequence: Box<Expression>,
        alternative: Option<Box<Expression>>,
    },
    BlockExpr(Vec<Statement>),
    CallExpr {
        name: Box<Expression>,
        args: Vec<Expression>,
    },
    BinaryExpr {
        kind: BinaryExprKind,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Ident(String),
    Integer(isize),
    Boolean(bool),
    Str(String),
}
impl Into<Node> for Expression {
    fn into(self) -> Node {
        Node::Expression(self)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnaryExprKind {
    Not,
    Minus,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinaryExprKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Le,
    Lt,
    Ge,
    Gt,
}
