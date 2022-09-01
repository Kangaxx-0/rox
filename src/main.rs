use rox::{chunk::Chunk, op_code::OpCode, value::Value, vm::Vm};

fn main() {
    let mut vm = Vm::new();
    vm.initialize();
    let mut chunk = Chunk::new();

    // -((1.2 + 3.4) / 5.6)
    {
        let index1 = chunk.push_constant(Value::Number(1.2));
        let constant1 = OpCode::Constant(index1 as u8);
        chunk.write_to_chunk(constant1, 1);

        let index2 = chunk.push_constant(Value::Number(3.4));
        let constant2 = OpCode::Constant(index2 as u8);
        chunk.write_to_chunk(constant2, 1);

        chunk.write_to_chunk(OpCode::Add, 1);

        let index3 = chunk.push_constant(Value::Number(5.6));
        let constant3 = OpCode::Constant(index3 as u8);
        chunk.write_to_chunk(constant3, 1);

        chunk.write_to_chunk(OpCode::Divide, 1);
        chunk.write_to_chunk(OpCode::Negative, 1);

        chunk.write_to_chunk(OpCode::Return, 3);
        vm.interpret(chunk).expect("Oops");
    }
}
