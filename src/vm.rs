use crate::{
    chunk::Chunk, compiler::Parser, op_code::OpCode, stack::Stack, utils::is_falsey, value::Value,
};

#[derive(Debug)]
pub enum InterpretError {
    CompileError,
    RuntimeError,
    Default,
}

pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: Stack,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            ip: 0,
            stack: Stack::new(),
        }
    }

    pub fn initialize(&mut self) {
        self.stack.reset();
    }

    pub fn interpret(&mut self, bytes: &[u8]) -> Result<(), InterpretError> {
        let mut parser = Parser::new(bytes, &mut self.chunk);
        if !parser.compile() {
            return Err(InterpretError::CompileError);
        }

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
            self.print_stack();
            self.chunk.disassemble_instruction(self.ip);
            match &chunk.code[self.ip] {
                OpCode::Return => {
                    let val = self.stack.pop().expect("unable to pop value from stack");
                    println!("Returning value of {:?}", val);
                    result = Ok(())
                }
                OpCode::Constant(v) => {
                    let val = self.chunk.constants[*v as usize];
                    println!("Executing value {:?}", val);

                    self.stack.push(val);
                    result = Ok(());
                }
                OpCode::Negative => {
                    match self.stack.peek(0).expect("unable to peek value") {
                        Value::Number(_) => {
                            if let Value::Number(v) = self.stack.pop().expect("unable to pop value")
                            {
                                self.stack.push(Value::Number(-v));
                            }
                        }
                        _ => {
                            println!("operand must be a number");
                            return Err(InterpretError::RuntimeError);
                        }
                    }

                    result = Ok(());
                }
                OpCode::Add => match self.binary_operation(OpCode::Add) {
                    Ok(number) => println!("binary add gets {}", number),
                    Err(e) => {
                        self.runtime_error("operands must be two numbers");
                        return Err(e);
                    }
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
                OpCode::Nil => {
                    self.stack.push(Value::Nil);
                    result = Ok(());
                }
                OpCode::True => {
                    self.stack.push(Value::Bool(true));
                    result = Ok(());
                }
                OpCode::False => {
                    self.stack.push(Value::Bool(false));
                    result = Ok(());
                }
                OpCode::Not => {
                    let val = self.stack.pop().expect("unable to pop value");
                    self.stack.push(Value::Bool(is_falsey(val)));
                    result = Ok(());
                }
                OpCode::Equal => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    self.stack.push(Value::Bool(a == b));
                    result = Ok(());
                }
                OpCode::Greater => match self.binary_operation(OpCode::Greater) {
                    Ok(number) => println!("binary greater gets {}", number),
                    Err(e) => return Err(e),
                },
                OpCode::Less => match self.binary_operation(OpCode::Less) {
                    Ok(number) => println!("binary less gets {}", number),
                    Err(e) => return Err(e),
                },
                OpCode::Print => {
                    let val = self.stack.pop().expect("unable to pop value");
                    println!("Printing value of {:?}", val);
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

    fn runtime_error(&mut self, message: &str) {
        eprint!("Runtime error: {}", message);

        let instruction = self.ip - self.chunk.code.len() - 1;
        let line = self.chunk.lines[instruction];

        eprintln!(" [line {}]", line);

        self.stack.reset();
    }

    fn binary_operation(&mut self, code: OpCode) -> Result<f64, InterpretError> {
        let (v1, v2) = (
            self.stack.pop().expect("unable to pop value"),
            self.stack.pop().expect("unable to pop value"),
        );
        match code {
            //FIXME - Refactor and simplify the code later
            OpCode::Add => {
                if let (Value::Number(x1), Value::Number(x2)) = (v1, v2) {
                    let result = x2 + x1;
                    self.stack.push(Value::Number(result));
                    Ok(result)
                } else {
                    self.stack.push(v1);
                    self.stack.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            OpCode::Subtract => {
                if let (Value::Number(x1), Value::Number(x2)) = (v1, v2) {
                    let result = x2 - x1;
                    self.stack.push(Value::Number(result));
                    Ok(result)
                } else {
                    self.stack.push(v1);
                    self.stack.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            OpCode::Multiply => {
                if let (Value::Number(x1), Value::Number(x2)) = (v1, v2) {
                    let result = x2 * x1;
                    self.stack.push(Value::Number(result));
                    Ok(result)
                } else {
                    self.stack.push(v1);
                    self.stack.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            OpCode::Divide => {
                if let (Value::Number(x1), Value::Number(x2)) = (v1, v2) {
                    let result = x2 / x1;
                    self.stack.push(Value::Number(result));
                    Ok(result)
                } else {
                    self.stack.push(v1);
                    self.stack.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            OpCode::Greater => {
                if let (Value::Number(x1), Value::Number(x2)) = (v1, v2) {
                    let result = x2 > x1;
                    self.stack.push(Value::Bool(result));
                    Ok(result as i32 as f64)
                } else {
                    self.stack.push(v1);
                    self.stack.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            OpCode::Less => {
                if let (Value::Number(x1), Value::Number(x2)) = (v1, v2) {
                    let result = x2 < x1;
                    self.stack.push(Value::Bool(result));
                    Ok(result as i32 as f64)
                } else {
                    self.stack.push(v1);
                    self.stack.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            _ => Err(InterpretError::RuntimeError),
        }
    }

    fn print_stack(&self) {
        for value in self.stack.clone() {
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
        vm.stack.push(Value::Number(1.0));
        vm.stack.push(Value::Number(2.0));
        vm.stack.push(Value::Number(3.0));

        assert_eq!(vm.stack.pop(), Some(Value::Number(3.0)));
        assert_eq!(vm.stack.pop(), Some(Value::Number(2.0)));
        assert_eq!(vm.stack.pop(), Some(Value::Number(1.0)));
    }

    #[test]
    fn test_add() {
        let mut vm = Vm::new();
        vm.initialize();
        vm.stack.push(Value::Number(1.0));
        vm.stack.push(Value::Number(2.0));
        vm.stack.push(Value::Number(3.0));

        vm.binary_operation(OpCode::Add).unwrap();
        assert_eq!(vm.stack.pop(), Some(Value::Number(5.0)));
    }

    #[test]
    fn test_subtract() {
        let mut vm = Vm::new();
        vm.initialize();
        vm.stack.push(Value::Number(1.0));
        vm.stack.push(Value::Number(2.0));
        vm.stack.push(Value::Number(3.0));

        vm.binary_operation(OpCode::Subtract).unwrap();
        assert_eq!(vm.stack.pop(), Some(Value::Number(-1.0)));
    }

    #[test]
    fn test_multiply() {
        let mut vm = Vm::new();
        vm.initialize();
        vm.stack.push(Value::Number(1.0));
        vm.stack.push(Value::Number(2.0));
        vm.stack.push(Value::Number(3.0));

        vm.binary_operation(OpCode::Multiply).unwrap();
        assert_eq!(vm.stack.pop(), Some(Value::Number(6.0)));
    }

    #[test]
    fn test_divide() {
        let mut vm = Vm::new();
        vm.initialize();
        vm.stack.push(Value::Number(2.0));
        vm.stack.push(Value::Number(3.0));

        vm.binary_operation(OpCode::Divide).unwrap();
        assert_eq!(vm.stack.pop(), Some(Value::Number(0.6666666666666666)));
    }

    #[test]
    fn test_true() {
        let mut vm = Vm::new();
        vm.initialize();
        vm.stack.push(Value::Bool(true));
        assert_eq!(vm.stack.pop(), Some(Value::Bool(true)));
    }

    #[test]
    fn test_false() {
        let mut vm = Vm::new();
        vm.initialize();
        vm.stack.push(Value::Bool(false));
        assert_eq!(vm.stack.pop(), Some(Value::Bool(false)));
    }
}
