use crate::chunk::Chunk;
use crate::op_code::OpCode;
use crate::scanner::Scanner;
use crate::token::{Token, TokenType};
use crate::utils::convert_slice_to_string;
use crate::value::Value;

const MAX_LOCALS: usize = 256;

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

#[derive(Clone, Copy)]
struct Local {
    name: Token,
    depth: i32,
}

pub struct Compiler {
    locals: Vec<Local>,
    local_count: usize,
    scope_depth: i32,
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            locals: vec![
                Local {
                    name: Token {
                        t_type: TokenType::Eof,
                        start: 0,
                        length: 0,
                        line: 0,
                    },
                    depth: 0,
                };
                MAX_LOCALS
            ],
            local_count: 0,
            scope_depth: 0,
        }
    }
}

pub struct Parser<'a> {
    scanner: Scanner<'a>,
    compiler: Compiler,
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
            compiler: Compiler::new(),
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

    fn consume(&mut self, expected_type: TokenType, msg: &str) {
        if self.current.t_type == expected_type {
            self.next_valid_token();
            return;
        }

        self.error_at_current(msg);
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

    fn patch_jump(&mut self, offset: usize) {
        let jump_offset = self.chunk.code.len() - offset - 1;

        if jump_offset > u16::MAX as usize {
            self.error("Too much code to jump over.");
        }

        let new_code = OpCode::Jump(jump_offset as u16);

        self.chunk.code[offset] = new_code;
    }

    fn patch_if_false_jump(&mut self, offset: usize) {
        let jump_offset = self.chunk.code.len() - offset - 1;

        if jump_offset > u16::MAX as usize {
            self.error("Too much code to jump over.");
        }

        let new_code = OpCode::JumpIfFalse(jump_offset as u16);

        self.chunk.code[offset] = new_code;
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if !self.check(token_type) {
            return false;
        }
        self.next_valid_token();
        true
    }

    fn get_rule(&mut self, t: TokenType) -> ParseRule<'a> {
        match t {
            TokenType::LeftParen => ParseRule {
                prefix: Some(Parser::parse_grouping),
                infix: None,
                precedence: Precedence::No,
            },
            TokenType::Minus => ParseRule {
                prefix: Some(Parser::parse_unary),
                infix: Some(Parser::parse_binary),
                precedence: Precedence::Term,
            },
            TokenType::Bang => ParseRule {
                prefix: Some(Parser::parse_unary),
                infix: None,
                precedence: Precedence::Term,
            },
            TokenType::BangEqual | TokenType::EqualEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary),
                precedence: Precedence::Equality,
            },
            TokenType::Plus => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary),
                precedence: Precedence::Term,
            },
            TokenType::Slash | TokenType::Star => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary),
                precedence: Precedence::Factor,
            },
            TokenType::Number => ParseRule {
                prefix: Some(Parser::parse_number),
                infix: None,
                precedence: Precedence::No,
            },
            TokenType::Nil | TokenType::True | TokenType::False => ParseRule {
                prefix: Some(Parser::parse_literal),
                infix: None,
                precedence: Precedence::No,
            },
            TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary),
                precedence: Precedence::Comparison,
            },
            TokenType::Print => ParseRule {
                prefix: Some(Parser::parse_print),
                infix: None,
                precedence: Precedence::No,
            },
            TokenType::Strings => ParseRule {
                prefix: Some(Parser::parse_string),
                infix: None,
                precedence: Precedence::No,
            },
            TokenType::Identifier => ParseRule {
                prefix: Some(Parser::variable),
                infix: None,
                precedence: Precedence::No,
            },
            TokenType::And => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_and),
                precedence: Precedence::And,
            },
            TokenType::Or => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_or),
                precedence: Precedence::Or,
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

    fn parse_number(&mut self, _: bool) {
        let start = self.previous.start;
        let length = self.previous.length;
        let value = convert_slice_to_string(self.scanner.bytes, start, start + length);
        let number = value
            .parse::<f64>()
            .expect("cannot convert target to usize");
        self.emit_constant(Value::Number(number));
    }

    fn parse_grouping(&mut self, _: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression");
    }

    fn parse_unary(&mut self, _: bool) {
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

    fn parse_binary(&mut self, _: bool) {
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

    fn parse_literal(&mut self, _: bool) {
        match self.previous.t_type {
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::True => self.emit_byte(OpCode::True),
            TokenType::Nil => self.emit_byte(OpCode::Nil),
            _ => unreachable!("{:?}", self.previous.t_type),
        }
    }

    fn parse_string(&mut self, _: bool) {
        let start = self.previous.start + 1;
        let length = self.previous.length - 2;
        let value = convert_slice_to_string(self.scanner.bytes, start, start + length);
        self.emit_constant(Value::String(value));
    }

    fn parse_print(&mut self, _: bool) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value");
        self.emit_byte(OpCode::Print);
    }

    fn parse_variable(&mut self, msg: &str) -> usize {
        self.consume(TokenType::Identifier, msg);

        self.declare_variable();
        if self.compiler.scope_depth > 0 {
            return 0;
        }

        self.identifier_constant()
    }

    fn parse_and(&mut self, _: bool) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse(0xff));
        self.emit_byte(OpCode::Pop);
        self.parse_precedence(Precedence::And);
        self.patch_if_false_jump(end_jump);
    }

    fn parse_or(&mut self, _: bool) {
        let end_jump = self.emit_jump(OpCode::Jump(0xff));

        self.emit_byte(OpCode::Pop);

        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn declare_variable(&mut self) {
        if self.compiler.scope_depth == 0 {
            return;
        }

        let name =
            &self.scanner.bytes[self.previous.start..self.previous.start + self.previous.length];
        for i in (0..self.compiler.local_count).rev() {
            let local = self.compiler.locals[i];
            if local.depth != -1 && local.depth < self.compiler.scope_depth {
                break;
            }

            let local_name =
                &self.scanner.bytes[local.name.start..local.name.start + local.name.length];

            if local_name == name {
                self.error("Variable with this name already declared in this scope.");
            }
        }

        self.add_local(self.previous);
    }

    fn add_local(&mut self, name: Token) {
        if self.compiler.local_count == MAX_LOCALS {
            self.error("Too many local variables in function");
            return;
        }
        let mut local = Local { name, depth: -1 };

        std::mem::swap(
            &mut local,
            &mut self.compiler.locals[self.compiler.local_count],
        );
        self.compiler.local_count += 1;
    }

    fn variable(&mut self, can_assign: bool) {
        self.compile_named_variable(self.previous, can_assign);
    }

    fn compile_named_variable(&mut self, name: Token, can_assign: bool) {
        let arg = self.resolve_local(&name);
        match arg {
            Some(index) => {
                if self.match_token(TokenType::Equal) && can_assign {
                    self.expression();
                    self.emit_byte(OpCode::SetLocal(index));
                } else {
                    self.emit_byte(OpCode::GetLocal(index));
                }
            }
            None => {
                let index = self.identifier_constant();
                if self.match_token(TokenType::Equal) && can_assign {
                    self.expression();
                    self.emit_byte(OpCode::SetGlobal(index));
                } else {
                    self.emit_byte(OpCode::GetGlobal(index));
                }
            }
        }
    }

    fn resolve_local(&mut self, name: &Token) -> Option<usize> {
        let token_literal = &self.scanner.bytes[name.start..name.start + name.length];
        for idx in (0..self.compiler.local_count).rev() {
            let local = self.compiler.locals[idx];
            let local_literal =
                &self.scanner.bytes[local.name.start..local.name.start + local.name.length];
            if local_literal == token_literal {
                return Some(idx);
            }
        }
        None
    }

    fn identifier_constant(&mut self) -> usize {
        let identifier = convert_slice_to_string(
            self.scanner.bytes,
            self.previous.start,
            self.previous.start + self.previous.length,
        );

        self.chunk.push_constant(Value::String(identifier))
    }

    fn emit_constant(&mut self, number: Value) {
        let index = self.chunk.push_constant(number);

        self.emit_byte(OpCode::Constant(index));
    }

    fn emit_loop(&mut self, loop_start: u16) {
        let len = u16::try_from(self.chunk.code.len()).expect("Chunk code too large");

        let offset = len - loop_start - 1;
        if offset > 0xff {
            self.error("Loop body too large.");
        }

        self.emit_byte(OpCode::Loop(offset))
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

    fn emit_jump(&mut self, code: OpCode) -> usize {
        self.emit_byte(code);
        self.chunk.code.len() - 1
    }

    fn end_compiler(&mut self) {
        self.emit_return();

        if !self.had_error {
            // self.chunk.disassemble_chunk("code");
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    // expression statement looks for a semicolon and also emits a pop instruction.
    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop);
    }

    fn begin_scope(&mut self) {
        self.compiler.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.compiler.scope_depth -= 1;

        // Clear the local variable inside the scope when the scope ends
        while self.compiler.local_count > 0
            && self.compiler.locals[self.compiler.local_count - 1].depth > self.compiler.scope_depth
        {
            self.emit_byte(OpCode::Pop);
            self.compiler.local_count -= 1;
        }
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn parse_if(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let jump_idx = self.emit_jump(OpCode::JumpIfFalse(0xff));
        self.emit_byte(OpCode::Pop);
        self.statement();

        let else_jump_idx = self.emit_jump(OpCode::Jump(0xff));
        self.patch_if_false_jump(jump_idx);
        self.emit_byte(OpCode::Pop);

        if self.match_token(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump_idx);
    }

    fn parse_while(&mut self) {
        let loop_start = self.chunk.code.len() - 1;
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let jump_idx = self.emit_jump(OpCode::JumpIfFalse(0xff));
        self.emit_byte(OpCode::Pop);
        self.statement();
        self.emit_loop(u16::try_from(loop_start).expect("Chunk code too large"));

        self.patch_if_false_jump(jump_idx);
        self.emit_byte(OpCode::Pop);
    }

    fn parse_for(&mut self) {
        // for loop var should be scoped
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");

        if self.match_token(TokenType::Semicolon) {
            // No initializer
        } else if self.match_token(TokenType::Var) {
            self.define_variable();
        } else {
            self.expression_statement();
        }

        let mut jump_idx = 0;

        // Condition cluase
        let mut loop_start = self.chunk.code.len() - 1;
        if !self.match_token(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");

            jump_idx = self.emit_jump(OpCode::JumpIfFalse(0xff));
            self.emit_byte(OpCode::Pop);
        }

        // Increment clause
        if !self.match_token(TokenType::RightParen) {
            let body_jump_idx = self.emit_jump(OpCode::Jump(0xff));
            let increment_start = self.chunk.code.len() - 1;
            self.expression();
            self.emit_byte(OpCode::Pop);
            self.consume(TokenType::RightParen, "Expect ')' after for clauses.");

            self.emit_loop(u16::try_from(loop_start).expect("Chunk code too large"));
            loop_start = increment_start;
            self.patch_jump(body_jump_idx);
        }
        self.statement();
        self.emit_loop(u16::try_from(loop_start).expect("Chunk code too large"));

        self.patch_if_false_jump(jump_idx);
        self.end_scope();
    }

    fn statement(&mut self) {
        if self.match_token(TokenType::Print) {
            self.parse_print(true);
        } else if self.match_token(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn define_variable(&mut self) {
        let index = self.parse_variable("Expect variable name.");
        if self.match_token(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil);
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );

        if self.compiler.scope_depth > 0 {
            self.mark_initialized();
        } else {
            self.emit_byte(OpCode::DefineGlobal(index));
        }
    }

    fn mark_initialized(&mut self) {
        self.compiler.locals[self.compiler.local_count - 1].depth = self.compiler.scope_depth;
    }

    fn declaration(&mut self) {
        if self.match_token(TokenType::Var) {
            self.define_variable();
        } else if self.match_token(TokenType::If) {
            self.parse_if();
        } else if self.match_token(TokenType::While) {
            self.parse_while();
        } else if self.match_token(TokenType::For) {
            self.parse_for();
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
        // Constant, DefineGlobal,Return
        assert_eq!(3, chunk.code.len());
    }

    #[test]
    fn test_scope() {
        let source = r#"
        {
            var a = 1;
        }
        "#
        .as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert!(parser.compile());
        assert_eq!(1, chunk.constants.len());
        // Constant,Pop,Return
        assert_eq!(3, chunk.code.len());
    }

    #[test]
    fn test_scope_nested() {
        let source = r#"
        {
            var a = 1;
            {
                var a = 2;
                print a;
            }
        }
        "#
        .as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert!(parser.compile());
        assert_eq!(2, chunk.constants.len());
        // Constant, Constant, Print, GetLocal,Pop, Pop, Return
        assert_eq!(7, chunk.code.len());
    }

    #[test]
    fn test_scope_fail() {
        let source = r#"
        {
            var a = 1;
            {
                var a = 2;
        }
        "#
        .as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert!(!parser.compile());
        assert!(parser.had_error);
    }

    #[test]
    fn test_if() {
        let source = r#"
        if (true) {
            print "true";
        }
        "#
        .as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert!(parser.compile());
        assert_eq!(1, chunk.constants.len());
        assert_eq!(8, chunk.code.len());
    }

    #[test]
    fn test_if_else() {
        let source = r#"
        if (true) {
            print "true";
        } else {
            print "false";
        }
        "#
        .as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert!(parser.compile());
        assert_eq!(2, chunk.constants.len());
        assert_eq!(10, chunk.code.len());
    }

    #[test]
    fn test_and() {
        let source = r#"
        if (true and false) {
            print "true";
        } else {
            print "false";
        }
        "#
        .as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert!(parser.compile());
        assert_eq!(2, chunk.constants.len());
        assert_eq!(13, chunk.code.len());
    }

    #[test]
    fn test_or() {
        let source = r#"
        if (true or false) {
            print "true";
        } else {
            print "false";
        }
        "#
        .as_bytes();
        let mut chunk = Chunk::new();
        let mut parser = Parser::new(source, &mut chunk);
        assert!(parser.compile());
        assert_eq!(2, chunk.constants.len());
        assert_eq!(13, chunk.code.len());
    }
}
