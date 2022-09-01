use std::fmt::Display;

// operation code.
#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Call(u8),
    Constant(u8),
    Negative,
    Add,
    Subtract,
    Multiply,
    Divide,
    Return,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Call(v) => write!(f, "system call {}", v),
            Self::Constant(v) => write!(f, "constant {}", v),
            Self::Negative => write!(f, "Negative operation"),
            Self::Add => write!(f, "Add operation"),
            Self::Subtract => write!(f, "Subtract operation"),
            Self::Multiply => write!(f, "Multiply operation"),
            Self::Divide => write!(f, "Divide operation"),
            Self::Return => write!(f, "system return"),
        }
    }
}
