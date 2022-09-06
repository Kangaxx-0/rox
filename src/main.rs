use std::{env, io, process::exit};

use rox::{
    chunk::Chunk,
    op_code::OpCode,
    value::Value,
    vm::{InterpretError, Vm},
};

fn main() {
    let mut vm = Vm::new();
    vm.initialize();

    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => repl(&mut vm),
        2 => run_file(&mut vm, args[1].clone()),
        _ => {
            println!("rox can not recognize arguments");
            exit(64)
        }
    }

    let mut chunk = Chunk::new();

    // -((1.2 + 3.4) / 5.6)
    {
        let index1 = chunk.push_constant(Value::Number(1.2));
        let constant1 = OpCode::Constant(index1);
        chunk.write_to_chunk(constant1, 1);

        let index2 = chunk.push_constant(Value::Number(3.4));
        let constant2 = OpCode::Constant(index2);
        chunk.write_to_chunk(constant2, 1);

        chunk.write_to_chunk(OpCode::Add, 1);

        let index3 = chunk.push_constant(Value::Number(5.6));
        let constant3 = OpCode::Constant(index3);
        chunk.write_to_chunk(constant3, 1);

        chunk.write_to_chunk(OpCode::Divide, 1);
        chunk.write_to_chunk(OpCode::Negative, 1);

        chunk.write_to_chunk(OpCode::Return, 3);
    }
}

fn repl(vm: &mut Vm) {
    loop {
        let mut input = String::new();
        if let Err(e) = io::stdin().read_line(&mut input) {
            print!("{}", e);
            exit(74)
        }
        if input.is_empty() {
            break;
        }
        let input_bytes = input.into_bytes();

        match vm.interpret(&input_bytes) {
            Ok(_) => exit(0),
            Err(error) => match error {
                InterpretError::Default => exit(2),
                InterpretError::RuntimeError => exit(70),
                InterpretError::CompileError => exit(65),
            },
        }
    }
}

fn run_file(vm: &mut Vm, file_name: String) {
    let content = std::fs::read(file_name).expect("Could not read file");
    match vm.interpret(&content) {
        Ok(_) => exit(0),
        Err(error) => match error {
            InterpretError::Default => exit(2),
            InterpretError::RuntimeError => exit(70),
            InterpretError::CompileError => exit(65),
        },
    }
}
