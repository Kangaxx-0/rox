use crate::{
    chunk::Chunk,
    compiler::Parser,
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
    pub fn new(function: ObjFunction, ip: usize, slots: usize) -> Self {
        Self {
            function,
            ip,
            slots,
        }
    }
}

pub struct Vm {
    // ip: usize,
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
            Ok(obj) => {
                self.run(obj);
                Ok(())
            }
            Err(_) => Err(InterpretError::CompileError),
        }
    }

    fn current_frame(&self) -> &CallFrame {
        self.frames.last().expect("no current frame")
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("no current frame")
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn run(&mut self, obj: ObjFunction) -> Result<(), InterpretError> {
        let mut result = Err(InterpretError::Default);
        let chunk = &mut self.frames[self.frame_count - 1].function.chunk;
        loop {
            // if chunk.ip == obj.chunk.len() {
            //     break;
            // }
            let chunk = &obj.chunk;
            // Enable this to see the chunk and stack
            // obj.chunk.disassemble_instruction(frame.ip);
            // self.print_stack();
            match &chunk.code[frame.ip] {
                OpCode::Return => result = Ok(()),
                OpCode::Constant(v) => {
                    let val = &obj.chunk.constants[*v];
                    self.push(val.clone());
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
                OpCode::Add => {
                    if self.binary_operation(OpCode::Add).is_err() {
                        self.runtime_error(&obj, "operands must be two numbers or two strings");
                        return Err(InterpretError::RuntimeError);
                    }
                }
                OpCode::Subtract => {
                    if self.binary_operation(OpCode::Subtract).is_err() {
                        self.runtime_error(&obj, "operands must be two numbers");
                        return Err(InterpretError::RuntimeError);
                    }
                }
                OpCode::Multiply => {
                    if self.binary_operation(OpCode::Multiply).is_err() {
                        self.runtime_error(&obj, "operands must be two numbers");
                        return Err(InterpretError::RuntimeError);
                    }
                }
                OpCode::Divide => {
                    if self.binary_operation(OpCode::Divide).is_err() {
                        self.runtime_error(&obj, "operands must be two numbers");
                        return Err(InterpretError::RuntimeError);
                    }
                }
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
                    self.stack.push(Value::Bool(is_falsey(&val)));
                    result = Ok(());
                }
                OpCode::Equal => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    self.stack.push(Value::Bool(a == b));
                    result = Ok(());
                }
                OpCode::Greater => self.binary_operation(OpCode::Greater)?,
                OpCode::Less => self.binary_operation(OpCode::Less)?,
                OpCode::Pop => {
                    self.stack.pop();
                    result = Ok(());
                }
                OpCode::Print => {
                    let val = self.stack.pop().expect("unable to pop value");
                    println!("Printing value of {}", val);
                    result = Ok(());
                }
                OpCode::DefineGlobal(v) => {
                    if let Value::String(s) = &obj.chunk.constants[*v] {
                        let key = HashKeyString {
                            value: s.clone(),
                            hash: hash(s),
                        };
                        self.table
                            .insert(key, self.stack.pop().expect("unable to pop value"));
                    }
                    result = Ok(());
                }
                OpCode::GetGlobal(v) => {
                    if let Value::String(s) = &obj.chunk.constants[*v] {
                        let key = HashKeyString {
                            value: s.clone(),
                            hash: hash(s),
                        };
                        if let Some(val) = self.table.get(&key) {
                            self.stack.push(val.clone());
                        } else {
                            self.runtime_error(
                                &obj,
                                format!("undefined variable '{}'", s).as_str(),
                            );
                            return Err(InterpretError::RuntimeError);
                        }
                    }
                    result = Ok(());
                }
                OpCode::SetGlobal(v) => {
                    if let Value::String(s) = &obj.chunk.constants[*v] {
                        let key = HashKeyString {
                            value: s.clone(),
                            hash: hash(s),
                        };
                        if self.table.get(&key).is_some() {
                            // We do not want to pop the value off the stack because it might be
                            // re-used in other places. e.g. a = 1; b = a + 1; c = 2+a; print c;
                            // should print 3
                            let val = self.stack.peek(0).expect("unable to peek value");
                            // insert would replace the value with the same key
                            self.table.insert(key, val.clone());
                        } else {
                            // when the key does note exist in the global has table, we throw a runtime error
                            self.runtime_error(
                                &obj,
                                format!("undefined variable '{}'", s).as_str(),
                            );
                            return Err(InterpretError::RuntimeError);
                        }
                    }
                    result = Ok(());
                }
                OpCode::GetLocal(v) => {
                    let val = &self.stack.values[*v];
                    self.stack.push(val.clone());
                    result = Ok(());
                }
                OpCode::SetLocal(v) => {
                    let val = self.stack.peek(0).expect("unable to pop value");
                    self.stack.values[*v] = val.clone();
                    result = Ok(());
                }
                OpCode::JumpIfFalse(offset) => {
                    if is_falsey(self.stack.peek(0).expect("unable to peek value")) {
                        frame.ip += *offset as usize;
                    }
                    result = Ok(());
                }
                OpCode::Jump(offset) => {
                    frame.ip += *offset as usize;
                    result = Ok(());
                }
                OpCode::Loop(offset) => {
                    frame.ip -= *offset as usize;
                    // We need to subtract 1 from the ip because the ip will be incremented by 1
                    // at the end of the loop
                    frame.ip -= 1;
                    result = Ok(());
                }
                _ => {
                    println!("Unknown operation code during interpreting!");
                    result = Err(InterpretError::RuntimeError);
                }
            }

            //FIXME - Can we come up with a better idea to exit the loop, then we might not need
            //the instruction pointer at all.
            frame.ip += 1;
        }

        result
    }

    fn runtime_error(&mut self, obj: &ObjFunction, message: &str) {
        let frame = &self.frames[self.frame_count - 1];
        eprint!("Runtime error: {}", message);

        let instruction = frame.ip - obj.chunk.code.len() - 1;
        let line = obj.chunk.lines[instruction];

        eprintln!(" [line {}]", line);

        self.stack.reset();
    }

    fn binary_operation(&mut self, code: OpCode) -> Result<(), InterpretError> {
        let (v1, v2) = (
            self.stack.pop().expect("unable to pop value"),
            self.stack.pop().expect("unable to pop value"),
        );
        match code {
            //FIXME - Refactor and simplify the code later
            OpCode::Add => {
                if let (Value::Number(x1), Value::Number(x2)) = (&v1, &v2) {
                    let result = x2 + x1;
                    self.stack.push(Value::Number(result));
                    Ok(())
                } else if let (Value::String(x1), Value::String(x2)) = (&v1, &v2) {
                    let mut result = x2.clone();
                    result.push_str(x1);
                    self.stack.push(Value::String(result));
                    Ok(())
                } else {
                    self.stack.push(v1);
                    self.stack.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            OpCode::Subtract => {
                if let (Value::Number(x1), Value::Number(x2)) = (&v1, &v2) {
                    let result = x2 - x1;
                    self.stack.push(Value::Number(result));
                    Ok(())
                } else {
                    self.stack.push(v1);
                    self.stack.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            OpCode::Multiply => {
                if let (Value::Number(x1), Value::Number(x2)) = (&v1, &v2) {
                    let result = x2 * x1;
                    self.stack.push(Value::Number(result));
                    Ok(())
                } else {
                    self.stack.push(v1);
                    self.stack.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            OpCode::Divide => {
                if let (Value::Number(x1), Value::Number(x2)) = (&v1, &v2) {
                    let result = x2 / x1;
                    self.stack.push(Value::Number(result));
                    Ok(())
                } else {
                    self.stack.push(v1);
                    self.stack.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            OpCode::Greater => {
                if let (Value::Number(x1), Value::Number(x2)) = (&v1, &v2) {
                    let result = x2 > x1;
                    self.stack.push(Value::Bool(result));
                    Ok(())
                } else {
                    self.stack.push(v1);
                    self.stack.push(v2);
                    Err(InterpretError::RuntimeError)
                }
            }
            OpCode::Less => {
                if let (Value::Number(x1), Value::Number(x2)) = (&v1, &v2) {
                    let result = x2 < x1;
                    self.stack.push(Value::Bool(result));
                    Ok(())
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
