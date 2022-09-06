use std::ptr;

use crate::{chunk::Chunk, compiler::Parser, op_code::OpCode, value::Value};

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

    pub fn interpret(&mut self, bytes: &[u8]) -> Result<(), InterpretError> {
        let mut parser = Parser::new(bytes, &mut self.chunk);
        if !parser.compile() {
            return Err(InterpretError::CompileError);
        }

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
                    let val = self.pop();
                    println!("Returning value of {:?}", val);
                    result = Ok(())
                }
                OpCode::Constant(v) => {
                    let val = self.chunk.constants[*v as usize];
                    println!("Executing value {:?}", val);

                    self.push(val);
                    result = Ok(());
                }
                OpCode::Negative => {
                    if let Value::Number(v) = self.pop() {
                        self.push(Value::Number(-v));
                    }

                    result = Ok(());
                }
                OpCode::Add => match self.binary_operation(OpCode::Add) {
                    Ok(number) => println!("binary add gets {}", number),
                    Err(e) => return Err(e),
                },
                OpCode::Subtract => match self.binary_operation(OpCode::Subtract) {
                    Ok(number) => println!("binary subtract gets {}", number),
                    Err(e) => return Err(e),
                },
                OpCode::Multiply => match self.binary_operation(OpCode::Multiply) {
                    Ok(number) => println!("binary multiply gets {}", number),
                    Err(e) => return Err(e),
                },
                OpCode::Divide => match self.binary_operation(OpCode::Divide) {
                    Ok(number) => println!("binary divide gets {}", number),
                    Err(e) => return Err(e),
                },
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

    fn binary_operation(&mut self, code: OpCode) -> Result<f64, InterpretError> {
        let (v1, v2) = (self.pop(), self.pop());
        match code {
            //FIXME - Refactor and simplify the code later
            OpCode::Add => {
                if let (Value::Number(x1), Value::Number(x2)) = (v1, v2) {
                    let result = x2 + x1;
                    self.push(Value::Number(result));
                    Ok(result)
                } else {
                    self.push(v1);
                    self.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            OpCode::Subtract => {
                if let (Value::Number(x1), Value::Number(x2)) = (v1, v2) {
                    let result = x2 - x1;
                    self.push(Value::Number(result));
                    Ok(result)
                } else {
                    self.push(v1);
                    self.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            OpCode::Multiply => {
                if let (Value::Number(x1), Value::Number(x2)) = (v1, v2) {
                    let result = x2 * x1;
                    self.push(Value::Number(result));
                    Ok(result)
                } else {
                    self.push(v1);
                    self.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            OpCode::Divide => {
                if let (Value::Number(x1), Value::Number(x2)) = (v1, v2) {
                    let result = x2 / x1;
                    self.push(Value::Number(result));
                    Ok(result)
                } else {
                    self.push(v1);
                    self.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            _ => Err(InterpretError::RuntimeError),
        }
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

// unit test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut vm = Vm::new();
        vm.initialize();
        vm.push(Value::Number(1.0));
        vm.push(Value::Number(2.0));
        vm.push(Value::Number(3.0));

        assert_eq!(vm.pop(), Value::Number(3.0));
        assert_eq!(vm.pop(), Value::Number(2.0));
        assert_eq!(vm.pop(), Value::Number(1.0));
    }

    #[test]
    fn test_add() {
        let mut vm = Vm::new();
        vm.initialize();
        vm.push(Value::Number(1.0));
        vm.push(Value::Number(2.0));
        vm.push(Value::Number(3.0));

        vm.binary_operation(OpCode::Add).unwrap();
        assert_eq!(vm.pop(), Value::Number(5.0));
    }

    #[test]
    fn test_subtract() {
        let mut vm = Vm::new();
        vm.initialize();
        vm.push(Value::Number(1.0));
        vm.push(Value::Number(2.0));
        vm.push(Value::Number(3.0));

        vm.binary_operation(OpCode::Subtract).unwrap();
        assert_eq!(vm.pop(), Value::Number(-1.0));
    }

    #[test]
    fn test_multiply() {
        let mut vm = Vm::new();
        vm.initialize();
        vm.push(Value::Number(1.0));
        vm.push(Value::Number(2.0));
        vm.push(Value::Number(3.0));

        vm.binary_operation(OpCode::Multiply).unwrap();
        assert_eq!(vm.pop(), Value::Number(6.0));
    }

    #[test]
    fn test_divide() {
        let mut vm = Vm::new();
        vm.initialize();
        vm.push(Value::Number(2.0));
        vm.push(Value::Number(3.0));

        vm.binary_operation(OpCode::Divide).unwrap();
        assert_eq!(vm.pop(), Value::Number(0.6666666666666666));
    }
}
