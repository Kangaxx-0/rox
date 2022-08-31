use rox::{chunk::Chunk, op_code::OpCode, value::Value, vm::Vm};

fn main() {
    let mut vm = Vm::new();
    vm.initialize();
    let mut chunk = Chunk::new();
    let index = chunk.push_constant(Value::Number(1.2));

    let code_return = OpCode::Return;
    let constant = OpCode::Constant(index as u8);

    chunk.write_to_chunk(constant, 1);
    chunk.write_to_chunk(code_return, 2);

    vm.interpret(chunk).expect("Oops");
}
