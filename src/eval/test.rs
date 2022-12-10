use crate::{
    eval::eval,
    lexer::Lexer,
    object::Object::{self, *},
    parser::Parser,
};

fn loop_test<T, const N: usize>(input: [T; N], expect: [Object; N])
where
    T: ToString,
{
    for i in 0..N {
        let l = Lexer::new(input[i].to_string());
        let mut p = Parser::new(l);
        let program = p.parse().unwrap();
        let object = eval(program).unwrap().unwrap();

        assert_eq!(object, expect[i].clone());
    }
}

#[test]
fn eval_integer_test() {
    let input = ["10;"];
    let expect = [Integer(10)];

    loop_test(input, expect);
}
