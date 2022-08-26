use rox::{chunk, op_code};

#[test]
fn insturction_return() {
    let mut chunk = chunk::Chunk::new();

    let code_return = op_code::OpCode::Return;

    chunk.push_instruction(code_return);
    //FIXME How to assert and test include_instruction?
    assert_eq!(1, chunk.len());
}
