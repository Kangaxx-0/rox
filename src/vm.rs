use crate::compiler::Parser;
use crate::{
    hashtable::HashTable,
    objects::{HashKeyString, ObjFunction},
    op_code::OpCode,
    stack::Stack,
    utils::{hash, is_falsey},
    value::Value,
};

const FRAME_MAX: usize = 64;

#[derive(Debug)]
pub enum InterpretError {
    CompileError,
    RuntimeError,
    Default,
}

#[derive(Clone, Debug)]
// represents a single ongoing function call
// TODO - function calla are a core operation, can we do not use heap allocation here?
pub struct CallFrame {
    function: ObjFunction,
    ip: usize,    // when we return from a function, caller needs to know where to resume
    slots: usize, // points to vm stack at the first slot function can use
}

impl CallFrame {
    pub fn new(function: ObjFunction) -> Self {
        Self {
            function,
            ip: 0,
            slots: 0,
        }
    }
}

pub struct Vm {
    stack: Stack,
    table: HashTable,
    frames: Vec<CallFrame>,
    frame_count: usize, // current height of the call frame stack - the number of ongoing calls.
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
            table: HashTable::new(),
            frames: Vec::with_capacity(FRAME_MAX),
            frame_count: 0,
        }
    }

    pub fn initialize(&mut self) {
        self.stack.reset();
    }

    pub fn interpret(&mut self, bytes: &str) -> Result<(), InterpretError> {
        let parser = Parser::new(bytes.as_bytes());
        match parser.compile() {
            Ok(function) => {
                self.frames.push(CallFrame::new(function));
                self.run()
            }
            Err(_) => Err(InterpretError::CompileError),
        }
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    fn peek(&self, distance: usize) -> Option<&Value> {
        self.stack.peek(distance)
    }

    fn runtime_error(&mut self, frame: &CallFrame, message: &str) {
        eprint!("Runtime error: {}", message);

        let line = frame.function.chunk.lines[frame.ip - 1];

        eprintln!(" [line {}]", line);

        self.stack.reset();
    }

    fn binary_operation(&mut self, code: OpCode) -> Result<(), InterpretError> {
        let (v1, v2) = (
            self.pop().expect("unable to pop value"),
            self.pop().expect("unable to pop value"),
        );
        match code {
            //FIXME - Refactor and simplify the code later
            OpCode::Add => {
                if let (Value::Number(x1), Value::Number(x2)) = (&v1, &v2) {
                    let result = x2 + x1;
                    self.push(Value::Number(result));
                    Ok(())
                } else if let (Value::String(x1), Value::String(x2)) = (&v1, &v2) {
                    let mut result = x2.clone();
                    result.push_str(x1);
                    self.push(Value::String(result));
                    Ok(())
                } else {
                    self.push(v1);
                    self.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            OpCode::Subtract => {
                if let (Value::Number(x1), Value::Number(x2)) = (&v1, &v2) {
                    let result = x2 - x1;
                    self.push(Value::Number(result));
                    Ok(())
                } else {
                    self.push(v1);
                    self.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            OpCode::Multiply => {
                if let (Value::Number(x1), Value::Number(x2)) = (&v1, &v2) {
                    let result = x2 * x1;
                    self.push(Value::Number(result));
                    Ok(())
                } else {
                    self.push(v1);
                    self.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            OpCode::Divide => {
                if let (Value::Number(x1), Value::Number(x2)) = (&v1, &v2) {
                    let result = x2 / x1;
                    self.push(Value::Number(result));
                    Ok(())
                } else {
                    self.push(v1);
                    self.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            OpCode::Greater => {
                if let (Value::Number(x1), Value::Number(x2)) = (&v1, &v2) {
                    let result = x2 > x1;
                    self.push(Value::Bool(result));
                    Ok(())
                } else {
                    self.push(v1);
                    self.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            OpCode::Less => {
                if let (Value::Number(x1), Value::Number(x2)) = (&v1, &v2) {
                    let result = x2 < x1;
                    self.push(Value::Bool(result));
                    Ok(())
                } else {
                    self.push(v1);
                    self.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            _ => Err(InterpretError::RuntimeError),
        }
    }

    fn run(&mut self) -> Result<(), InterpretError> {
        let mut frame = self.frames.pop().expect("no current chunk");
        loop {
            let instruction = frame.function.chunk.code[frame.ip];
            // Enable this to see the chunk and stack
            frame.function.chunk.disassemble_instruction(frame.ip);
            self.print_stack();
            frame.ip += 1;
            match instruction {
                OpCode::Return => return Ok(()),
                OpCode::Constant(v) => {
                    let val = &frame.function.chunk.constants[v];
                    self.push(val.clone());
                    return Ok(());
                }
                OpCode::Negative => {
                    match self.peek(0).expect("unable to peek value") {
                        Value::Number(_) => {
                            if let Value::Number(v) = self.pop().expect("unable to pop value") {
                                self.push(Value::Number(-v));
                            }
                        }
                        _ => {
                            println!("operand must be a number");
                            return Err(InterpretError::RuntimeError);
                        }
                    }

                    return Ok(());
                }
                OpCode::Add => {
                    if self.binary_operation(OpCode::Add).is_err() {
                        self.runtime_error(&frame, "operands must be two numbers or two strings");
                        return Err(InterpretError::RuntimeError);
                    }
                }
                OpCode::Subtract => {
                    if self.binary_operation(OpCode::Subtract).is_err() {
                        self.runtime_error(&frame, "operands must be two numbers");
                        return Err(InterpretError::RuntimeError);
                    }
                }
                OpCode::Multiply => {
                    if self.binary_operation(OpCode::Multiply).is_err() {
                        self.runtime_error(&frame, "operands must be two numbers");
                        return Err(InterpretError::RuntimeError);
                    }
                }
                OpCode::Divide => {
                    if self.binary_operation(OpCode::Divide).is_err() {
                        self.runtime_error(&frame, "operands must be two numbers");
                        return Err(InterpretError::RuntimeError);
                    }
                }
                OpCode::Nil => {
                    self.push(Value::Nil);
                    return Ok(());
                }
                OpCode::True => {
                    self.push(Value::Bool(true));
                    return Ok(());
                }
                OpCode::False => {
                    self.push(Value::Bool(false));
                    return Ok(());
                }
                OpCode::Not => {
                    let val = self.pop().expect("unable to pop value");
                    self.push(Value::Bool(is_falsey(&val)));
                    return Ok(());
                }
                OpCode::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a == b));
                    return Ok(());
                }
                OpCode::Greater => self.binary_operation(OpCode::Greater)?,
                OpCode::Less => self.binary_operation(OpCode::Less)?,
                OpCode::Pop => {
                    self.pop();
                    return Ok(());
                }
                OpCode::Print => {
                    let val = self.pop().expect("unable to pop value");
                    println!("Printing value of {}", val);
                    return Ok(());
                }
                OpCode::DefineGlobal(v) => {
                    if let Value::String(s) = &frame.function.chunk.constants[v] {
                        let key = HashKeyString {
                            value: s.clone(),
                            hash: hash(s),
                        };
                        let val = self.pop().expect("unable to pop value");
                        self.table.insert(key, val);
                    }
                    return Ok(());
                }
                OpCode::GetGlobal(v) => {
                    if let Value::String(s) = &frame.function.chunk.constants[v] {
                        let key = HashKeyString {
                            value: s.clone(),
                            hash: hash(s),
                        };
                        if let Some(val) = self.table.get(&key) {
                            self.push(val.clone());
                        } else {
                            self.runtime_error(
                                &frame,
                                format!("undefined variable '{}'", s).as_str(),
                            );
                            return Err(InterpretError::RuntimeError);
                        }
                    }
                    return Ok(());
                }
                OpCode::SetGlobal(v) => {
                    if let Value::String(s) = &frame.function.chunk.constants[v] {
                        let key = HashKeyString {
                            value: s.clone(),
                            hash: hash(s),
                        };
                        if self.table.get(&key).is_some() {
                            // We do not want to pop the value off the stack because it might be
                            // re-used in other places. e.g. a = 1; b = a + 1; c = 2+a; print c;
                            // should print 3
                            let val = self.peek(0).expect("unable to peek value");
                            // insert would replace the value with the same key
                            self.table.insert(key, val.clone());
                        } else {
                            // when the key does note exist in the global has table, we throw a runtime error
                            self.runtime_error(
                                &frame,
                                format!("undefined variable '{}'", s).as_str(),
                            );
                            return Err(InterpretError::RuntimeError);
                        }
                    }
                    return Ok(());
                }
                OpCode::GetLocal(v) => {
                    let val = &self.stack.values[v];
                    self.push(val.clone());
                    return Ok(());
                }
                OpCode::SetLocal(v) => {
                    let val = self.peek(0).expect("unable to pop value");
                    self.stack.values[v] = val.clone();
                    return Ok(());
                }
                OpCode::JumpIfFalse(offset) => {
                    if is_falsey(self.peek(0).expect("unable to peek value")) {
                        frame.ip += offset as usize;
                    }
                    return Ok(());
                }
                OpCode::Jump(offset) => {
                    frame.ip += offset as usize;
                    return Ok(());
                }
                OpCode::Loop(offset) => {
                    frame.ip -= offset as usize;
                    // We need to subtract 1 from the ip because the ip will be incremented by 1
                    // at the end of the loop
                    frame.ip -= 1;
                    return Ok(());
                }
                _ => {
                    println!("Unknown operation code during interpreting!");
                    return Err(InterpretError::RuntimeError);
                }
            }
        }
    }

    fn print_stack(&self) {
        for value in self.stack.clone() {
            println!("[{}]", value);
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

    #[test]
    fn test_string() {
        let mut vm = Vm::new();
        vm.initialize();
        vm.stack.push(Value::String("hello".to_string()));
        assert_eq!(vm.stack.pop(), Some(Value::String("hello".to_string())));
    }

    #[test]
    fn test_less() {
        let mut vm = Vm::new();
        vm.initialize();
        vm.stack.push(Value::Number(1.0));
        vm.stack.push(Value::Number(2.0));
        vm.stack.push(Value::Number(3.0));

        vm.binary_operation(OpCode::Less).unwrap();
        assert_eq!(vm.stack.pop(), Some(Value::Bool(true)));
    }

    #[test]
    fn test_greater() {
        let mut vm = Vm::new();
        vm.initialize();
        vm.stack.push(Value::Number(1.0));
        vm.stack.push(Value::Number(2.0));
        vm.stack.push(Value::Number(3.0));

        vm.binary_operation(OpCode::Greater).unwrap();
        assert_eq!(vm.stack.pop(), Some(Value::Bool(false)));
    }
}
