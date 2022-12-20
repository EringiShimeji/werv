use crate::{
    environment::Environment,
    eval::eval,
    lexer::Lexer,
    object::Object::{self, *},
    parser::Parser,
};

use super::{error::EvalError::*, EResult};

fn loop_test<T, const N: usize>(input: [T; N], expect: [Object; N])
where
    T: ToString,
{
    for i in 0..N {
        let l = Lexer::new(input[i].to_string());
        let mut p = Parser::new(l);
        let program = p.parse().unwrap();
        let mut env = Environment::new();
        let object = eval(program, &mut env).unwrap();

        assert_eq!(object, expect[i].clone());
    }
}

fn loop_test_res<T, const N: usize>(input: [T; N], expect: [EResult; N])
where
    T: ToString,
{
    for i in 0..N {
        let l = Lexer::new(input[i].to_string());
        let mut p = Parser::new(l);
        let program = p.parse().unwrap();
        let mut env = Environment::new();
        let object = eval(program, &mut env);

        assert_eq!(object, expect[i]);
    }
}

#[test]
fn eval_integer_test() {
    let input = ["10;"];
    let expect = [Integer(10)];

    loop_test(input, expect);
}

#[test]
fn eval_binary_expr_test() {
    let input = [
        "1+2-3;", "3*4;", "8/2;", "100%11;", "20--10;", "1==1;", "1!=2;", "1<2;", "1<=2;", "2>1;",
        "2>=1;", "1==2;", "1!=1;", "1<1;", "1<=0;", "2>2;", "1>=2;",
    ];
    let expect = [
        Integer(0),
        Integer(12),
        Integer(4),
        Integer(1),
        Integer(30),
        Boolean(true),
        Boolean(true),
        Boolean(true),
        Boolean(true),
        Boolean(true),
        Boolean(true),
        Boolean(false),
        Boolean(false),
        Boolean(false),
        Boolean(false),
        Boolean(false),
        Boolean(false),
    ];

    loop_test(input, expect);
}

#[test]
fn eval_boolean_test() {
    let input = ["true;", "false;"];
    let expect = [Boolean(true), Boolean(false)];

    loop_test(input, expect);
}

#[test]
fn eval_unary_expr_test() {
    let input = ["!true;", "!false;", "-10;", "!(!!true);", "-(-10);"];
    let expect = [
        Boolean(false),
        Boolean(true),
        Integer(-10),
        Boolean(false),
        Integer(10),
    ];

    loop_test(input, expect);
}

#[test]
fn eval_if_expr_test() {
    let input = [
        "if true { 0 };",
        "if false { 0 } else if true { 1 };",
        "if false { 0 } else if false { 1 } else { 2 };",
    ];
    let expect = [Integer(0), Integer(1), Integer(2)];

    loop_test(input, expect);
}

#[test]
fn eval_let_stmt_test() {
    let input = ["let a = 10; let b = 20; b * a + 20;"];
    let expect = [Integer(220)];

    loop_test(input, expect);
}

#[test]
fn eval_scope_test() {
    let input = ["{let a=10;};a;", "let a = 10; {let b=10; a+b;};"];
    let expect = [Err(EvalIdentError), Ok(Integer(20))];

    loop_test_res(input, expect);
}
