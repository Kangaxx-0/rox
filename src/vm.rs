use std::time::{SystemTime, UNIX_EPOCH};

use rox_gc::Gc;

use crate::chunk::Chunk;
use crate::compiler::Parser;
use crate::objects::{ObjClosure, ObjUpValue, MAX_UPVALUES};
use crate::{
    hashtable::HashTable,
    objects::{HashKeyString, ObjNative},
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
// TODO - function calls are a core operation, can we do not use heap allocation here?
pub struct CallFrame {
    closure: ObjClosure,
    ip: usize,    // when we return from a function, caller needs to know where to resume
    slots: usize, // points to vm stack at the first slot function can use
}

impl CallFrame {
    pub fn new(closure: ObjClosure) -> Self {
        Self {
            closure,
            ip: 0,
            slots: 0,
        }
    }
}

pub struct Vm {
    stack: Stack,
    table: HashTable,
    frames: Vec<CallFrame>,
    // Gc managed heap allocation is used for both vm open_values
    // and ObjClosure upvalues
    open_values: Vec<Gc<ObjUpValue>>,
}

impl Vm {
    pub fn new() -> Self {
        let mut res = Self {
            stack: Stack::new(),
            table: HashTable::new(),
            frames: Vec::with_capacity(FRAME_MAX),
            open_values: Vec::with_capacity(MAX_UPVALUES),
        };
        res.define_native(ObjNative::new("clock".to_string(), clock_native));

        res
    }

    pub fn initialize(&mut self) {
        self.stack.reset();
    }

    pub fn interpret(&mut self, bytes: &str) -> Result<(), InterpretError> {
        let parser = Parser::new(bytes.as_bytes());
        match parser.compile() {
            Ok(function) => {
                // script function is always at the top of the stack
                let closure = ObjClosure::new(function);
                let gc_closure = Gc::new(closure);
                self.pop();
                self.push(Value::Closure(gc_closure.clone()));
                self.call(&gc_closure, 0);
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

    // We do not want to pop the value out of the stack if it just peeks
    fn peek(&self, distance: usize) -> Option<&Value> {
        self.stack.peek(distance)
    }

    fn call_value(&mut self, callee: Value, arg_count: usize) -> bool {
        match &callee {
            // call a function will push the callee to call frame which represents a single ongoing function call
            Value::Closure(closure) => self.call(closure, arg_count),
            Value::NativeFunction(native) => {
                let idx = self.stack.len() - arg_count;
                let result = (native.func)(&self.stack.values[idx..]);
                self.stack.values.truncate(idx - 1);
                self.push(result);
                true
            }
            _ => {
                println!("Can only call functions and classes.");
                false
            }
        }
    }

    fn call(&mut self, closure: &ObjClosure, arg_count: usize) -> bool {
        if arg_count != closure.function.arity as usize {
            println!(
                "Expected {} arguments but got {}.",
                closure.function.arity, arg_count
            );
            return false;
        }

        if self.frames.len() == FRAME_MAX {
            println!("Stack overflow!");
            return false;
        }

        // calculate the stack start slot for the function
        let stack_top = self.stack.len() - arg_count - 1;
        let mut frame = CallFrame::new(closure.clone());
        frame.ip = 0;
        frame.slots = stack_top;
        self.frames.push(frame);
        true
    }

    fn capture_upvalue(&mut self, index: usize) -> Gc<ObjUpValue> {
        for vm_upvalue in self.open_values.iter_mut() {
            if vm_upvalue.location == index {
                return std::mem::take(vm_upvalue);
            }
        }
        let upvalue = Gc::new(ObjUpValue::new(index));
        self.open_values.push(upvalue.clone());
        upvalue
    }

    fn close_upvalues(&mut self, index: usize) {
        let mut i = 0;
        while i != self.open_values.len() {
            let upvalue = &self.open_values[i];
            if upvalue.location >= index {
                let upvalue = self.open_values.remove(i);
                let local = upvalue.location;
                *upvalue.closed.borrow_mut() = Some(self.stack.values[local].clone());
            } else {
                i += 1;
            }
        }
    }

    fn define_native(&mut self, native: ObjNative) {
        self.table
            .insert(native.name.clone(), Value::NativeFunction(Gc::new(native)));
    }

    fn runtime_error(&mut self, message: &str) {
        eprint!("Runtime error: {}", message);

        let line = self.current_line();

        eprintln!(" [line {}]", line);

        for frame in self.frames.iter().rev() {
            let function = &frame.closure.function;
            let line = function.chunk.lines[frame.ip - 1];
            eprintln!("[line {}] in {}", line, function.name.value);
        }

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
                } else if let (Value::String(s1), Value::String(s2)) = (&v1, &v2) {
                    let result = format!("{}{}", s2, s1);
                    self.push(Value::String(Gc::new(result)));
                    Ok(())
                } else {
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
        self.frames.last().expect("unable to get current frame")
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("unable to get current frame")
    }

    fn current_chunk(&self) -> &Chunk {
        &self.current_frame().closure.function.chunk
    }

    fn current_line(&self) -> usize {
        self.current_chunk().lines[self.current_frame().ip - 1]
    }

    fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            let instruction = self.current_chunk().code[self.current_frame().ip];
            // Enable this to see the chunk and stack
            // self.current_chunk()
            //     .disassemble_instruction(self.current_frame().ip);
            // self.print_stack();
            self.current_frame_mut().ip += 1;
            match instruction {
                OpCode::Return => {
                    // When a function returns, we pop the top value off the stack and discard it.
                    let res = self.pop().expect("unable to pop value");
                    // Discard the call frame for the returning function.
                    let frame = self.frames.pop().expect("unable to pop frame");
                    self.close_upvalues(frame.slots);
                    if self.frames.is_empty() {
                        // we've finished executing the top-level code. We are done
                        return Ok(());
                    } else {
                        // the call is done, the caller does not need it anymore, the top of the stack
                        // ends up right at the beginning of the returning function's stack window
                        self.stack.values.truncate(frame.slots);
                        self.push(res);
                    }
                }
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
                OpCode::CloseUpvalue => {
                    self.close_upvalues(self.stack.values.len() - 1);
                    self.pop();
                }
                OpCode::Print => {
                    let val = self.pop().expect("unable to pop value");
                    match &val {
                        Value::Function(v) => println!("{}", v.name.value),
                        Value::String(v) => println!("Printing value of {}", v),
                        Value::Number(v) => println!("Printing value of {}", v),
                        Value::Bool(v) => println!("Printing value of {}", v),
                        Value::Nil => println!("nil"),
                        _ => println!("unknown value"),
                    }
                }
                OpCode::DefineGlobal(v) => {
                    if let Value::String(s) =
                        &self.current_frame().closure.function.chunk.constants[v]
                    {
                        let key = HashKeyString {
                            hash: hash(s),
                            value: s.to_string(),
                        };
                        let val = self.pop().expect("unable to pop value");
                        self.table.insert(key, val);
                    }
                }
                OpCode::GetGlobal(v) => {
                    if let Value::String(s) =
                        &self.current_frame().closure.function.chunk.constants[v]
                    {
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
                    if let Value::String(s) =
                        &self.current_frame().closure.function.chunk.constants[v]
                    {
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
                OpCode::GetLocal(index) => {
                    let addr = self.current_frame().slots + index + 1;
                    let val = &self.stack.values[addr];
                    self.push(val.clone());
                }
                OpCode::GetUpvalue(index) => {
                    let val = &self.current_frame().closure.obj_upvalues[index];
                    let res = {
                        if let Some(val) = val.closed.borrow().as_ref() {
                            val.clone()
                        } else {
                            let val = &self.stack.values[val.location];
                            val.clone()
                        }
                    };

                    self.push(res);
                }
                OpCode::SetLocal(index) => {
                    let addr = self.current_frame().slots + index + 1;
                    let val = self.peek(0).expect("unable to pop value");
                    self.stack.values[addr] = val.clone();
                }
                OpCode::SetUpvalue(index) => {
                    let closure = &self.current_frame().closure.clone();
                    let obj_upvalue = &closure.obj_upvalues[index];
                    let val = self.peek(0).expect("unable to pop value");
                    if obj_upvalue.closed.borrow().is_none() {
                        self.stack.values[obj_upvalue.location] = val.clone();
                    } else {
                        *obj_upvalue.closed.borrow_mut() = Some(val.clone());
                    }
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
                    // We need to subtract 1 from the ip because the ip will be incremented at the
                    // beginning of the loop
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
                OpCode::Closure(v) => {
                    let val = &self.current_chunk().constants[v];
                    if let Value::Function(f) = val {
                        let closure = &**f;
                        let mut closure = ObjClosure::new(closure.clone());
                        for upvalue in &closure.function.upvalues {
                            let obj_upvalue = if upvalue.is_local {
                                let index = self.current_frame().slots + upvalue.index + 1;
                                self.capture_upvalue(index)
                            } else {
                                self.current_frame().closure.obj_upvalues[upvalue.index].clone()
                            };
                            closure.obj_upvalues.push(obj_upvalue)
                        }
                        self.push(Value::Closure(Gc::new(closure)));
                    }
                }
                _ => {
                    println!("Unknown operation code during interpreting!");
                    return Err(InterpretError::RuntimeError);
                }
            }
        }
    }

    // Enable this function to print the stack
    // fn print_stack(&self) {
    //     for value in self.stack.clone() {
    //         println!("[{}]", value);
    //     }
    // }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

fn clock_native(_args: &[Value]) -> Value {
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    Value::Number(since_the_epoch.as_secs_f64())
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
        vm.stack.push(Value::String(Gc::new("hello".to_string())));
        assert_eq!(
            vm.stack.pop(),
            Some(Value::String(Gc::new("hello".to_string())))
        );
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
