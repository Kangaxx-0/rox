use std::{env, io, process::exit};

use rox::vm::{InterpretError, Vm};

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
