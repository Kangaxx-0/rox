use crate::{chunk::Chunk, op_code::OpCode};

#[derive(Debug)]
pub enum InterpretError {
    CompileError,
    RunetimeError,
    Default,
}

pub struct Vm {
    chunk: Chunk,
    ip: usize,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            ip: 0,
        }
    }

    pub fn interprete(&mut self, chunk: Chunk) -> Result<(), InterpretError> {
        self.chunk = chunk;

        self.run()
    }

    fn run(&mut self) -> Result<(), InterpretError> {
        let mut result = Err(InterpretError::Default);
        loop {
            if self.ip == self.chunk.len() {
                println!("Reaching the last instruction, exiting...");
                break;
            }
            let chunk = &self.chunk;
            self.chunk.disassemble_instruction(self.ip);
            match &chunk.code[self.ip] {
                OpCode::Return => {
                    println!("Executing return statement!");
                    result = Ok(())
                }
                OpCode::Constant(v) => {
                    println!("Executing value {}", v);
                    result = Ok(());
                }
                _ => {
                    println!("Unknown operation code during interpreting!");
                    result = Err(InterpretError::RunetimeError);
                }
            }

            //FIXME - Can we come up with a better idea to exit the loop, then we might not need
            //the instruction pointer at all.
            self.ip += 1;
        }

        result
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}
