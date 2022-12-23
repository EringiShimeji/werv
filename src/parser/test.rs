use super::Parser;
use crate::{
    ast::{BinaryExprKind::*, Expression::*, Node, Statement::*, UnaryExprKind::*},
    lexer::Lexer,
};

fn loop_test<T, U, P, const N: usize>(input: [T; N], expect: [U; N], f: impl Fn(&mut Parser) -> P)
where
    T: ToString,
    U: Clone + std::fmt::Debug,
    P: std::cmp::PartialEq<U> + std::fmt::Debug,
{
    for i in 0..N {
        let l = Lexer::new(input[i].to_string());
        let mut p = Parser::new(l);
        let a = f(&mut p);
        let b = expect[i].clone();

        assert_eq!(a, b, "{:#?}\n{:#?}\n{}", a, b, input[i].to_string());
    }
}

#[test]
fn parse_test() {
    let input = [r#"
let a = 1234567890;
let _ = 1 + (2 - 3) * 4 / 5;
let answer() = 42;
let calc(a, b, c, d, e) = (a + b - c) * d / e;
let fib(n) = add(fib(n-1), fib(n-2));
print(fib(10)+10);
let f(x, y) = {
    let nx = x + 1;
    let ny = y + 1;

    return nx + ny;
};
if a%2 { a } else { 0 }
if false { true } else { !false }
20 - -10
1==1
1!=1
1<1
1>1
1<=1
1>=1
"input123"
while true {}
while false {}
while 1 {}
while 0 {}
while 200 {}
while rand_bool() {}
while true println("hello")
"#];
    let expect = [Node::Program(vec![
        LetStmt {
            name: Ident("a".to_string()),
            value: Integer(1234567890),
        },
        LetStmt {
            name: Ident("_".to_string()),
            value: BinaryExpr {
                kind: Add,
                lhs: Box::new(Integer(1)),
                rhs: Box::new(BinaryExpr {
                    kind: Div,
                    lhs: Box::new(BinaryExpr {
                        kind: Mul,
                        lhs: Box::new(BinaryExpr {
                            kind: Sub,
                            lhs: Box::new(Integer(2)),
                            rhs: Box::new(Integer(3)),
                        }),
                        rhs: Box::new(Integer(4)),
                    }),
                    rhs: Box::new(Integer(5)),
                }),
            },
        },
        LetFnStmt {
            name: Ident("answer".into()),
            params: vec![],
            body: Integer(42),
        },
        LetFnStmt {
            name: Ident("calc".into()),
            params: vec![
                Ident("a".into()),
                Ident("b".into()),
                Ident("c".into()),
                Ident("d".into()),
                Ident("e".into()),
            ],
            body: BinaryExpr {
                kind: Div,
                lhs: Box::new(BinaryExpr {
                    kind: Mul,
                    lhs: Box::new(BinaryExpr {
                        kind: Sub,
                        lhs: Box::new(BinaryExpr {
                            kind: Add,
                            lhs: Box::new(Ident("a".into())),
                            rhs: Box::new(Ident("b".into())),
                        }),
                        rhs: Box::new(Ident("c".into())),
                    }),
                    rhs: Box::new(Ident("d".into())),
                }),
                rhs: Box::new(Ident("e".into())),
            },
        },
        LetFnStmt {
            name: Ident("fib".into()),
            params: vec![Ident("n".into())],
            body: CallExpr {
                name: Box::new(Ident("add".into())),
                args: vec![
                    CallExpr {
                        name: Box::new(Ident("fib".into())),
                        args: vec![BinaryExpr {
                            kind: Sub,
                            lhs: Box::new(Ident("n".into())),
                            rhs: Box::new(Integer(1)),
                        }],
                    },
                    CallExpr {
                        name: Box::new(Ident("fib".into())),
                        args: vec![BinaryExpr {
                            kind: Sub,
                            lhs: Box::new(Ident("n".into())),
                            rhs: Box::new(Integer(2)),
                        }],
                    },
                ],
            },
        },
        ExprStmt {
            is_null: true,
            expr: CallExpr {
                name: Box::new(Ident("print".into())),
                args: vec![BinaryExpr {
                    kind: Add,
                    lhs: Box::new(CallExpr {
                        name: Box::new(Ident("fib".into())),
                        args: vec![Integer(10)],
                    }),
                    rhs: Box::new(Integer(10)),
                }],
            },
        },
        LetFnStmt {
            name: Ident("f".into()),
            params: vec![Ident("x".into()), Ident("y".into())],
            body: BlockExpr(vec![
                LetStmt {
                    name: Ident("nx".into()),
                    value: BinaryExpr {
                        kind: Add,
                        lhs: Box::new(Ident("x".into())),
                        rhs: Box::new(Integer(1)),
                    },
                },
                LetStmt {
                    name: Ident("ny".into()),
                    value: BinaryExpr {
                        kind: Add,
                        lhs: Box::new(Ident("y".into())),
                        rhs: Box::new(Integer(1)),
                    },
                },
                ReturnStmt(BinaryExpr {
                    kind: Add,
                    lhs: Box::new(Ident("nx".into())),
                    rhs: Box::new(Ident("ny".into())),
                }),
            ]),
        },
        ExprStmt {
            is_null: false,
            expr: IfExpr {
                condition: Box::new(BinaryExpr {
                    kind: Mod,
                    lhs: Box::new(Ident("a".into())),
                    rhs: Box::new(Integer(2)),
                }),
                consequence: Box::new(BlockExpr(vec![ExprStmt {
                    is_null: false,
                    expr: Ident("a".into()),
                }])),
                alternative: Some(Box::new(BlockExpr(vec![ExprStmt {
                    is_null: false,
                    expr: Integer(0),
                }]))),
            },
        },
        ExprStmt {
            is_null: false,
            expr: IfExpr {
                condition: Box::new(Boolean(false)),
                consequence: Box::new(BlockExpr(vec![ExprStmt {
                    is_null: false,
                    expr: Boolean(true),
                }])),
                alternative: Some(Box::new(BlockExpr(vec![ExprStmt {
                    is_null: false,
                    expr: UnaryExpr {
                        kind: Not,
                        expr: Box::new(Boolean(false)),
                    },
                }]))),
            },
        },
        ExprStmt {
            is_null: false,
            expr: BinaryExpr {
                kind: Sub,
                lhs: Box::new(Integer(20)),
                rhs: Box::new(UnaryExpr {
                    kind: Minus,
                    expr: Box::new(Integer(10)),
                }),
            },
        },
        ExprStmt {
            is_null: false,
            expr: BinaryExpr {
                kind: Eq,
                lhs: Box::new(Integer(1)),
                rhs: Box::new(Integer(1)),
            },
        },
        ExprStmt {
            is_null: false,
            expr: BinaryExpr {
                kind: Ne,
                lhs: Box::new(Integer(1)),
                rhs: Box::new(Integer(1)),
            },
        },
        ExprStmt {
            is_null: false,
            expr: BinaryExpr {
                kind: Lt,
                lhs: Box::new(Integer(1)),
                rhs: Box::new(Integer(1)),
            },
        },
        ExprStmt {
            is_null: false,
            expr: BinaryExpr {
                kind: Gt,
                lhs: Box::new(Integer(1)),
                rhs: Box::new(Integer(1)),
            },
        },
        ExprStmt {
            is_null: false,
            expr: BinaryExpr {
                kind: Le,
                lhs: Box::new(Integer(1)),
                rhs: Box::new(Integer(1)),
            },
        },
        ExprStmt {
            is_null: false,
            expr: BinaryExpr {
                kind: Ge,
                lhs: Box::new(Integer(1)),
                rhs: Box::new(Integer(1)),
            },
        },
        ExprStmt {
            is_null: false,
            expr: Str("input123".into()),
        },
        WhileStmt {
            condition: Boolean(true),
            body: BlockExpr(vec![]),
        },
        WhileStmt {
            condition: Boolean(false),
            body: BlockExpr(vec![]),
        },
        WhileStmt {
            condition: Integer(1),
            body: BlockExpr(vec![]),
        },
        WhileStmt {
            condition: Integer(0),
            body: BlockExpr(vec![]),
        },
        WhileStmt {
            condition: Integer(200),
            body: BlockExpr(vec![]),
        },
        WhileStmt {
            condition: CallExpr {
                name: Box::new(Ident("rand_bool".into())),
                args: vec![],
            },
            body: BlockExpr(vec![]),
        },
        WhileStmt {
            condition: Boolean(true),
            body: CallExpr {
                name: Box::new(Ident("println".into())),
                args: vec![Str("hello".into())],
            },
        },
    ])];

    loop_test(input, expect, |p| p.parse().unwrap());
}

#[test]
fn parse_return_stmt_test() {
    let input = ["return nx + ny;"];
    let expect = [ReturnStmt(BinaryExpr {
        kind: Add,
        lhs: Box::new(Ident("nx".into())),
        rhs: Box::new(Ident("ny".into())),
    })];

    loop_test(input, expect, |p| p.parse_return_stmt().unwrap());
}

#[test]
fn parse_let_stmt_test() {
    let input = [
        "let a = 1234567890;",
        "let _ = 1 + (2 - 3) * 4 / 5;",
        "let nx = x + 1;",
        "let ny = y + 1;",
        "let answer() = 42;",
        "let calc(a, b, c, d, e) = (a + b - c) * d / e;",
        "let fib(n) = add(fib(n-1), fib(n-2));",
        "let f(x, y) = { let nx = x + 1; let ny = y + 1; return nx + ny; };",
    ];
    let expect = [
        LetStmt {
            name: Ident("a".to_string()),
            value: Integer(1234567890),
        },
        LetStmt {
            name: Ident("_".to_string()),
            value: BinaryExpr {
                kind: Add,
                lhs: Box::new(Integer(1)),
                rhs: Box::new(BinaryExpr {
                    kind: Div,
                    lhs: Box::new(BinaryExpr {
                        kind: Mul,
                        lhs: Box::new(BinaryExpr {
                            kind: Sub,
                            lhs: Box::new(Integer(2)),
                            rhs: Box::new(Integer(3)),
                        }),
                        rhs: Box::new(Integer(4)),
                    }),
                    rhs: Box::new(Integer(5)),
                }),
            },
        },
        LetStmt {
            name: Ident("nx".to_string()),
            value: BinaryExpr {
                kind: Add,
                lhs: Box::new(Ident("x".to_string())),
                rhs: Box::new(Integer(1)),
            },
        },
        LetStmt {
            name: Ident("ny".to_string()),
            value: BinaryExpr {
                kind: Add,
                lhs: Box::new(Ident("y".to_string())),
                rhs: Box::new(Integer(1)),
            },
        },
        LetFnStmt {
            name: Ident("answer".to_string()),
            params: vec![],
            body: Integer(42),
        },
        LetFnStmt {
            name: Ident("calc".to_string()),
            params: vec![
                Ident("a".to_string()),
                Ident("b".to_string()),
                Ident("c".to_string()),
                Ident("d".to_string()),
                Ident("e".to_string()),
            ],
            body: BinaryExpr {
                kind: Div,
                lhs: Box::new(BinaryExpr {
                    kind: Mul,
                    lhs: Box::new(BinaryExpr {
                        kind: Sub,
                        lhs: Box::new(BinaryExpr {
                            kind: Add,
                            lhs: Box::new(Ident("a".to_string())),
                            rhs: Box::new(Ident("b".to_string())),
                        }),
                        rhs: Box::new(Ident("c".to_string())),
                    }),
                    rhs: Box::new(Ident("d".to_string())),
                }),
                rhs: Box::new(Ident("e".to_string())),
            },
        },
        LetFnStmt {
            name: Ident("fib".to_string()),
            params: vec![Ident("n".to_string())],
            body: CallExpr {
                name: Box::new(Ident("add".to_string())),
                args: vec![
                    CallExpr {
                        name: Box::new(Ident("fib".to_string())),
                        args: vec![BinaryExpr {
                            kind: Sub,
                            lhs: Box::new(Ident("n".to_string())),
                            rhs: Box::new(Integer(1)),
                        }],
                    },
                    CallExpr {
                        name: Box::new(Ident("fib".to_string())),
                        args: vec![BinaryExpr {
                            kind: Sub,
                            lhs: Box::new(Ident("n".to_string())),
                            rhs: Box::new(Integer(2)),
                        }],
                    },
                ],
            },
        },
        LetFnStmt {
            name: Ident("f".to_string()),
            params: vec![Ident("x".to_string()), Ident("y".to_string())],
            body: BlockExpr(vec![
                LetStmt {
                    name: Ident("nx".to_string()),
                    value: BinaryExpr {
                        kind: Add,
                        lhs: Box::new(Ident("x".to_string())),
                        rhs: Box::new(Integer(1)),
                    },
                },
                LetStmt {
                    name: Ident("ny".to_string()),
                    value: BinaryExpr {
                        kind: Add,
                        lhs: Box::new(Ident("y".to_string())),
                        rhs: Box::new(Integer(1)),
                    },
                },
                ReturnStmt(BinaryExpr {
                    kind: Add,
                    lhs: Box::new(Ident("nx".to_string())),
                    rhs: Box::new(Ident("ny".to_string())),
                }),
            ]),
        },
    ];

    loop_test(input, expect, |p| p.parse_let_stmt().unwrap());
}

#[test]
fn parse_expression_test() {
    let input = [
        "1234567890",
        "1 + (2 - 3) * 4 / 5",
        "42",
        "(a + b - c) * d / e",
        "add(fib(n-1), fib(n-2))",
        "print(fib(10)+10)",
        "{ let nx = x + 1; let ny = y + 1; return nx + ny; }",
        "if a%2 { a } else { 0 }",
        "if false { true } else { false }",
    ];
    let expect = [
        Integer(1234567890),
        BinaryExpr {
            kind: Add,
            lhs: Box::new(Integer(1)),
            rhs: Box::new(BinaryExpr {
                kind: Div,
                lhs: Box::new(BinaryExpr {
                    kind: Mul,
                    lhs: Box::new(BinaryExpr {
                        kind: Sub,
                        lhs: Box::new(Integer(2)),
                        rhs: Box::new(Integer(3)),
                    }),
                    rhs: Box::new(Integer(4)),
                }),
                rhs: Box::new(Integer(5)),
            }),
        },
        Integer(42),
        BinaryExpr {
            kind: Div,
            lhs: Box::new(BinaryExpr {
                kind: Mul,
                lhs: Box::new(BinaryExpr {
                    kind: Sub,
                    lhs: Box::new(BinaryExpr {
                        kind: Add,
                        lhs: Box::new(Ident("a".into())),
                        rhs: Box::new(Ident("b".into())),
                    }),
                    rhs: Box::new(Ident("c".into())),
                }),
                rhs: Box::new(Ident("d".into())),
            }),
            rhs: Box::new(Ident("e".into())),
        },
        CallExpr {
            name: Box::new(Ident("add".into())),
            args: vec![
                CallExpr {
                    name: Box::new(Ident("fib".into())),
                    args: vec![BinaryExpr {
                        kind: Sub,
                        lhs: Box::new(Ident("n".into())),
                        rhs: Box::new(Integer(1)),
                    }],
                },
                CallExpr {
                    name: Box::new(Ident("fib".into())),
                    args: vec![BinaryExpr {
                        kind: Sub,
                        lhs: Box::new(Ident("n".into())),
                        rhs: Box::new(Integer(2)),
                    }],
                },
            ],
        },
        CallExpr {
            name: Box::new(Ident("print".into())),
            args: vec![BinaryExpr {
                kind: Add,
                lhs: Box::new(CallExpr {
                    name: Box::new(Ident("fib".into())),
                    args: vec![Integer(10)],
                }),
                rhs: Box::new(Integer(10)),
            }],
        },
        BlockExpr(vec![
            LetStmt {
                name: Ident("nx".into()),
                value: BinaryExpr {
                    kind: Add,
                    lhs: Box::new(Ident("x".into())),
                    rhs: Box::new(Integer(1)),
                },
            },
            LetStmt {
                name: Ident("ny".into()),
                value: BinaryExpr {
                    kind: Add,
                    lhs: Box::new(Ident("y".into())),
                    rhs: Box::new(Integer(1)),
                },
            },
            ReturnStmt(BinaryExpr {
                kind: Add,
                lhs: Box::new(Ident("nx".into())),
                rhs: Box::new(Ident("ny".into())),
            }),
        ]),
        IfExpr {
            condition: Box::new(BinaryExpr {
                kind: Mod,
                lhs: Box::new(Ident("a".into())),
                rhs: Box::new(Integer(2)),
            }),
            consequence: Box::new(BlockExpr(vec![ExprStmt {
                is_null: false,
                expr: Ident("a".into()),
            }])),
            alternative: Some(Box::new(BlockExpr(vec![ExprStmt {
                is_null: false,
                expr: Integer(0),
            }]))),
        },
        IfExpr {
            condition: Box::new(Boolean(false)),
            consequence: Box::new(BlockExpr(vec![ExprStmt {
                is_null: false,
                expr: Boolean(true),
            }])),
            alternative: Some(Box::new(BlockExpr(vec![ExprStmt {
                is_null: false,
                expr: Boolean(false),
            }]))),
        },
    ];

    loop_test(input, expect, |p| p.parse_expr().unwrap());
}

#[test]
fn parse_if_test() {
    let input = ["if a%2 { a } else { 0 };"];
    let expect = [IfExpr {
        condition: Box::new(BinaryExpr {
            kind: Mod,
            lhs: Box::new(Ident("a".into())),
            rhs: Box::new(Integer(2)),
        }),
        consequence: Box::new(BlockExpr(vec![ExprStmt {
            is_null: false,
            expr: Ident("a".into()),
        }])),
        alternative: Some(Box::new(BlockExpr(vec![ExprStmt {
            is_null: false,
            expr: Integer(0),
        }]))),
    }];

    loop_test(input, expect, |p| p.parse_if_expr().unwrap());
}

#[test]
fn parse_block_test() {
    let input = ["{ let nx = x + 1; let ny = y + 1; return nx + ny; }"];
    let expect = [BlockExpr(vec![
        LetStmt {
            name: Ident("nx".into()),
            value: BinaryExpr {
                kind: Add,
                lhs: Box::new(Ident("x".into())),
                rhs: Box::new(Integer(1)),
            },
        },
        LetStmt {
            name: Ident("ny".into()),
            value: BinaryExpr {
                kind: Add,
                lhs: Box::new(Ident("y".into())),
                rhs: Box::new(Integer(1)),
            },
        },
        ReturnStmt(BinaryExpr {
            kind: Add,
            lhs: Box::new(Ident("nx".into())),
            rhs: Box::new(Ident("ny".into())),
        }),
    ])];

    loop_test(input, expect, |p| p.parse_block_expr().unwrap());
}

#[test]
fn parse_bool_test() {
    let input = ["true", "false"];
    let expect = [Boolean(true), Boolean(false)];

    loop_test(input, expect, |p| p.parse_bool().unwrap());
}

#[test]
fn parse_assign_test() {
    let input = ["a=10", "vec[0]=true"];
    let expect = [
        AssignExpr {
            name: Box::new(Ident("a".into())),
            value: Box::new(Integer(10)),
        },
        AssignExpr {
            name: Box::new(IndexExpr {
                expr: Box::new(Ident("vec".into())),
                index: Box::new(Integer(0)),
            }),
            value: Box::new(Boolean(true)),
        },
    ];

    loop_test(input, expect, |p| p.parse_assign().unwrap());
}

#[test]
fn parse_add_test() {
    let input = [
        "1234567890",
        "1 + (2 - 3) * 4 / 5",
        "42",
        "(a + b - c) * d / e",
        "n-1",
        "n-2",
        "add(fib(n-1), fib(n-2))",
        "print(fib(10)+10)",
        "fib(10)+10",
        "x + 1",
        "y + 1",
        "nx + ny",
    ];
    let expect = [
        Integer(1234567890),
        BinaryExpr {
            kind: Add,
            lhs: Box::new(Integer(1)),
            rhs: Box::new(BinaryExpr {
                kind: Div,
                lhs: Box::new(BinaryExpr {
                    kind: Mul,
                    lhs: Box::new(BinaryExpr {
                        kind: Sub,
                        lhs: Box::new(Integer(2)),
                        rhs: Box::new(Integer(3)),
                    }),
                    rhs: Box::new(Integer(4)),
                }),
                rhs: Box::new(Integer(5)),
            }),
        },
        Integer(42),
        BinaryExpr {
            kind: Div,
            lhs: Box::new(BinaryExpr {
                kind: Mul,
                lhs: Box::new(BinaryExpr {
                    kind: Sub,
                    lhs: Box::new(BinaryExpr {
                        kind: Add,
                        lhs: Box::new(Ident("a".into())),
                        rhs: Box::new(Ident("b".into())),
                    }),
                    rhs: Box::new(Ident("c".into())),
                }),
                rhs: Box::new(Ident("d".into())),
            }),
            rhs: Box::new(Ident("e".into())),
        },
        BinaryExpr {
            kind: Sub,
            lhs: Box::new(Ident("n".into())),
            rhs: Box::new(Integer(1)),
        },
        BinaryExpr {
            kind: Sub,
            lhs: Box::new(Ident("n".into())),
            rhs: Box::new(Integer(2)),
        },
        CallExpr {
            name: Box::new(Ident("add".into())),
            args: vec![
                CallExpr {
                    name: Box::new(Ident("fib".into())),
                    args: vec![BinaryExpr {
                        kind: Sub,
                        lhs: Box::new(Ident("n".into())),
                        rhs: Box::new(Integer(1)),
                    }],
                },
                CallExpr {
                    name: Box::new(Ident("fib".into())),
                    args: vec![BinaryExpr {
                        kind: Sub,
                        lhs: Box::new(Ident("n".into())),
                        rhs: Box::new(Integer(2)),
                    }],
                },
            ],
        },
        CallExpr {
            name: Box::new(Ident("print".into())),
            args: vec![BinaryExpr {
                kind: Add,
                lhs: Box::new(CallExpr {
                    name: Box::new(Ident("fib".into())),
                    args: vec![Integer(10)],
                }),
                rhs: Box::new(Integer(10)),
            }],
        },
        BinaryExpr {
            kind: Add,
            lhs: Box::new(CallExpr {
                name: Box::new(Ident("fib".into())),
                args: vec![Integer(10)],
            }),
            rhs: Box::new(Integer(10)),
        },
        BinaryExpr {
            kind: Add,
            lhs: Box::new(Ident("x".into())),
            rhs: Box::new(Integer(1)),
        },
        BinaryExpr {
            kind: Add,
            lhs: Box::new(Ident("y".into())),
            rhs: Box::new(Integer(1)),
        },
        BinaryExpr {
            kind: Add,
            lhs: Box::new(Ident("nx".into())),
            rhs: Box::new(Ident("ny".into())),
        },
    ];

    loop_test(input, expect, |p| p.parse_add().unwrap());
}

#[test]
fn parse_mul_test() {
    let input = [
        "1234567890",
        "(2 - 3) * 4 / 5",
        "42",
        "(a + b - c) * d / e",
        "print(fib(10)+10)",
        "fib(10)",
    ];
    let expect = [
        Integer(1234567890),
        BinaryExpr {
            kind: Div,
            lhs: Box::new(BinaryExpr {
                kind: Mul,
                lhs: Box::new(BinaryExpr {
                    kind: Sub,
                    lhs: Box::new(Integer(2)),
                    rhs: Box::new(Integer(3)),
                }),
                rhs: Box::new(Integer(4)),
            }),
            rhs: Box::new(Integer(5)),
        },
        Integer(42),
        BinaryExpr {
            kind: Div,
            lhs: Box::new(BinaryExpr {
                kind: Mul,
                lhs: Box::new(BinaryExpr {
                    kind: Sub,
                    lhs: Box::new(BinaryExpr {
                        kind: Add,
                        lhs: Box::new(Ident("a".into())),
                        rhs: Box::new(Ident("b".into())),
                    }),
                    rhs: Box::new(Ident("c".into())),
                }),
                rhs: Box::new(Ident("d".into())),
            }),
            rhs: Box::new(Ident("e".into())),
        },
        CallExpr {
            name: Box::new(Ident("print".into())),
            args: vec![BinaryExpr {
                kind: Add,
                lhs: Box::new(CallExpr {
                    name: Box::new(Ident("fib".into())),
                    args: vec![Integer(10)],
                }),
                rhs: Box::new(Integer(10)),
            }],
        },
        CallExpr {
            name: Box::new(Ident("fib".into())),
            args: vec![Integer(10)],
        },
    ];

    loop_test(input, expect, |p| p.parse_mul().unwrap());
}

#[test]

fn parse_unary_test() {
    let input = ["-20", "!true", "!false"];
    let expect = [
        UnaryExpr {
            kind: Minus,
            expr: Box::new(Integer(20)),
        },
        UnaryExpr {
            kind: Not,
            expr: Box::new(Boolean(true)),
        },
        UnaryExpr {
            kind: Not,
            expr: Box::new(Boolean(false)),
        },
    ];

    loop_test(input, expect, |p| p.parse_unary().unwrap());
}

#[test]
fn parse_index_test() {
    let input = ["[1,2,3][-1]", "[][zero()]", r#""hello"[0]"#, "vec[0][0]"];
    let expect = [
        IndexExpr {
            expr: Box::new(Array(vec![Integer(1), Integer(2), Integer(3)])),
            index: Box::new(UnaryExpr {
                kind: Minus,
                expr: Box::new(Integer(1)),
            }),
        },
        IndexExpr {
            expr: Box::new(Array(vec![])),
            index: Box::new(CallExpr {
                name: Box::new(Ident("zero".into())),
                args: vec![],
            }),
        },
        IndexExpr {
            expr: Box::new(Str("hello".into())),
            index: Box::new(Integer(0)),
        },
        IndexExpr {
            expr: Box::new(IndexExpr {
                expr: Box::new(Ident("vec".into())),
                index: Box::new(Integer(0)),
            }),
            index: Box::new(Integer(0)),
        },
    ];

    loop_test(input, expect, |p| p.parse_index().unwrap());
}

#[test]
fn parse_primary_test() {
    let input = [
        "1234567890",
        "1",
        "(2 - 3)",
        "4",
        "5",
        "42",
        "(a + b - c)",
        "d",
        "e",
        "(n-1)",
        "add(fib(n-1), fib(n-2))",
        "print(fib(10)+10)",
        "x",
        "y",
        "nx",
        "ny",
    ];
    let expect = [
        Integer(1234567890),
        Integer(1),
        BinaryExpr {
            kind: Sub,
            lhs: Box::new(Integer(2)),
            rhs: Box::new(Integer(3)),
        },
        Integer(4),
        Integer(5),
        Integer(42),
        BinaryExpr {
            kind: Sub,
            lhs: Box::new(BinaryExpr {
                kind: Add,
                lhs: Box::new(Ident("a".into())),
                rhs: Box::new(Ident("b".into())),
            }),
            rhs: Box::new(Ident("c".into())),
        },
        Ident("d".into()),
        Ident("e".into()),
        BinaryExpr {
            kind: Sub,
            lhs: Box::new(Ident("n".into())),
            rhs: Box::new(Integer(1)),
        },
        CallExpr {
            name: Box::new(Ident("add".into())),
            args: vec![
                CallExpr {
                    name: Box::new(Ident("fib".into())),
                    args: vec![BinaryExpr {
                        kind: Sub,
                        lhs: Box::new(Ident("n".into())),
                        rhs: Box::new(Integer(1)),
                    }],
                },
                CallExpr {
                    name: Box::new(Ident("fib".into())),
                    args: vec![BinaryExpr {
                        kind: Sub,
                        lhs: Box::new(Ident("n".into())),
                        rhs: Box::new(Integer(2)),
                    }],
                },
            ],
        },
        CallExpr {
            name: Box::new(Ident("print".into())),
            args: vec![BinaryExpr {
                kind: Add,
                lhs: Box::new(CallExpr {
                    name: Box::new(Ident("fib".into())),
                    args: vec![Integer(10)],
                }),
                rhs: Box::new(Integer(10)),
            }],
        },
        Ident("x".into()),
        Ident("y".into()),
        Ident("nx".into()),
        Ident("ny".into()),
    ];

    loop_test(input, expect, |p| {
        eprintln!("{:?}", p);
        p.parse_primary().unwrap()
    });
}

#[test]
fn parse_integer_test() {
    let input = ["1234567890", "1", "2", "3", "4", "5", "42", "10"];
    let expect = [
        Integer(1234567890),
        Integer(1),
        Integer(2),
        Integer(3),
        Integer(4),
        Integer(5),
        Integer(42),
        Integer(10),
    ];

    loop_test(input, expect, |p| p.parse_integer().unwrap());
}

#[test]
fn parse_ident_test() {
    let input = [
        "a", "_", "answer", "calc", "b", "c", "d", "e", "add", "fib", "n", "print", "f", "x", "y",
        "nx", "ny",
    ];
    let expect = [
        Ident("a".into()),
        Ident("_".into()),
        Ident("answer".into()),
        Ident("calc".into()),
        Ident("b".into()),
        Ident("c".into()),
        Ident("d".into()),
        Ident("e".into()),
        Ident("add".into()),
        Ident("fib".into()),
        Ident("n".into()),
        Ident("print".into()),
        Ident("f".into()),
        Ident("x".into()),
        Ident("y".into()),
        Ident("nx".into()),
        Ident("ny".into()),
    ];

    loop_test(input, expect, |p| p.parse_ident().unwrap());
}

#[test]
fn parse_array_test() {
    let input = [
        "[1,2,3]",
        r#"["[","]","a"]"#,
        "[true, false]",
        "[one(),two()]",
    ];
    let expect = [
        Array(vec![Integer(1), Integer(2), Integer(3)]),
        Array(vec![
            Str("[".to_string()),
            Str("]".to_string()),
            Str("a".to_string()),
        ]),
        Array(vec![Boolean(true), Boolean(false)]),
        Array(vec![
            CallExpr {
                name: Box::new(Ident("one".into())),
                args: vec![],
            },
            CallExpr {
                name: Box::new(Ident("two".into())),
                args: vec![],
            },
        ]),
    ];

    loop_test(input, expect, |p| p.parse_array().unwrap());
}
