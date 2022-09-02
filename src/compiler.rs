use crate::scanner::Scanner;
use crate::token::TokenType;

pub fn compile(source: Vec<u8>) {
    let mut scanner = Scanner::new(&source);

    let mut line = usize::MAX;
    loop {
        let tokens = scanner.scan_token();

        if tokens.line != line {
            println!("{}", tokens.line);
            line = tokens.line;
        } else {
            println!("|");
        }

        println!("{:?} {} {}", tokens.t_type, tokens.length, tokens.line);

        if let TokenType::Eof = tokens.t_type {
            break;
        }
    }
}
