use std::fmt::Display;

// operation code.
#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Add,
    Call(usize),
    Constant(usize),
    Divide,
    False,
    Nil,
    Not,
    Multiply,
    Negative,
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
            Self::False => write!(f, "false"),
            Self::Multiply => write!(f, "Multiply operation"),
            Self::Negative => write!(f, "Negative operation"),
            Self::Nil => write!(f, "nil"),
            Self::Not => write!(f, "Not operation"),
            Self::Return => write!(f, "system return"),
            Self::Subtract => write!(f, "Subtract operation"),
            Self::True => write!(f, "true"),
        }
    }
}
