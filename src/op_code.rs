use std::fmt::Display;

// operation code.
#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Add,
    Call(usize),
    Constant(usize),
    Divide,
    Equal,
    False,
    Greater,
    Less,
    Nil,
    Not,
    Multiply,
    Negative,
    Print,
    Return,
    Subtract,
    True,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "Add operation"),
            Self::Call(v) => write!(f, "system call {}", v),
            Self::Constant(v) => write!(f, "constant {}", v),
            Self::Divide => write!(f, "Divide operation"),
            Self::Equal => write!(f, "Equal operation"),
            Self::False => write!(f, "false"),
            Self::Greater => write!(f, "Greater operation"),
            Self::Less => write!(f, "Less operation"),
            Self::Multiply => write!(f, "Multiply operation"),
            Self::Negative => write!(f, "Negative operation"),
            Self::Nil => write!(f, "nil"),
            Self::Not => write!(f, "Not operation"),
            Self::Print => write!(f, "Print operation"),
            Self::Return => write!(f, "system return"),
            Self::Subtract => write!(f, "Subtract operation"),
            Self::True => write!(f, "true"),
        }
    }
}
