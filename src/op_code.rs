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
    DefineGlobal(usize),
    DefineLocal,
    SetGlobal(usize),
    GetGlobal(usize),
    SetLocal(usize),
    GetLocal(usize),
    Greater,
    Less,
    Loop(u16),
    Jump(u16),
    JumpIfFalse(u16),
    Nil,
    Not,
    Multiply,
    Negative,
    Placeholder,
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
            Self::DefineGlobal(v) => write!(f, "define global from index {}", v),
            Self::GetLocal(v) => write!(f, "define local variable in stack from index {}", v),
            Self::SetLocal(v) => write!(f, "set local variable in stack from index {}", v),
            Self::DefineLocal => write!(f, "define local variable"),
            Self::GetGlobal(v) => write!(f, "get global variable from index {}", v),
            Self::SetGlobal(v) => write!(f, "set global variable from index {}", v),
            Self::Greater => write!(f, "greater operation"),
            Self::Less => write!(f, "less operation"),
            Self::Loop(v) => write!(f, "loop to offset {}", v),
            Self::Jump(v) => write!(f, "jump to {}", v),
            Self::JumpIfFalse(v) => write!(f, "jump to offset {}", v),
            Self::Multiply => write!(f, "multiply operation"),
            Self::Negative => write!(f, "negative operation"),
            Self::Nil => write!(f, "nil"),
            Self::Not => write!(f, "not operation"),
            Self::Placeholder => write!(f, "placeholder"),
            Self::Pop => write!(f, "pop operation"),
            Self::Print => write!(f, "print operation"),
            Self::Return => write!(f, "system return"),
            Self::Subtract => write!(f, "subtract operation"),
            Self::True => write!(f, "true"),
        }
    }
}
