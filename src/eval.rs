mod error;
#[cfg(test)]
mod test;

use self::error::{EvalError, EvalError::*};
use crate::{
    ast::{BinaryExprKind, Expression, Node, Statement, UnaryExprKind},
    builtin::call_builtin,
    environment::Environment,
    object::{Object, Object::*, NULL},
};

type EResult = Result<Object, EvalError>;

pub fn eval(node: impl Into<Node>, env: &mut Environment) -> EResult {
    let node: Node = node.into();

    match node {
        Node::Program(stmts) => eval_statements(stmts, env),
        Node::Statement(s) => eval_statement(s, env),
        Node::Expression(e) => eval_expression(e, env),
    }
}

fn eval_statements(stmts: Vec<Statement>, env: &mut Environment) -> EResult {
    let mut result = NULL;

    for s in stmts {
        result = eval(s, env)?;

        // もしevalが値を返したなら、BlockReturnStatementかReturnStatementが評価されたということなので、
        // 即座に値を返す
        if result != NULL {
            return Ok(result);
        }
    }

    Ok(result)
}

fn eval_statement(s: Statement, env: &mut Environment) -> EResult {
    match s {
        // TODO: 今はデバッグのためにExprStatementが値を返すようになっているが、本来は返さない
        Statement::ExprStmt(e) => eval(e, env),
        Statement::BlockReturnStmt(e) => eval_block_return_stmt(e, env),
        Statement::LetStmt { name, value } => eval_let_stmt(name, value, env),
        Statement::LetFnStmt { name, params, body } => eval_fn_def_stmt(name, params, body, env),
        Statement::ReturnStmt(e) => eval(e, env),
    }
}

fn eval_fn_def_stmt(
    name: Expression,
    params: Vec<Expression>,
    body: Expression,
    env: &mut Environment,
) -> EResult {
    if let Expression::Ident(name) = name {
        env.insert(name, Function { params, body });
    } else {
        return Err(EvalFunctionDefinitionStatementError);
    }

    Ok(NULL)
}

fn eval_let_stmt(name: Expression, value: Expression, env: &mut Environment) -> EResult {
    let value = eval(value, env)?;

    if let Expression::Ident(name) = name {
        env.insert(name, value);
    } else {
        return Err(EvalLetStatementError);
    }

    Ok(NULL)
}

fn eval_block_return_stmt(expr: Expression, env: &mut Environment) -> EResult {
    eval(expr, env)
}

fn eval_expression(e: Expression, env: &mut Environment) -> EResult {
    match e {
        Expression::IfExpr {
            condition,
            consequence,
            alternative,
        } => eval_if_expr(condition, consequence, alternative, env),
        Expression::BlockExpr(stmts) => eval_block(stmts, env),
        Expression::UnaryExpr { kind, expr } => eval_unary_expr(kind, expr, env),
        Expression::BinaryExpr { kind, lhs, rhs } => eval_binary_expr(kind, lhs, rhs, env),
        Expression::Integer(i) => eval_integer(i),
        Expression::Boolean(b) => eval_boolean(b),
        Expression::Ident(i) => eval_ident(i, env),
        Expression::CallExpr { name, args } => eval_call_expr(name, args, env),
        Expression::Str(s) => eval_string(s),
    }
}

fn eval_call_expr(name: Box<Expression>, args: Vec<Expression>, env: &mut Environment) -> EResult {
    if let Expression::Ident(name) = name.as_ref() {
        if let Ok(res) = call_builtin(name, &args, env) {
            return Ok(res);
        }
    }

    let name = eval(*name, env)?;

    if let Function { params, body } = name {
        if params.len() != args.len() {
            return Err(EvalCallExprError);
        }

        let mut env = env.clone();

        // map params and args and insert to env
        for i in 0..params.len() {
            let param = &params[i];
            let arg = args[i].clone();

            if let Expression::Ident(param) = param {
                let arg = eval(arg, &mut env)?;

                env.insert(param.clone(), arg);
            } else {
                return Err(EvalCallExprError);
            }
        }

        return eval(body, &mut env);
    }

    Err(EvalCallExprError)
}

fn eval_block(stmts: Vec<Statement>, env: &mut Environment) -> EResult {
    eval_statements(stmts, &mut env.clone())
}

fn eval_if_expr(
    condition: Box<Expression>,
    consequence: Box<Expression>,
    alternative: Option<Box<Expression>>,
    env: &mut Environment,
) -> EResult {
    let condition = eval(*condition, env)?;

    if condition == Boolean(true) {
        return eval(*consequence, env);
    } else if let Some(alternative) = alternative {
        return eval(*alternative, env);
    }

    Ok(NULL)
}

fn eval_unary_expr(kind: UnaryExprKind, expr: Box<Expression>, env: &mut Environment) -> EResult {
    let expr = eval(*expr, env)?;

    if let (UnaryExprKind::Not, Object::Boolean(b)) = (kind, &expr) {
        return eval_boolean(!b);
    }
    if let (UnaryExprKind::Minus, Object::Integer(i)) = (kind, &expr) {
        return eval_integer(-i);
    }

    Err(EvalUnaryExpressionError)
}

fn eval_binary_expr(
    kind: BinaryExprKind,
    lhs: Box<Expression>,
    rhs: Box<Expression>,
    env: &mut Environment,
) -> EResult {
    let lhs = eval(*lhs, env)?;
    let rhs = eval(*rhs, env)?;

    if kind == BinaryExprKind::Eq {
        return Ok(Boolean(lhs == rhs));
    };
    if kind == BinaryExprKind::Ne {
        return Ok(Boolean(lhs != rhs));
    };

    if let (Integer(lhs), Integer(rhs)) = (&lhs, &rhs) {
        let result = match kind {
            BinaryExprKind::Add => Integer(lhs + rhs),
            BinaryExprKind::Sub => Integer(lhs - rhs),
            BinaryExprKind::Mul => Integer(lhs * rhs),
            BinaryExprKind::Div => Integer(lhs / rhs),
            BinaryExprKind::Mod => Integer(lhs % rhs),
            BinaryExprKind::Lt => Boolean(lhs < rhs),
            BinaryExprKind::Le => Boolean(lhs <= rhs),
            BinaryExprKind::Gt => Boolean(lhs > rhs),
            BinaryExprKind::Ge => Boolean(lhs >= rhs),
            _ => return Err(EvalBinaryExpressionError),
        };

        return Ok(result);
    } else if let (Str(lhs), Str(rhs)) = (lhs, rhs) {
        let result = match kind {
            BinaryExprKind::Add => Str(lhs + &rhs),
            BinaryExprKind::Lt => Boolean(lhs < rhs),
            BinaryExprKind::Le => Boolean(lhs <= rhs),
            BinaryExprKind::Gt => Boolean(lhs > rhs),
            BinaryExprKind::Ge => Boolean(lhs >= rhs),
            _ => return Err(EvalBinaryExpressionError),
        };

        return Ok(result);
    }

    Err(EvalBinaryExpressionError)
}

fn eval_integer(i: isize) -> EResult {
    Ok(Integer(i))
}

fn eval_boolean(b: bool) -> EResult {
    Ok(Boolean(b))
}

fn eval_string(s: String) -> EResult {
    Ok(Str(s))
}

fn eval_ident(i: String, env: &mut Environment) -> EResult {
    Ok(env.get(i).ok_or(EvalIdentError)?.clone())
}
