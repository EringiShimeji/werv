mod error;
#[cfg(test)]
mod test;

use self::error::{EvalError, EvalError::*};
use crate::{
    ast::{BinaryExprKind, Expression, Node, Statement, UnaryExprKind},
    builtin::call_builtin,
    object::{Object, Object::*, NULL},
};
use std::{borrow::Borrow, collections::BTreeMap};

type EResult = Result<Object, EvalError>;

#[derive(Clone, Debug)]
pub struct Environment {
    store: BTreeMap<String, Object>,
    outer: Option<Box<Environment>>,
}
impl Environment {
    pub fn new(outer: Option<Box<Environment>>) -> Environment {
        Environment {
            store: BTreeMap::new(),
            outer,
        }
    }
    pub fn get(&self, key: &String) -> Option<&Object> {
        let inner = self.store.get(key);

        if inner == None {
            if let Some(outer) = &self.outer {
                return outer.get(key);
            }
        }

        inner
    }
    pub fn insert(&mut self, key: String, value: Object) -> Option<Object> {
        self.store.insert(key, value)
    }
    pub fn update(&mut self, key: String, value: Object) -> Result<Object, EvalError> {
        let inner = self.store.get(&key);

        if inner == None {
            if let Some(outer) = &mut self.outer {
                return outer.update(key, value);
            }

            return Err(EvalUpdateEnvError);
        }

        Ok(self.insert(key, value).unwrap())
    }

    pub fn eval(&mut self, node: impl Into<Node>) -> EResult {
        let node: Node = node.into();

        match node {
            Node::Program(stmts) => self.eval_statements(stmts),
            Node::Statement(s) => self.eval_statement(s),
            Node::Expression(e) => self.eval_expression(e),
        }
    }

    fn eval_statements(&mut self, stmts: Vec<Statement>) -> EResult {
        let mut result = NULL;

        for s in stmts {
            result = self.eval(s)?;

            // もしevalが値を返したなら、BlockReturnStatementかReturnStatementが評価されたということなので、
            // 即座に値を返す
            if result != NULL {
                return Ok(result);
            }
        }

        Ok(result)
    }

    fn eval_statement(&mut self, s: Statement) -> EResult {
        match s {
            Statement::ExprStmt { is_null, expr } => self.eval_expr_stmt(is_null, expr),
            Statement::LetStmt { name, value } => self.eval_let_stmt(name, value),
            Statement::LetFnStmt { name, params, body } => {
                self.eval_fn_def_stmt(name, params, body)
            }
            Statement::ReturnStmt(e) => self.eval(e),
            Statement::WhileStmt { condition, body } => self.eval_while_stmt(condition, body),
        }
    }

    fn eval_expr_stmt(&mut self, is_null: bool, expr: Expression) -> EResult {
        if is_null {
            self.eval(expr)?;

            Ok(NULL)
        } else {
            self.eval(expr)
        }
    }

    fn eval_while_stmt(&mut self, condition: Expression, body: Expression) -> EResult {
        loop {
            let cond = self.eval(condition.clone())?;

            if let Boolean(true) = cond {
                self.eval(body.clone())?;
            } else {
                break;
            }
        }

        Ok(NULL)
    }

    fn eval_fn_def_stmt(
        &mut self,
        name: Expression,
        params: Vec<Expression>,
        body: Expression,
    ) -> EResult {
        if let Expression::Ident(name) = name {
            self.insert(name, Function { params, body });
        } else {
            return Err(EvalFunctionDefinitionStatementError);
        }

        Ok(NULL)
    }

    fn eval_let_stmt(&mut self, name: Expression, value: Expression) -> EResult {
        let value = self.eval(value)?;

        if let Expression::Ident(name) = name {
            self.insert(name, value);
        } else {
            return Err(EvalLetStatementError);
        }

        Ok(NULL)
    }

    fn eval_expression(&mut self, e: Expression) -> EResult {
        match e {
            Expression::IfExpr {
                condition,
                consequence,
                alternative,
            } => self.eval_if_expr(condition, consequence, alternative),
            Expression::BlockExpr(stmts) => self.eval_block(stmts),
            Expression::UnaryExpr { kind, expr } => self.eval_unary_expr(kind, expr),
            Expression::BinaryExpr { kind, lhs, rhs } => self.eval_binary_expr(kind, lhs, rhs),
            Expression::Integer(i) => self.eval_integer(i),
            Expression::Boolean(b) => self.eval_boolean(b),
            Expression::Ident(i) => self.eval_ident(i),
            Expression::CallExpr { name, args } => self.eval_call_expr(name, args),
            Expression::Str(s) => self.eval_string(s),
            Expression::AssignExpr { name, value } => self.eval_assign_expr(name, value),
            Expression::Array(elems) => self.eval_array(elems),
            Expression::IndexExpr { expr: array, index } => self.eval_index_expr(array, index),
        }
    }

    fn eval_call_expr<'a>(&mut self, name: Box<Expression>, args: Vec<Expression>) -> EResult {
        if let Expression::Ident(name) = name.as_ref() {
            if let Ok(res) = call_builtin(name, &args, self) {
                return Ok(res);
            }
        }

        let name = self.eval(*name)?;

        if let Function { params, body } = name {
            if params.len() != args.len() {
                return Err(EvalCallExprError);
            }

            let mut env = Environment::new(Some(Box::new(self.clone())));

            // map params and args and insert to env
            for i in 0..params.len() {
                let param = &params[i];
                let arg = args[i].clone();

                if let Expression::Ident(param) = param {
                    let arg = env.eval(arg)?;

                    env.insert(param.clone(), arg);
                } else {
                    return Err(EvalCallExprError);
                }
            }

            return env.eval(body);
        }

        Err(EvalCallExprError)
    }

    fn eval_block(&mut self, stmts: Vec<Statement>) -> EResult {
        let mut env = Environment::new(Some(Box::new(self.clone())));
        let result = env.eval_statements(stmts);

        *self = *env.outer.unwrap();
        result
    }

    fn eval_if_expr(
        &mut self,
        condition: Box<Expression>,
        consequence: Box<Expression>,
        alternative: Option<Box<Expression>>,
    ) -> EResult {
        let condition = self.eval(*condition)?;

        if condition == Boolean(true) {
            return self.eval(*consequence);
        } else if let Some(alternative) = alternative {
            return self.eval(*alternative);
        }

        Ok(NULL)
    }

    fn eval_unary_expr(&mut self, kind: UnaryExprKind, expr: Box<Expression>) -> EResult {
        let expr = self.eval(*expr)?;

        if let (UnaryExprKind::Not, Object::Boolean(b)) = (kind, &expr) {
            return self.eval_boolean(!b);
        }
        if let (UnaryExprKind::Minus, Object::Integer(i)) = (kind, &expr) {
            return self.eval_integer(-i);
        }

        Err(EvalUnaryExpressionError)
    }

    fn eval_binary_expr(
        &mut self,
        kind: BinaryExprKind,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    ) -> EResult {
        let lhs = self.eval(*lhs)?;
        let rhs = self.eval(*rhs)?;

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
        } else if let (Str(lhs), Str(rhs)) = (&lhs, &rhs) {
            let result = match kind {
                BinaryExprKind::Add => Str(format!("{}{}", lhs, rhs)),
                BinaryExprKind::Lt => Boolean(lhs < rhs),
                BinaryExprKind::Le => Boolean(lhs <= rhs),
                BinaryExprKind::Gt => Boolean(lhs > rhs),
                BinaryExprKind::Ge => Boolean(lhs >= rhs),
                _ => return Err(EvalBinaryExpressionError),
            };

            return Ok(result);
        } else if let (Array(mut lhs), Array(mut rhs)) = (lhs, rhs) {
            let result = match kind {
                BinaryExprKind::Add => {
                    lhs.append(&mut rhs);
                    Array(lhs)
                }
                _ => return Err(EvalBinaryExpressionError),
            };

            return Ok(result);
        }

        Err(EvalBinaryExpressionError)
    }

    fn eval_integer(&self, i: isize) -> EResult {
        Ok(Integer(i))
    }

    fn eval_boolean(&self, b: bool) -> EResult {
        Ok(Boolean(b))
    }

    fn eval_string(&self, s: String) -> EResult {
        Ok(Str(s))
    }

    fn eval_ident(&mut self, i: String) -> EResult {
        Ok(self.get(&i).ok_or(EvalIdentError)?.clone())
    }

    fn eval_assign_expr(&mut self, name: Box<Expression>, value: Box<Expression>) -> EResult {
        if let Expression::Ident(name) = *name {
            let value = self.eval(*value)?;

            self.update(name, value)?;

            return Ok(NULL);
        } else if let Expression::IndexExpr { expr, index } = *name {
            if let Expression::Ident(name) = expr.borrow() {
                let expr = self.eval(*expr)?;
                let index = self.eval(*index)?;
            }
        }

        Err(EvalAssignExprError)
    }

    fn eval_array(&mut self, elems: Vec<Expression>) -> EResult {
        let mut elements = Vec::new();

        for e in elems {
            elements.push(self.eval(e)?);
        }

        Ok(Array(elements))
    }

    fn eval_index_expr(&mut self, expr: Box<Expression>, index: Box<Expression>) -> EResult {
        let expr = self.eval(*expr)?;
        let index = self.eval(*index)?;

        if let Array(elems) = expr {
            if let Integer(index) = index {
                if let Ok(index) = index.try_into() {
                    if elems.len() <= index {
                        return Err(EvalArrayIndexExprError);
                    }

                    return Ok(elems[index].clone());
                }

                let index_from_end: usize = (index.abs() - 1).try_into().unwrap();

                if elems.len() <= index_from_end {
                    return Err(EvalArrayIndexExprError);
                }

                let fixed_index = elems.len() - index_from_end - 1;

                return Ok(elems[fixed_index].clone());
            }
        } else if let Str(str) = expr {
            if let Integer(index) = index {
                if let Ok(index) = index.try_into() {
                    return str
                        .chars()
                        .nth(index)
                        .ok_or(EvalArrayIndexExprError)
                        .map(|o| Str(o.to_string()));
                }

                let index_from_end: usize = (index.abs() - 1).try_into().unwrap();

                if str.len() <= index_from_end {
                    return Err(EvalArrayIndexExprError);
                }

                let fixed_index = str.len() - index_from_end - 1;

                return str
                    .chars()
                    .nth(fixed_index)
                    .ok_or(EvalArrayIndexExprError)
                    .map(|o| Str(o.to_string()));
            }
        }

        Err(EvalArrayIndexExprError)
    }
}
