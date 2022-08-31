use std::ptr;

use crate::{chunk::Chunk, op_code::OpCode, value::Value};

const STACK_MAX: usize = 20;

#[derive(Debug)]
pub enum InterpretError {
    CompileError,
    RuntimeError,
    Default,
}

pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: [Value; STACK_MAX],
    stack_top: *mut Value,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            ip: 0,
            stack: [Value::Nil; STACK_MAX],
            stack_top: ptr::null_mut(),
        }
    }

    pub fn initialize(&mut self) {
        self.stack_top = self.stack.as_mut_ptr();
    }

    pub fn interpret(&mut self, chunk: Chunk) -> Result<(), InterpretError> {
        self.chunk = chunk;

        self.run()
    }

    pub fn push(&mut self, value: Value) {
        // SAFETY: the pushing value will not be null, and offset cannot overflow an `isize`
        unsafe {
            *self.stack_top = value;
            self.stack_top = self.stack_top.add(1);
        }
    }

    pub fn pop(&mut self) -> Value {
        if let Value::Nil = self.stack[0] {
            panic!("The stack is empty, cannot pop value")
        }
        // SAFETY: We never pop out value from empty stack, so the index won't be negative, and offset cannot overflow an `isize`
        // Also, we do not need to explicitly remove value from array, move `stackTop` down is
        // enough to mark that slot as no longer in use
        unsafe {
            self.stack_top = self.stack_top.sub(1);
            *self.stack_top
        }
    }

    fn run(&mut self) -> Result<(), InterpretError> {
        let mut result = Err(InterpretError::Default);
        loop {
            if self.ip == self.chunk.len() {
                println!("Reaching the last instruction, exiting...");
                break;
            }
            let chunk = &self.chunk;
            self.print_stack();
            self.chunk.disassemble_instruction(self.ip);
            match &chunk.code[self.ip] {
                OpCode::Return => {
                    let v = self.pop();
                    println!("Returning value of {:?}", v);
                    result = Ok(())
                }
                OpCode::Constant(v) => {
                    let v = self.chunk.constants[*v as usize];
                    println!("Executing value {:?}", v);

                    self.push(v);
                    result = Ok(());
                }
                _ => {
                    println!("Unknown operation code during interpreting!");
                    result = Err(InterpretError::RuntimeError);
                }
            }

            //FIXME - Can we come up with a better idea to exit the loop, then we might not need
            //the instruction pointer at all.
            self.ip += 1;
        }

        result
    }

    fn print_stack(&self) {
        for value in self.stack {
            println!("[{:?}]", value);
        }
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}
