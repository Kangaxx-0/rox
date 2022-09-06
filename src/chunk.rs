use crate::lec::Lec;
use crate::op_code::OpCode;
use crate::value::Value;

pub struct Chunk {
    pub code: Lec<OpCode>,
    pub constants: Lec<Value>,
    pub lines: Lec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Lec::new(),
            constants: Lec::new(),
            lines: Lec::new(),
        }
    }

    pub fn write_to_chunk(&mut self, value: OpCode, line: usize) {
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
    pub fn push_line(&mut self, line: usize) {
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
            OpCode::Call(v) => self.constant_instruction("Call", Some(v), offset, *line),
            OpCode::Constant(v) => self.constant_instruction("Constant", Some(v), offset, *line),
            OpCode::Negative => self.constant_instruction("Negative", None, offset, *line),
            OpCode::Return => self.constant_instruction("Return", None, offset, *line),
            OpCode::Add => self.constant_instruction("Add", None, offset, *line),
            OpCode::Subtract => self.constant_instruction("Subtract", None, offset, *line),
            OpCode::Multiply => self.constant_instruction("Multiply", None, offset, *line),
            OpCode::Divide => self.constant_instruction("Divide", None, offset, *line),
            _ => println!("Unknown opcode {}", instruction),
        }
    }

    // FIXME - complete this function
    fn constant_instruction(&self, msg: &str, value: Option<&usize>, offset: usize, line: usize) {
        match value {
            Some(v) => {
                let constant = &self.constants[*v];

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
