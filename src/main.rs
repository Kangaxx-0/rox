use std::{
    env,
    io::{self, Write},
    process::exit,
};

use rox::vm::{InterpretError, Vm};

fn main() {
    let mut vm = Vm::new();
    vm.initialize();

    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => repl(&mut vm),
        2 => run_file(&mut vm, &args[1]),
        _ => {
            println!("rox can not recognize arguments");
            exit(64)
        }
    }
}

fn repl(vm: &mut Vm) {
    loop {
        print!("> ");
        io::stdout().flush().expect("Can't flush stdout");
        let mut input = String::new();
        if let Err(e) = io::stdin().read_line(&mut input) {
            print!("{}", e);
            exit(74)
        }
        if input.is_empty() {
            break;
        }

        if let Err(e) = vm.interpret(&input) {
            match e {
                InterpretError::Default => exit(2),
                InterpretError::RuntimeError => exit(70),
                InterpretError::CompileError => exit(65),
            }
        }
    }
}

fn run_file(vm: &mut Vm, file_name: &str) {
    let content = std::fs::read(file_name).expect("Could not read file");
    let input = String::from_utf8(content).expect("Could not convert file to string");
    match vm.interpret(&input) {
        Ok(_) => exit(0),
        Err(error) => match error {
            InterpretError::Default => exit(2),
            InterpretError::RuntimeError => exit(70),
            InterpretError::CompileError => exit(65),
        },
    }
}
