use crate::chunk::Chunk;
use crate::op_code::OpCode;
use crate::scanner::Scanner;
use crate::token::{Token, TokenType};
use crate::utils::convert_arrayslice_to_string;
use crate::value::Value;

//FIXME - remove dead_code
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    No,
    Assignemnt,
    Or,
    And,
    Equality,
    Comparsion,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    fn next(&self) -> Self {
        match self {
            Precedence::No => Precedence::Assignemnt,
            Precedence::Assignemnt => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparsion,
            Precedence::Comparsion => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
        }
    }
}

type ParseFn<'a, 'b> = fn(&mut Parser<'a, 'b>) -> ();

struct ParseRule<'a, 'b> {
    prefix: Option<ParseFn<'a, 'b>>,
    infix: Option<ParseFn<'a, 'b>>,
    precedence: Precedence,
}

pub struct Parser<'a, 'b> {
    scanner: Scanner<'a>,
    chunk: &'b mut Chunk,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
}

impl<'a, 'b> Parser<'a, 'b> {
    pub fn new(source: &'a [u8], chunk: &'b mut Chunk) -> Self {
        Self {
            scanner: Scanner::new(source),
            chunk,
            current: Token {
                t_type: TokenType::Nil,
                start: 0,
                length: 0,
                line: 0,
            },
            previous: Token {
                t_type: TokenType::Nil,
                start: 0,
                length: 0,
                line: 0,
            },
            had_error: false,
            panic_mode: false,
        }
    }

    fn consume(&mut self, expected_type: TokenType, msg: &str) {
        if self.current.t_type == expected_type {
            self.next_valid_token();
            return;
        }

        self.error_at_curent(msg);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.next_valid_token();

        let prefix_rule = match self.get_rule(self.previous.t_type).prefix {
            Some(rule) => rule,
            None => {
                self.error("Expect expression.");
                return;
            }
        };

        prefix_rule(self);

        while precedence <= self.get_rule(self.current.t_type).precedence {
            self.next_valid_token();
            if let Some(infix_rule) = self.get_rule(self.previous.t_type).infix {
                infix_rule(self);
            }
        }
    }

    fn compile_number(&mut self) {
        let start = self.previous.start;
        let length = self.previous.length;
        let value = convert_arrayslice_to_string(self.scanner.bytes, start, start + length);
        let number = value
            .parse::<f64>()
            .expect("cannot convert target to usize");
        self.emit_constant(Value::Number(number));
    }

    fn compile_grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression");
    }

    fn compile_unary(&mut self) {
        let operator_type = self.previous.t_type;

        self.parse_precedence(Precedence::Unary);

        if let TokenType::Minus = operator_type {
            self.emit_byte(OpCode::Negative);
        }
    }

    fn compile_binary(&mut self) {
        let operator_type = self.previous.t_type;
        let rule = self.get_rule(operator_type);
        self.parse_precedence(rule.precedence.next());

        match operator_type {
            TokenType::Plus => self.emit_byte(OpCode::Add),
            TokenType::Minus => self.emit_byte(OpCode::Subtract),
            TokenType::Star => self.emit_byte(OpCode::Multiply),
            TokenType::Slash => self.emit_byte(OpCode::Divide),
            _ => unreachable!("{:?}", operator_type),
        }
    }

    fn emit_constant(&mut self, number: Value) {
        let index = self.chunk.push_constant(number);

        self.emit_byte(OpCode::Constant(index));
    }

    fn end_compiler(&mut self) {
        self.emit_return();

        if !self.had_error {
            self.chunk.disassemble_chunk("code");
        }
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return);
    }
    fn emit_byte(&mut self, code: OpCode) {
        self.chunk.write_to_chunk(code, self.previous.line);
    }

    fn next_valid_token(&mut self) {
        self.previous = self.current;

        loop {
            self.current = self.scanner.scan_token();

            if self.current.t_type == TokenType::Error {
                let start = self.current.start;
                let end = start + self.current.length;
                self.error_at_curent(&convert_arrayslice_to_string(
                    self.scanner.bytes,
                    start,
                    end,
                ));
            } else {
                break;
            }
        }
    }

    fn error_at_curent(&mut self, msg: &str) {
        self.error_at(self.current, msg);
    }

    fn error(&mut self, msg: &str) {
        self.error_at(self.previous, msg)
    }

    fn error_at(&mut self, token: Token, msg: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        eprint!("[line {}] error", token.line);
        if token.t_type == TokenType::Eof {
            eprint!(" at end");
        } else if token.t_type == TokenType::Error {
        } else {
            eprint!(" at {} {}", token.length, token.start);
        }

        eprint!("{}", msg);

        self.had_error = true;
    }
    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignemnt);
    }
    fn get_rule(&mut self, t: TokenType) -> ParseRule<'a, 'b> {
        match t {
            TokenType::LeftParen => ParseRule {
                prefix: Some(Parser::compile_grouping),
                infix: None,
                precedence: Precedence::No,
            },
            TokenType::Minus => ParseRule {
                prefix: Some(Parser::compile_unary),
                infix: Some(Parser::compile_binary),
                precedence: Precedence::Term,
            },
            TokenType::Plus => ParseRule {
                prefix: None,
                infix: Some(Parser::compile_binary),
                precedence: Precedence::Term,
            },
            TokenType::Slash => ParseRule {
                prefix: None,
                infix: Some(Parser::compile_binary),
                precedence: Precedence::Factor,
            },
            TokenType::Star => ParseRule {
                prefix: None,
                infix: Some(Parser::compile_binary),
                precedence: Precedence::Factor,
            },
            TokenType::Number => ParseRule {
                prefix: Some(Parser::compile_number),
                infix: None,
                precedence: Precedence::No,
            },
            _ => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::No,
            },
        }
    }
    pub fn compile(&mut self) -> bool {
        self.next_valid_token();
        self.expression();

        self.consume(TokenType::Eof, "Expect end of expression.");
        self.end_compiler();
        !self.had_error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precendence_no() {
        let pre = Precedence::No;
        assert_eq!(Precedence::Assignemnt, pre.next())
    }

    #[test]
    fn test_precendence_term() {
        let pre = Precedence::Term;
        assert_eq!(Precedence::Factor, pre.next())
    }

    #[test]
    fn test_precendence_factor() {
        let pre = Precedence::Factor;
        assert_eq!(Precedence::Unary, pre.next())
    }

    #[test]
    fn test_precendence_unary() {
        let pre = Precedence::Unary;
        assert_eq!(Precedence::Call, pre.next())
    }

    #[test]
    fn test_precendence_call() {
        let pre = Precedence::Call;
        assert_eq!(Precedence::Primary, pre.next())
    }

    #[test]
    fn test_precendence_primary() {
        let pre = Precedence::Primary;
        assert_eq!(Precedence::Primary, pre.next())
    }

    #[test]
    fn test_compile() {
        let source = "1 + 2".as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert_eq!(true, parser.compile());
    }

    #[test]
    fn test_compile_negative() {
        let source = "-1".as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert_eq!(true, parser.compile());
    }

    #[test]
    fn test_compile_grouping() {
        let source = "(1)".as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert_eq!(true, parser.compile());
    }

    #[test]
    fn test_compile_grouping_negative() {
        let source = "(-1)".as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert_eq!(true, parser.compile());
    }

    #[test]
    fn test_compile_grouping_negative_with_plus() {
        let source = "(-1 + 1)".as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert_eq!(true, parser.compile());
    }

    #[test]
    fn test_compile_grouping_negative_with_plus_and_mult() {
        let source = "(-1 + 1) * 2".as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert_eq!(true, parser.compile());
    }
}
