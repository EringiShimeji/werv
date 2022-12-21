use super::Lexer;
use crate::token::{Token, TokenKind::*};

#[test]
fn next_token_test() {
    let input = r#"
let _aA10 = 1 + 2 - 3 * 4 / 5; 
let fib(n) = fib(n-1) + fib(n-2);
let add(a, b) = {
    return a + b;
};
// this is a test comment
/*
this is a 

comment !!!
!!!
 */

 // bla
let only_odd(n) = if n % 2 n else 0;
if 1==1 { 
    // block inner comment
    !false 
} else if true { !true };
1!=10;
1<10;
1<=10;
1>10;
1>=10;
"input123";
"#;
    let expect = [
        (Let, "let"),
        (Ident, "_aA10"),
        (Assign, "="),
        (Number, "1"),
        (Plus, "+"),
        (Number, "2"),
        (Minus, "-"),
        (Number, "3"),
        (Asterisk, "*"),
        (Number, "4"),
        (Slash, "/"),
        (Number, "5"),
        (SemiColon, ";"),
        (Let, "let"),
        (Ident, "fib"),
        (LParen, "("),
        (Ident, "n"),
        (RParen, ")"),
        (Assign, "="),
        (Ident, "fib"),
        (LParen, "("),
        (Ident, "n"),
        (Minus, "-"),
        (Number, "1"),
        (RParen, ")"),
        (Plus, "+"),
        (Ident, "fib"),
        (LParen, "("),
        (Ident, "n"),
        (Minus, "-"),
        (Number, "2"),
        (RParen, ")"),
        (SemiColon, ";"),
        (Let, "let"),
        (Ident, "add"),
        (LParen, "("),
        (Ident, "a"),
        (Comma, ","),
        (Ident, "b"),
        (RParen, ")"),
        (Assign, "="),
        (LBrace, "{"),
        (Return, "return"),
        (Ident, "a"),
        (Plus, "+"),
        (Ident, "b"),
        (SemiColon, ";"),
        (RBrace, "}"),
        (SemiColon, ";"),
        (Let, "let"),
        (Ident, "only_odd"),
        (LParen, "("),
        (Ident, "n"),
        (RParen, ")"),
        (Assign, "="),
        (If, "if"),
        (Ident, "n"),
        (Percent, "%"),
        (Number, "2"),
        (Ident, "n"),
        (Else, "else"),
        (Number, "0"),
        (SemiColon, ";"),
        (If, "if"),
        (Number, "1"),
        (Eq, "=="),
        (Number, "1"),
        (LBrace, "{"),
        (Bang, "!"),
        (False, "false"),
        (RBrace, "}"),
        (Else, "else"),
        (If, "if"),
        (True, "true"),
        (LBrace, "{"),
        (Bang, "!"),
        (True, "true"),
        (RBrace, "}"),
        (SemiColon, ";"),
        (Number, "1"),
        (Ne, "!="),
        (Number, "10"),
        (SemiColon, ";"),
        (Number, "1"),
        (Lt, "<"),
        (Number, "10"),
        (SemiColon, ";"),
        (Number, "1"),
        (Le, "<="),
        (Number, "10"),
        (SemiColon, ";"),
        (Number, "1"),
        (Gt, ">"),
        (Number, "10"),
        (SemiColon, ";"),
        (Number, "1"),
        (Ge, ">="),
        (Number, "10"),
        (SemiColon, ";"),
        (Str, "input123"),
        (SemiColon, ";"),
        (EOF, "\0"),
    ];
    let mut l = Lexer::new(input.to_string());

    for (kind, literal) in expect {
        assert_eq!(
            l.next_token().unwrap(),
            Token::new(kind, literal.to_string())
        );
    }

    assert_eq!(l.next_token().unwrap(), Token::new(EOF, "\0".to_string()));
}
