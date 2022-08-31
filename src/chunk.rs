use crate::lec::Lec;
use crate::op_code::OpCode;
use crate::value::Value;

pub struct Chunk {
    pub code: Lec<OpCode>,
    pub constants: Lec<Value>,
    pub lines: Lec<u8>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Lec::new(),
            constants: Lec::new(),
            lines: Lec::new(),
        }
    }

    pub fn write_to_chunk(&mut self, value: OpCode, line: u8) {
        self.push_instruction(value);
        self.push_line(line);
    }

    pub fn push_constant(&mut self, constant: Value) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn is_empty(&self) -> bool {
        self.code.len() == 0
    }

    //TODO - Refactor this with push line.
    pub fn push_instruction(&mut self, value: OpCode) {
        self.code.push(value);
    }

    //TODO - Refactor this
    pub fn push_line(&mut self, line: u8) {
        self.lines.push(line);
    }
    // FIXME - Chunk should have a name then we can disassemble?
    pub fn disassemble_chunk(&self, name: &str) {
        println!("== Begin to disassemble {} ==", name);

        for (offset, _) in self.code.iter().enumerate() {
            self.disassemble_instruction(offset);
        }
    }

    #[allow(unreachable_patterns)]
    pub fn disassemble_instruction(&self, offset: usize) {
        println!("offset -> {}", offset);
        let instruction = &self.code[offset];
        let line = &self.lines[offset];
        match instruction {
            OpCode::Call(v) => self.constant_instruction("OP_CALL", Some(v), offset, *line),
            OpCode::Constant(v) => self.constant_instruction("OP_CONSTANT", Some(v), offset, *line),
            OpCode::Return => self.constant_instruction("OP_RETURN", None, offset, *line),
            _ => println!("Unknown opcode {}", instruction),
        }
    }

    // FIXME - complete this function
    fn constant_instruction(&self, msg: &str, value: Option<&u8>, offset: usize, line: u8) {
        match value {
            Some(v) => {
                let constant = &self.constants[*v as usize];

                println!(
                    "OP CODE:{} - Line number {} - Constant pool index:{} and the value:{:?}",
                    msg, line, offset, constant
                );
            }

            None => println!("OP CODE:{} - Line number {}", msg, line),
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
