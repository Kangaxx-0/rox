use rox::{chunk, op_code};

fn main() {
    let mut chunk = chunk::Chunk::new();

    let code_return = op_code::OpCode::Return;
    let call = op_code::OpCode::Call(20);
    let constant = op_code::OpCode::Constant(1);

    chunk.push_instruction(constant);
    chunk.push_instruction(code_return);
    chunk.push_instruction(call);
    println!("chunk contains {}", chunk.len());
    chunk.disassemble_chunk("test");
}
