use crate::chunk::Chunk;
use crate::op_code::OpCode;
use crate::scanner::Scanner;
use crate::token::{Token, TokenType};
use crate::utils::convert_slice_to_string;
use crate::value::Value;

//FIXME - remove dead_code
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
// Precedence symbols from low to high:
//  No -> no Precedence
//  Assignment -> =
//  Or -> or
//  And -> and
//  Equality -> == !=
//  Comparison -> < > <= >=
//  Term -> + -
//  Factor -> * /
//  Unary -> ! -
//  Call -> . ()
//  Primary -> literals and grouping
//
enum Precedence {
    No,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    fn next(&self) -> Self {
        match self {
            Precedence::No => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
        }
    }
}

type ParseFn<'a> = fn(&mut Parser<'a>, can_assign: bool) -> ();

struct ParseRule<'a> {
    prefix: Option<ParseFn<'a>>,
    infix: Option<ParseFn<'a>>,
    precedence: Precedence,
}

pub struct Parser<'a> {
    scanner: Scanner<'a>,
    chunk: &'a mut Chunk,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a [u8], chunk: &'a mut Chunk) -> Self {
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

        self.error_at_current(msg);
    }

    fn compile_number(&mut self, _: bool) {
        let start = self.previous.start;
        let length = self.previous.length;
        let value = convert_slice_to_string(self.scanner.bytes, start, start + length);
        let number = value
            .parse::<f64>()
            .expect("cannot convert target to usize");
        self.emit_constant(Value::Number(number));
    }

    fn compile_grouping(&mut self, _: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression");
    }

    fn compile_unary(&mut self, _: bool) {
        let operator_type = self.previous.t_type;

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => {
                self.emit_byte(OpCode::Negative);
            }
            TokenType::Bang => {
                self.emit_byte(OpCode::Not);
            }
            _ => (),
        }
    }

    fn compile_binary(&mut self, _: bool) {
        let operator_type = self.previous.t_type;
        let rule = self.get_rule(operator_type);
        self.parse_precedence(rule.precedence.next());

        match operator_type {
            TokenType::Plus => self.emit_byte(OpCode::Add),
            TokenType::Minus => self.emit_byte(OpCode::Subtract),
            TokenType::Star => self.emit_byte(OpCode::Multiply),
            TokenType::Slash => self.emit_byte(OpCode::Divide),
            TokenType::EqualEqual => self.emit_byte(OpCode::Equal),
            TokenType::BangEqual => self.emit_two_bytes(OpCode::Equal, OpCode::Not),
            TokenType::Greater => self.emit_byte(OpCode::Greater),
            TokenType::GreaterEqual => self.emit_two_bytes(OpCode::Less, OpCode::Not),
            TokenType::Less => self.emit_byte(OpCode::Less),
            TokenType::LessEqual => self.emit_two_bytes(OpCode::Greater, OpCode::Not),
            _ => unreachable!("{:?}", operator_type),
        }
    }

    fn compile_literal(&mut self, _: bool) {
        match self.previous.t_type {
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::True => self.emit_byte(OpCode::True),
            TokenType::Nil => self.emit_byte(OpCode::Nil),
            _ => unreachable!("{:?}", self.previous.t_type),
        }
    }

    fn compile_string(&mut self, _: bool) {
        let start = self.previous.start + 1;
        let length = self.previous.length - 2;
        let value = convert_slice_to_string(self.scanner.bytes, start, start + length);
        self.emit_constant(Value::String(value));
    }

    fn compile_print(&mut self, _: bool) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value");
        self.emit_byte(OpCode::Print);
    }

    fn variable_declare(&mut self, msg: &str) -> usize {
        self.consume(TokenType::Identifier, msg);
        self.identifier_constant()
    }

    fn compile_named_variable(&mut self, can_assign: bool) {
        let index = self.identifier_constant();
        if self.match_token(TokenType::Equal) && can_assign {
            self.expression();
            self.emit_byte(OpCode::SetGlobal(index));
        } else {
            self.emit_byte(OpCode::GetGlobal(index));
        }
    }

    fn identifier_constant(&mut self) -> usize {
        let identifier = convert_slice_to_string(
            self.scanner.bytes,
            self.previous.start,
            self.previous.start + self.previous.length,
        );

        let index = self.chunk.push_constant(Value::String(identifier));

        self.emit_byte(OpCode::Constant(index));

        index
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

    fn emit_two_bytes(&mut self, code1: OpCode, code2: OpCode) {
        self.emit_byte(code1);
        self.emit_byte(code2);
    }

    fn next_valid_token(&mut self) {
        self.previous = self.current;

        loop {
            self.current = self.scanner.scan_token();

            if self.current.t_type == TokenType::Error {
                let start = self.current.start;
                let end = start + self.current.length;
                self.error_at_current(&convert_slice_to_string(self.scanner.bytes, start, end));
            } else {
                break;
            }
        }
    }

    fn error_at_current(&mut self, msg: &str) {
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
            eprint!(" unknown type found.");
        } else {
            eprint!(" at {} {}", token.length, token.start);
        }

        eprint!(" : {}", msg);

        self.had_error = true;
    }
    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }
    fn get_rule(&mut self, t: TokenType) -> ParseRule<'a> {
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
            TokenType::Bang => ParseRule {
                prefix: Some(Parser::compile_unary),
                infix: None,
                precedence: Precedence::Term,
            },
            TokenType::BangEqual | TokenType::EqualEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::compile_binary),
                precedence: Precedence::Equality,
            },
            TokenType::Plus => ParseRule {
                prefix: None,
                infix: Some(Parser::compile_binary),
                precedence: Precedence::Term,
            },
            TokenType::Slash | TokenType::Star => ParseRule {
                prefix: None,
                infix: Some(Parser::compile_binary),
                precedence: Precedence::Factor,
            },
            TokenType::Number => ParseRule {
                prefix: Some(Parser::compile_number),
                infix: None,
                precedence: Precedence::No,
            },
            TokenType::Nil | TokenType::True | TokenType::False => ParseRule {
                prefix: Some(Parser::compile_literal),
                infix: None,
                precedence: Precedence::No,
            },
            TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::compile_binary),
                precedence: Precedence::Comparison,
            },
            TokenType::Print => ParseRule {
                prefix: Some(Parser::compile_print),
                infix: None,
                precedence: Precedence::No,
            },
            TokenType::Strings => ParseRule {
                prefix: Some(Parser::compile_string),
                infix: None,
                precedence: Precedence::No,
            },
            TokenType::Identifier => ParseRule {
                prefix: Some(Parser::compile_named_variable),
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

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.next_valid_token();

        let prefix_rule = match self.get_rule(self.previous.t_type).prefix {
            Some(rule) => rule,
            None => {
                self.error("Expect expression.");
                return;
            }
        };

        let can_assign = precedence <= Precedence::Assignment;
        prefix_rule(self, can_assign);

        while precedence <= self.get_rule(self.current.t_type).precedence {
            self.next_valid_token();
            if let Some(infix_rule) = self.get_rule(self.previous.t_type).infix {
                infix_rule(self, can_assign);
            }
        }

        if can_assign && self.match_token(TokenType::Equal) {
            self.error("Invalid assignment target.");
        }
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.t_type != TokenType::Eof {
            if self.previous.t_type == TokenType::Semicolon {
                return;
            }

            match self.current.t_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => (),
            }

            self.next_valid_token();
        }
    }

    fn check(&mut self, t: TokenType) -> bool {
        self.current.t_type == t
    }

    fn match_token(&mut self, print: TokenType) -> bool {
        if !self.check(print) {
            return false;
        }
        self.next_valid_token();
        true
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop);
    }

    fn statement(&mut self) {
        if self.match_token(TokenType::Print) {
            self.compile_print(true);
        } else {
            self.expression_statement();
        }
    }

    fn variable_declaration(&mut self) {
        let index = self.variable_declare("Expect variable name.");
        if self.match_token(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil);
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );

        self.emit_byte(OpCode::DefineGlobal(index));
    }

    fn declaration(&mut self) {
        if self.match_token(TokenType::Var) {
            self.variable_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    pub fn compile(&mut self) -> bool {
        self.next_valid_token();

        while self.current.t_type != TokenType::Eof {
            self.declaration();
        }

        self.consume(TokenType::Eof, "Expect end of expression.");
        self.end_compiler();
        !self.had_error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precedence_no() {
        let pre = Precedence::No;
        assert_eq!(Precedence::Assignment, pre.next())
    }

    #[test]
    fn test_precedence_term() {
        let pre = Precedence::Term;
        assert_eq!(Precedence::Factor, pre.next())
    }

    #[test]
    fn test_precedence_factor() {
        let pre = Precedence::Factor;
        assert_eq!(Precedence::Unary, pre.next())
    }

    #[test]
    fn test_precedence_unary() {
        let pre = Precedence::Unary;
        assert_eq!(Precedence::Call, pre.next())
    }

    #[test]
    fn test_precedence_call() {
        let pre = Precedence::Call;
        assert_eq!(Precedence::Primary, pre.next())
    }

    #[test]
    fn test_precedence_primary() {
        let pre = Precedence::Primary;
        assert_eq!(Precedence::Primary, pre.next())
    }

    #[test]
    fn test_compile() {
        let source = "1 + 2;".as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert!(parser.compile());
    }

    #[test]
    fn test_compile_negative() {
        let source = "-1;".as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert!(parser.compile());
    }

    #[test]
    fn test_compile_grouping() {
        let source = "(1);".as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert!(parser.compile());
    }

    #[test]
    fn test_compile_grouping_negative() {
        let source = "(-1);".as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert!(parser.compile());
    }

    #[test]
    fn test_compile_grouping_negative_with_plus() {
        let source = "(-1 + 1);".as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert!(parser.compile());
    }

    #[test]
    fn test_compile_grouping_negative_with_plus_and_multi() {
        let source = "(-1 + 1) * 2;".as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert!(parser.compile());
    }

    #[test]
    fn test_compile_string() {
        let source = r#""hello";"#.as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert!(parser.compile());
    }

    #[test]
    fn test_synchonize() {
        let source = r#"1 + &;"#.as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert!(!parser.compile());
        assert!(parser.had_error);
    }

    #[test]
    fn test_global() {
        let source = r#"var a = 1;"#.as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert!(parser.compile());
        assert_eq!(2, chunk.constants.len());
        assert_eq!(4, chunk.code.len());
    }
}
