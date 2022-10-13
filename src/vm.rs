use crate::chunk::Chunk;
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
                self.push(Value::Function(function.clone()));
                self.call(function, 0);
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

    fn call_value(&mut self, callee: Value, arg_count: usize) -> bool {
        match callee {
            Value::Function(function) => self.call(function, arg_count),
            _ => {
                println!("Can only call functions and classes.");
                false
            }
        }
    }

    fn call(&mut self, function: ObjFunction, arg_count: usize) -> bool {
        if arg_count != function.arity as usize {
            println!(
                "Expected {} arguments but got {}.",
                function.arity, arg_count
            );
            return false;
        }

        if self.frame_count == FRAME_MAX {
            println!("Stack overflow!");
            return false;
        }

        let stack_top = self.stack.len() - arg_count - 1;
        let mut frame = CallFrame::new(function);
        frame.ip = 0;
        frame.slots = stack_top;
        self.frames.push(frame);
        self.frame_count += 1;
        true
    }

    fn runtime_error(&mut self, message: &str) {
        eprint!("Runtime error: {}", message);

        let line = self.current_line();

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

    fn current_frame(&self) -> &CallFrame {
        &self.frames[self.frame_count - 1]
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        &mut self.frames[self.frame_count - 1]
    }

    fn current_chunk(&self) -> &Chunk {
        &self.current_frame().function.chunk
    }

    fn current_line(&self) -> usize {
        self.current_chunk().lines[self.current_frame().ip - 1]
    }

    fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            let instruction = self.current_chunk().code[self.current_frame().ip];
            // Enable this to see the chunk and stack
            self.current_chunk()
                .disassemble_instruction(self.current_frame().ip);
            self.print_stack();
            self.current_frame_mut().ip += 1;
            match instruction {
                OpCode::Return => return Ok(()),
                OpCode::Constant(v) => {
                    let val = self.current_chunk().constants[v].clone();
                    self.push(val);
                }
                OpCode::Negative => match self.peek(0).expect("unable to peek value") {
                    Value::Number(_) => {
                        if let Value::Number(v) = self.pop().expect("unable to pop value") {
                            self.push(Value::Number(-v));
                        }
                    }
                    _ => {
                        println!("operand must be a number");
                        return Err(InterpretError::RuntimeError);
                    }
                },
                OpCode::Add => {
                    if self.binary_operation(OpCode::Add).is_err() {
                        self.runtime_error("operands must be two numbers or two strings");
                        return Err(InterpretError::RuntimeError);
                    }
                }
                OpCode::Subtract => {
                    if self.binary_operation(OpCode::Subtract).is_err() {
                        self.runtime_error("operands must be two numbers");
                        return Err(InterpretError::RuntimeError);
                    }
                }
                OpCode::Multiply => {
                    if self.binary_operation(OpCode::Multiply).is_err() {
                        self.runtime_error("operands must be two numbers");
                        return Err(InterpretError::RuntimeError);
                    }
                }
                OpCode::Divide => {
                    if self.binary_operation(OpCode::Divide).is_err() {
                        self.runtime_error("operands must be two numbers");
                        return Err(InterpretError::RuntimeError);
                    }
                }
                OpCode::Nil => {
                    self.push(Value::Nil);
                }
                OpCode::True => {
                    self.push(Value::Bool(true));
                }
                OpCode::False => {
                    self.push(Value::Bool(false));
                }
                OpCode::Not => {
                    let val = self.pop().expect("unable to pop value");
                    self.push(Value::Bool(is_falsey(&val)));
                }
                OpCode::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a == b));
                }
                OpCode::Greater => self.binary_operation(OpCode::Greater)?,
                OpCode::Less => self.binary_operation(OpCode::Less)?,
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::Print => {
                    let val = self.pop().expect("unable to pop value");
                    match val {
                        Value::Function(v) => println!("{}", v.name.value),
                        Value::String(v) => println!("Printing value of {}", v),
                        Value::Number(v) => println!("Printing value of {}", v),
                        Value::Bool(v) => println!("Printing value of {}", v),
                        Value::Nil => println!("nil"),
                        _ => println!("unknown value"),
                    }
                }
                OpCode::DefineGlobal(v) => {
                    if let Value::String(s) = &self.current_frame().function.chunk.constants[v] {
                        let key = HashKeyString {
                            hash: hash(s),
                            value: s.to_string(),
                        };
                        let val = self.pop().expect("unable to pop value");
                        self.table.insert(key, val);
                    }
                }
                OpCode::GetGlobal(v) => {
                    if let Value::String(s) = &self.current_frame().function.chunk.constants[v] {
                        let key = HashKeyString {
                            hash: hash(s),
                            value: s.to_string(),
                        };
                        if let Some(val) = self.table.get(&key) {
                            self.push(val.clone());
                        } else {
                            self.runtime_error(format!("undefined variable '{}'", s).as_str());
                            return Err(InterpretError::RuntimeError);
                        }
                    }
                }
                OpCode::SetGlobal(v) => {
                    if let Value::String(s) = &self.current_frame().function.chunk.constants[v] {
                        let key = HashKeyString {
                            hash: hash(s),
                            value: s.to_string(),
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
                            self.runtime_error(format!("undefined variable '{}'", s).as_str());
                            return Err(InterpretError::RuntimeError);
                        }
                    }
                }
                OpCode::GetLocal(v) => {
                    let addr = self.current_frame().slots + v + 1;
                    let val = &self.stack.values[addr];
                    self.push(val.clone());
                }
                OpCode::SetLocal(v) => {
                    let addr = self.current_frame().slots + v + 1;
                    let val = self.peek(0).expect("unable to pop value");
                    self.stack.values[addr] = val.clone();
                }
                OpCode::JumpIfFalse(offset) => {
                    if is_falsey(self.peek(0).expect("unable to peek value")) {
                        self.current_frame_mut().ip += offset as usize;
                    }
                }
                OpCode::Jump(offset) => {
                    self.current_frame_mut().ip += offset as usize;
                }
                OpCode::Loop(offset) => {
                    self.current_frame_mut().ip -= offset as usize;
                    // We need to subtract 1 from the ip because the ip will be incremented by 1
                    // at the end of the loop
                    self.current_frame_mut().ip -= 1;
                }
                OpCode::Call(arg_count) => {
                    if !self.call_value(
                        self.peek(arg_count).expect("unable to peek value").clone(),
                        arg_count,
                    ) {
                        return Err(InterpretError::RuntimeError);
                    }
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
