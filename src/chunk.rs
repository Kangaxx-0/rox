use crate::lec::Lec;
use crate::op_code::OpCode;

pub struct Chunk {
    pub instruction: Lec<OpCode>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            instruction: Lec::new(),
        }
    }

    pub fn push_instruction(&mut self, value: OpCode) {
        self.instruction.push(value);
    }

    pub fn len(&self) -> usize {
        self.instruction.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instruction.len() == 0
    }

    // FIXME - Chunk should have a name then we can disassemble?
    pub fn disassemble_chunk(&self, name: &str) {
        println!("== Begin to disassemble {} ==", name);

        for (offset, _) in self.instruction.iter().enumerate() {
            self.disassemble_instruction(offset);
        }
    }

    #[allow(unreachable_patterns)]
    fn disassemble_instruction(&self, offset: usize) {
        println!("offset -> {}", offset);
        let instruction = &self.instruction[offset];
        match instruction {
            OpCode::Call(v) => self.simple_instruction("system call", Some(v)),
            OpCode::Constant(v) => self.simple_instruction("constant", Some(v)),
            OpCode::Return => self.simple_instruction("return code", None),
            _ => println!("Unknown opcode {}", instruction),
        }
    }

    // FIXME - complete this function
    fn simple_instruction(&self, msg: &str, value: Option<&u8>) {
        match value {
            Some(v) => println!("{} -> {}", msg, v),
            None => println!("{}", msg),
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
