#[derive(Debug, PartialEq, Eq)]
pub enum Object {
    Integer(isize),
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Integer(i) => i.to_string(),
            }
        )
    }
}