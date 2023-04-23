pub mod error;

use std::fmt::Display;

use error::CompileError;
use wervc_ast::{
    BinaryExpr, BinaryExprKind, BlockExpr, CallExpr, Expression, Integer, Node, Program,
    ReturnExpr, Statement, UnaryExpr,
};
use wervc_parser::parser::Parser;

type CResult = Result<(), CompileError>;

const X86_64_ARG_REGISTERS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

pub struct Compiler {
    pub output: String,
    pub label_count: usize,
    // push/pop によって rsp が変化するので、その変化量を記録しておく
    // これは、関数の prologue/epilogue で rsp を調整するために必要
    pub depth: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            label_count: 0,
            depth: 0,
        }
    }

    fn add_code(&mut self, code: impl ToString) {
        self.output.push_str(code.to_string().as_str());
        self.output.push('\n');
    }

    fn get_serial_label(&mut self, label: impl Display) -> String {
        let label = format!(".L{}{:>03}", label, self.label_count);

        self.label_count += 1;

        label
    }

    fn get_if_end_label(&mut self) -> String {
        self.get_serial_label("end")
    }

    fn get_if_else_label(&mut self) -> String {
        self.get_serial_label("else")
    }

    fn gen_label(&mut self, label: impl Display) {
        self.add_code(format!("{}:", label));
    }

    fn nullary(&mut self, operation: impl Display) {
        self.add_code(format!("  {}", operation));
    }

    fn unary_op(&mut self, operation: impl Display, operand: impl Display) {
        self.add_code(format!("  {} {}", operation, operand));
    }

    fn binary_op(&mut self, operation: impl Display, lhs: impl Display, rhs: impl Display) {
        self.add_code(format!("  {} {}, {}", operation, lhs, rhs));
    }

    fn mov(&mut self, lhs: impl Display, rhs: impl Display) {
        self.binary_op("mov", lhs, rhs);
    }

    fn call(&mut self, name: impl Display) {
        self.unary_op("call", name);
    }

    fn ret(&mut self) {
        self.nullary("ret");
    }

    fn push(&mut self, from: impl Display) {
        self.unary_op("push", from);
        self.depth += 1;
    }

    fn pop(&mut self, to: impl Display) {
        self.unary_op("pop", to);
        self.depth -= 1;
    }

    fn add(&mut self, lhs: impl Display, rhs: impl Display) {
        self.binary_op("add", lhs, rhs);
    }

    fn sub(&mut self, lhs: impl Display, rhs: impl Display) {
        self.binary_op("sub", lhs, rhs);
    }

    fn imul(&mut self, lhs: impl Display, rhs: impl Display) {
        self.binary_op("imul", lhs, rhs);
    }

    fn idiv(&mut self, value: impl Display) {
        self.nullary("cqo");
        self.unary_op("idiv", value);
    }

    fn neg(&mut self, value: impl Display) {
        self.unary_op("neg", value);
    }

    fn cmp(&mut self, lhs: impl Display, rhs: impl Display) {
        self.binary_op("cmp", lhs, rhs);
    }

    fn movzb(&mut self, lhs: impl Display, rhs: impl Display) {
        self.add_code(format!("  movzb {}, {}", lhs, rhs));
    }

    fn je(&mut self, label: impl Display) {
        self.unary_op("je", label);
    }

    fn jmp(&mut self, label: impl Display) {
        self.unary_op("jmp", label);
    }

    pub fn compile(&mut self, program: impl ToString) -> CResult {
        let program = Parser::new(program)
            .parse_program()
            .map_err(CompileError::ParserError)?;
        let program = match program {
            Node::Program(p) => p,
            _ => {
                return Err(CompileError::InputIsNotProgram);
            }
        };

        self.gen_program(&program)?;

        Ok(())
    }

    fn gen_prelude(&mut self) {
        self.add_code(".intel_syntax noprefix");
        self.add_code(".globl main");
        self.add_code("main:");
    }

    fn gen_program(&mut self, program: &Program) -> CResult {
        let Program {
            statements,
            total_offset,
        } = program;

        self.gen_prelude();
        self.gen_prologue(*total_offset);

        self.gen_statements(statements)?;

        self.mov("rsp", "rbp");
        self.pop("rbp");
        self.ret();

        Ok(())
    }

    fn gen_statements(&mut self, statements: &Vec<Statement>) -> CResult {
        for statement in statements {
            self.gen_statement(statement)?;
            self.pop("rax");
        }

        Ok(())
    }

    fn gen_statement(&mut self, statement: &Statement) -> CResult {
        match statement {
            Statement::ExprStmt(e) => {
                self.gen_expr(e)?;
                self.pop("rax");
                self.mov("rax", 0);
                self.push("rax");
            }
            Statement::ExprReturnStmt(e) => {
                self.gen_expr(e)?;
            }
        }

        Ok(())
    }

    fn gen_expr(&mut self, e: &Expression) -> CResult {
        match e {
            Expression::Integer(e) => self.gen_integer(e),
            Expression::BinaryExpr(e) => self.gen_binary_expr(e),
            Expression::UnaryExpr(e) => self.gen_unary_expr(e),
            Expression::Ident(_) => self.gen_ident(e),
            Expression::ReturnExpr(e) => self.gen_return_expr(e),
            Expression::IfExpr(e) => self.gen_if_expr(e),
            Expression::BlockExpr(e) => self.gen_block_expr(e),
            Expression::CallExpr(e) => self.gen_call_expr(e),
            _ => Err(CompileError::Unimplemented),
        }
    }

    fn gen_integer(&mut self, e: &Integer) -> CResult {
        self.push(e.value);

        Ok(())
    }

    fn gen_binary_expr(&mut self, e: &BinaryExpr) -> CResult {
        self.gen_expr(&e.lhs)?;
        self.gen_expr(&e.rhs)?;

        self.pop("rdi");
        self.pop("rax");

        match e.kind {
            BinaryExprKind::Add => {
                self.add("rax", "rdi");
            }
            BinaryExprKind::Sub => {
                self.sub("rax", "rdi");
            }
            BinaryExprKind::Mul => {
                self.imul("rax", "rdi");
            }
            BinaryExprKind::Div => {
                self.idiv("rdi");
            }
            BinaryExprKind::Eq => {
                self.cmp("rax", "rdi");
                self.unary_op("sete", "al");
                self.movzb("rax", "al");
            }
            BinaryExprKind::Ne => {
                self.cmp("rax", "rdi");
                self.unary_op("setne", "al");
                self.movzb("rax", "al");
            }
            BinaryExprKind::Ge => {
                self.cmp("rax", "rdi");
                self.unary_op("setge", "al");
                self.movzb("rax", "al");
            }
            BinaryExprKind::Gt => {
                self.cmp("rax", "rdi");
                self.unary_op("setg", "al");
                self.movzb("rax", "al");
            }
            BinaryExprKind::Le => {
                self.cmp("rax", "rdi");
                self.unary_op("setle", "al");
                self.movzb("rax", "al");
            }
            BinaryExprKind::Lt => {
                self.cmp("rax", "rdi");
                self.unary_op("setl", "al");
                self.movzb("rax", "al");
            }
            BinaryExprKind::Assign => {
                self.gen_left_val(&e.lhs)?;
                self.gen_expr(&e.rhs)?;

                self.pop("rdi");
                self.pop("rax");
                self.mov("[rax]", "rdi");
                self.push("rdi");
            }
        }

        self.push("rax");

        Ok(())
    }

    fn gen_unary_expr(&mut self, e: &UnaryExpr) -> CResult {
        self.gen_expr(&e.expr)?;

        self.pop("rax");

        match e.kind {
            wervc_ast::UnaryExprKind::Minus => {
                self.neg("rax");
            }
            _ => {
                return Err(CompileError::Unimplemented);
            }
        }

        self.push("rax");

        Ok(())
    }

    fn gen_left_val(&mut self, e: &Expression) -> CResult {
        match e {
            Expression::Ident(e) => {
                self.mov("rax", "rbp");
                self.sub("rax", e.offset);
                self.push("rax");
            }
            _ => {
                return Err(CompileError::NotLeftValue);
            }
        }

        Ok(())
    }

    fn gen_ident(&mut self, e: &Expression) -> CResult {
        self.gen_left_val(e)?;
        self.pop("rax");
        self.mov("rax", "[rax]");
        self.push("rax");

        Ok(())
    }

    fn gen_prologue(&mut self, total_offset: isize) {
        self.push("rbp");
        self.mov("rbp", "rsp");
        self.sub("rsp", total_offset);
    }

    fn gen_return_expr(&mut self, e: &ReturnExpr) -> CResult {
        self.gen_expr(&e.value)?;
        self.pop("rax");
        self.mov("rsp", "rbp");
        self.pop("rbp");
        self.ret();

        Ok(())
    }

    fn gen_if_expr(&mut self, e: &wervc_ast::IfExpr) -> CResult {
        self.gen_expr(&e.condition)?;
        self.pop("rax");
        self.cmp("rax", 0);

        if let Some(alternative) = &e.alternative {
            let else_label = self.get_if_else_label();
            let end_label = self.get_if_end_label();

            self.je(&else_label);
            self.gen_expr(&e.consequence)?;

            self.jmp(&end_label);
            self.gen_label(else_label);
            self.gen_expr(alternative)?;
            self.gen_label(end_label);
        } else {
            let end_label = self.get_if_end_label();

            self.je(&end_label);
            self.gen_expr(&e.consequence)?;
            self.gen_label(end_label);
        }

        Ok(())
    }

    fn gen_block_expr(&mut self, e: &BlockExpr) -> CResult {
        self.gen_statements(&e.statements)?;
        self.push("rax");

        Ok(())
    }

    fn gen_call_expr(&mut self, e: &CallExpr) -> CResult {
        match &*e.func {
            Expression::Ident(func_name) => {
                for (arg, register) in e.args.iter().zip(X86_64_ARG_REGISTERS.iter()) {
                    self.gen_expr(arg)?;
                    self.pop(register);
                }

                self.mov("rax", 0);

                // rspを16バイト境界に揃える
                if self.depth % 2 == 0 {
                    self.call(&func_name.name);
                } else {
                    self.sub("rsp", 8);
                    self.call(&func_name.name);
                    self.add("rsp", 8);
                }

                self.push("rax");
            }
            _ => {
                return Err(CompileError::Unimplemented);
            }
        }

        Ok(())
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}
