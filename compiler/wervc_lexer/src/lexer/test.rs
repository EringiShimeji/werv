use crate::{
    lexer::Lexer,
    token::{
        Token,
        TokenKind::{self, *},
    },
};

fn loop_assert<const N: usize>(inputs: [impl ToString; N], expects: [Vec<(TokenKind, &str)>; N]) {
    for (input, expects) in inputs.into_iter().zip(expects) {
        let mut lexer = Lexer::new(input);

        for (kind, literal) in expects {
            assert_eq!(Token::new(kind, literal), lexer.next_token());
        }
    }
}

#[test]
fn lexer_number_test() {
    let inputs = ["0", "42", "1234567890;"];
    let expects = [
        vec![(Number, "0"), (EOF, "\0")],
        vec![(Number, "42"), (EOF, "\0")],
        vec![(Number, "1234567890"), (SemiColon, ";"), (EOF, "\0")],
    ];

    loop_assert(inputs, expects);
}

#[test]
fn lexer_arithmetic_test() {
    let inputs = ["1 + (2 - 3) * 4 / 5"];
    let expects = [vec![
        (Number, "1"),
        (Plus, "+"),
        (LParen, "("),
        (Number, "2"),
        (Minus, "-"),
        (Number, "3"),
        (RParen, ")"),
        (Asterisk, "*"),
        (Number, "4"),
        (Slash, "/"),
        (Number, "5"),
        (EOF, "\0"),
    ]];

    loop_assert(inputs, expects);
}
