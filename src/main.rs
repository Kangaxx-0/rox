use rox::{chunk::Chunk, op_code::OpCode, value::Value};

fn main() {
    let mut chunk = Chunk::new();
    let index = chunk.push_constant(Value::Number(1.2));

    let code_return = OpCode::Return;
    let constant = OpCode::Constant(index as u8);

    chunk.write_to_chunk(constant, Value::Number(1.2), 1);
    chunk.write_to_chunk(code_return, Value::Number(1.2), 2);
    println!("chunk contains {}", chunk.len());
    chunk.disassemble_chunk("test");
}
