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

    pub fn push_instuction(&mut self, value: OpCode) {
        self.instruction.push(value);
    }

    pub fn len(&self) -> usize {
        self.instruction.len()
    }

    // Chunk should have a name then we can disassemble?
    pub fn disassemble_chunk(&self, name: &str) {
        println!("== Begin to disassemble {} ==", name);

        for (offset, _chunk) in self.instruction.iter().enumerate() {
            self.disassemble_instruction(offset);
        }
    }

    #[allow(unreachable_patterns)]
    pub fn disassemble_instruction(&self, offset: usize) {
        println!("offset -> {}", offset);
        let instruction = &self.instruction[offset];
        match instruction {
            OpCode::Call(v) => println!("instruction -> call with {}", v),
            OpCode::Constant(v) => println!("constant -> {}", v),
            OpCode::Return => println!("instruction of return"),
            _ => println!("Unknown opcode {}", instruction),
        }
    }
}
