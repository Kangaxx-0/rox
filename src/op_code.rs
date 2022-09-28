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
    SetGlobal(usize),
    GetGlobal(usize),
    Greater,
    Less,
    Nil,
    Not,
    Multiply,
    Negative,
    Pop,
    Print,
    Return,
    Subtract,
    True,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "add operation"),
            Self::Call(v) => write!(f, "system call {}", v),
            Self::Constant(v) => write!(f, "constant {}", v),
            Self::Divide => write!(f, "divide operation"),
            Self::Equal => write!(f, "equal operation"),
            Self::False => write!(f, "false"),
            Self::GetGlobal(v) => write!(f, "get global variable {}", v),
            Self::SetGlobal(v) => write!(f, "define global from index {}", v),
            Self::Greater => write!(f, "greater operation"),
            Self::Less => write!(f, "less operation"),
            Self::Multiply => write!(f, "multiply operation"),
            Self::Negative => write!(f, "negative operation"),
            Self::Nil => write!(f, "nil"),
            Self::Not => write!(f, "not operation"),
            Self::Pop => write!(f, "pop operation"),
            Self::Print => write!(f, "print operation"),
            Self::Return => write!(f, "system return"),
            Self::Subtract => write!(f, "subtract operation"),
            Self::True => write!(f, "true"),
        }
    }
}
