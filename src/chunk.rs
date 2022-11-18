use crate::op_code::OpCode;
use crate::value::Value;

use gc_derive::{Finalize, Trace};
#[derive(PartialEq, Eq, PartialOrd, Debug, Clone, Trace, Finalize)]
pub struct Chunk {
    #[unsafe_ignore_trace]
    pub code: Vec<OpCode>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            // Instruction OP code
            code: Vec::new(),
            //TODO: use hash table to store constants?
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn write_to_chunk(&mut self, value: OpCode, line: usize) {
        self.push_instruction(value);
        self.push_line(line);
    }

    pub fn push_constant(&mut self, constant: Value) -> usize {
        self.constants.push(constant);
        // return the index of the constant
        self.constants.len() - 1
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn is_empty(&self) -> bool {
        self.code.len() == 0
    }

    pub fn push_instruction(&mut self, value: OpCode) {
        self.code.push(value);
    }

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
            OpCode::Call(v) => self.constant_instruction("Call", Some(*v), offset, *line),
            OpCode::Closure(v) => self.constant_instruction("Closure", Some(*v), offset, *line),
            OpCode::CloseUpvalue => self.constant_instruction("CloseUpValue", None, offset, *line),
            OpCode::Constant(v) => self.constant_instruction("Constant", Some(*v), offset, *line),
            OpCode::Negative => self.constant_instruction("Negative", None, offset, *line),
            OpCode::Return => self.constant_instruction("Return", None, offset, *line),
            OpCode::Add => self.constant_instruction("Add", None, offset, *line),
            OpCode::Subtract => self.constant_instruction("Subtract", None, offset, *line),
            OpCode::Multiply => self.constant_instruction("Multiply", None, offset, *line),
            OpCode::Divide => self.constant_instruction("Divide", None, offset, *line),
            OpCode::Nil => self.constant_instruction("Nil", None, offset, *line),
            OpCode::True => self.constant_instruction("True", None, offset, *line),
            OpCode::False => self.constant_instruction("False", None, offset, *line),
            OpCode::Not => self.constant_instruction("Not", None, offset, *line),
            OpCode::Equal => self.constant_instruction("Equal", None, offset, *line),
            OpCode::Greater => self.constant_instruction("Greater", None, offset, *line),
            OpCode::Less => self.constant_instruction("Less", None, offset, *line),
            OpCode::Print => self.constant_instruction("Print", None, offset, *line),
            OpCode::Pop => self.constant_instruction("Pop", None, offset, *line),
            OpCode::SetGlobal(v) => {
                self.constant_instruction("Set Global", Some(*v), offset, *line)
            }
            OpCode::GetGlobal(v) => {
                self.constant_instruction("Get Global", Some(*v), offset, *line)
            }
            OpCode::DefineGlobal(v) => {
                self.constant_instruction("Define Global", Some(*v), offset, *line)
            }
            OpCode::GetLocal(v) => self.constant_instruction("Get Local", Some(*v), offset, *line),
            OpCode::GetUpvalue(v) => {
                self.constant_instruction("Get Upvalue", Some(*v), offset, *line)
            }
            OpCode::SetLocal(v) => self.constant_instruction("Set Local", Some(*v), offset, *line),
            OpCode::SetUpvalue(v) => {
                self.constant_instruction("Set Upvalue", Some(*v), offset, *line)
            }
            OpCode::Call(v) => self.constant_instruction("Function", Some(*v), offset, *line),
            OpCode::GetUpvalue(v) => {
                self.constant_instruction("Get Upvalue", Some(*v), offset, *line)
            }
            OpCode::SetUpvalue(v) => {
                self.constant_instruction("Set Upvalue", Some(*v), offset, *line)
            }
            _ => println!("Unknown opcode {}", instruction),
        }
    }

    // FIXME - complete this function
    fn constant_instruction(&self, msg: &str, value: Option<usize>, offset: usize, line: usize) {
        match value {
            Some(v) => {
                let constant = &self.constants[v];

                println!(
                    "OP CODE:{} - Line number {} - Constant pool index:{} and the value:{}",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_chunk() {
        let chunk = Chunk::new();
        assert_eq!(chunk.code.len(), 0);
        assert_eq!(chunk.constants.len(), 0);
        assert_eq!(chunk.lines.len(), 0);
    }

    #[test]
    fn test_write_to_chunk() {
        let mut chunk = Chunk::new();
        chunk.write_to_chunk(OpCode::Constant(1), 1);
        assert_eq!(chunk.code.len(), 1);
        assert_eq!(chunk.constants.len(), 0);
        assert_eq!(chunk.lines.len(), 1);
    }

    #[test]
    fn test_push_constant() {
        let mut chunk = Chunk::new();
        let constant = Value::Number(1.0);
        let index = chunk.push_constant(constant);
        assert_eq!(chunk.constants.len(), 1);
        assert_eq!(index, 0);
    }

    #[test]
    fn test_len() {
        let mut chunk = Chunk::new();
        chunk.write_to_chunk(OpCode::Constant(1), 1);
        assert_eq!(chunk.len(), 1);
    }

    #[test]
    fn test_is_empty() {
        let mut chunk = Chunk::new();
        assert!(chunk.is_empty());
        chunk.write_to_chunk(OpCode::Constant(1), 1);
        assert!(!chunk.is_empty());
    }

    #[test]
    fn test_push_instruction() {
        let mut chunk = Chunk::new();
        chunk.push_instruction(OpCode::Constant(1));
        assert_eq!(chunk.code.len(), 1);
    }

    #[test]
    fn test_push_line() {
        let mut chunk = Chunk::new();
        chunk.push_line(1);
        assert_eq!(chunk.lines.len(), 1);
    }

    #[test]
    fn test_disassemble_chunk() {
        let mut chunk = Chunk::new();
        let constant = Value::Number(1.0);
        let index = chunk.push_constant(constant);
        chunk.write_to_chunk(OpCode::Constant(index), 1);
        chunk.disassemble_chunk("test");
    }

    #[test]
    fn test_disassemble_instruction() {
        let mut chunk = Chunk::new();
        let constant = Value::Number(1.0);
        let index = chunk.push_constant(constant);
        chunk.write_to_chunk(OpCode::Constant(index), 1);
        chunk.disassemble_instruction(0);
    }

    #[test]
    fn test_constant_instruction() {
        let mut chunk = Chunk::new();
        let constant = Value::Number(1.0);
        let index = chunk.push_constant(constant);
        chunk.write_to_chunk(OpCode::Constant(index), 1);
        chunk.constant_instruction("Constant", Some(index), 0, 1);
        assert_eq!(1, chunk.len());
    }

    #[test]
    fn test_return_instruction() {
        let mut chunk = Chunk::new();
        let code_return = OpCode::Return;
        chunk.push_instruction(code_return);
        assert_eq!(1, chunk.len());
    }

    #[test]
    fn test_false_instruction() {
        let mut chunk = Chunk::new();
        let code_false = OpCode::False;
        chunk.push_instruction(code_false);
        assert_eq!(1, chunk.len());
    }

    #[test]
    fn test_true_instruction() {
        let mut chunk = Chunk::new();
        let code_true = OpCode::True;
        chunk.push_instruction(code_true);
        assert_eq!(1, chunk.len());
    }

    #[test]
    fn test_nil_instruction() {
        let mut chunk = Chunk::new();
        let code_nil = OpCode::Nil;
        chunk.push_instruction(code_nil);
        assert_eq!(1, chunk.len());
    }
}
