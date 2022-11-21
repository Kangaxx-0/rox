use rox_gc::Gc;

use crate::chunk::Chunk;
use crate::objects::{ObjFunction, UpValue, MAX_UPVALUES};
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
    is_captured: bool, // This field is `true` if the local is captured by any later closure.
}

#[derive(PartialEq, Eq)]
enum FunctionType {
    Function,
    Script,
}

// compiler here is a chunk, each function's coding living in separate chunk.
pub struct Compiler {
    locals: Vec<Local>,
    local_count: usize,
    scope_depth: i32,
    function: ObjFunction,
    function_type: FunctionType,
    // each compiler points to the enclosing compiler
    enclosing: Option<Box<Compiler>>,
}

impl Compiler {
    fn new(name: String, types: FunctionType) -> Self {
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
                    is_captured: false,
                };
                MAX_LOCALS
            ],
            local_count: 0,
            scope_depth: 0,
            function: ObjFunction::new(name),
            function_type: types,
            enclosing: None,
        }
    }

    fn resolve_local(&mut self, bytes: &[u8], name: &Token) -> Option<usize> {
        let token_literal = &bytes[name.start..name.start + name.length];
        for idx in (0..self.local_count).rev() {
            let local = self.locals[idx];
            let local_literal = &bytes[local.name.start..local.name.start + local.name.length];
            if local_literal == token_literal {
                return Some(idx);
            }
        }
        None
    }

    fn resolve_upvalue(&mut self, bytes: &[u8], name: &Token) -> Option<usize> {
        // First, we look for a matching local variable in the current enclosing function.
        // If we find one, we capture and return the index of local variable in the enclosing function.
        if let Some(enclosing) = self.enclosing.as_mut() {
            if let Some(index) = enclosing.resolve_local(bytes, name) {
                // When resolving an identifier, if we end up creating a new upvalue for a local
                // var, we mark it as captured.
                enclosing.locals[index].is_captured = true;
                return Some(self.add_upvalue(index, true));
            }
            // Otherwise, we look for a local variable beyond the immediate enclosing function recursively.
            // When a local variable is found, the most deeply nested call to resolve_upvalue captures it
            // and returns the index.

            if let Some(index) = enclosing.resolve_upvalue(bytes, name) {
                return Some(self.add_upvalue(index, false));
            }
        }
        None
    }

    fn add_upvalue(&mut self, index: usize, is_local: bool) -> usize {
        let count = self.function.upvalues.len();
        for value in self.function.upvalues.iter() {
            if value.index == index && value.is_local == is_local {
                return value.index;
            }
        }

        if count == MAX_UPVALUES {
            // TODO -  propagate error back to the parser
            panic!("Too many closure variables in function.");
        }

        self.function.upvalues.push(UpValue { index, is_local });
        count
    }
}

pub struct Parser<'a> {
    scanner: Scanner<'a>,
    compiler: Compiler,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self {
            scanner: Scanner::new(source),
            compiler: Compiler::new(String::from("script"), FunctionType::Script),
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

    // The current function chunk is always the chunk owned by the function we're in the middle of compiling.
    fn current_function_chunk(&self) -> &Chunk {
        &self.compiler.function.chunk
    }

    fn current_function_chunk_mut(&mut self) -> &mut Chunk {
        &mut self.compiler.function.chunk
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
        let jump_offset = self.current_function_chunk().code.len() - offset - 1;

        if jump_offset > u16::MAX as usize {
            self.error("Too much code to jump over.");
        }

        let new_code = OpCode::Jump(jump_offset as u16);

        self.current_function_chunk_mut().code[offset] = new_code;
    }

    fn patch_if_false_jump(&mut self, offset: usize) {
        let jump_offset = self.current_function_chunk().code.len() - offset - 1;

        if jump_offset > u16::MAX as usize {
            self.error("Too much code to jump over.");
        }

        let new_code = OpCode::JumpIfFalse(jump_offset as u16);

        self.current_function_chunk_mut().code[offset] = new_code;
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
                prefix: Some(Parser::grouping),
                infix: Some(Parser::call),
                precedence: Precedence::Call,
            },
            TokenType::Minus => ParseRule {
                prefix: Some(Parser::unary),
                infix: Some(Parser::binary),
                precedence: Precedence::Term,
            },
            TokenType::Bang => ParseRule {
                prefix: Some(Parser::unary),
                infix: None,
                precedence: Precedence::Term,
            },
            TokenType::BangEqual | TokenType::EqualEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Equality,
            },
            TokenType::Plus => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Term,
            },
            TokenType::Slash | TokenType::Star => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Factor,
            },
            TokenType::Number => ParseRule {
                prefix: Some(Parser::number),
                infix: None,
                precedence: Precedence::No,
            },
            TokenType::Nil | TokenType::True | TokenType::False => ParseRule {
                prefix: Some(Parser::literal),
                infix: None,
                precedence: Precedence::No,
            },
            TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::Print => ParseRule {
                prefix: Some(Parser::print),
                infix: None,
                precedence: Precedence::No,
            },
            TokenType::Strings => ParseRule {
                prefix: Some(Parser::string),
                infix: None,
                precedence: Precedence::No,
            },
            TokenType::Identifier => ParseRule {
                prefix: Some(Parser::parse_variable),
                infix: None,
                precedence: Precedence::No,
            },
            TokenType::And => ParseRule {
                prefix: None,
                infix: Some(Parser::and),
                precedence: Precedence::And,
            },
            TokenType::Or => ParseRule {
                prefix: None,
                infix: Some(Parser::or),
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

    fn number(&mut self, _: bool) {
        let start = self.previous.start;
        let length = self.previous.length;
        let value = convert_slice_to_string(self.scanner.bytes, start, start + length);
        let number = value
            .parse::<f64>()
            .expect("cannot convert target to usize");
        self.emit_constant(Value::Number(number));
    }

    fn grouping(&mut self, _: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression");
    }

    fn unary(&mut self, _: bool) {
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

    fn binary(&mut self, _: bool) {
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

    fn literal(&mut self, _: bool) {
        match self.previous.t_type {
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::True => self.emit_byte(OpCode::True),
            TokenType::Nil => self.emit_byte(OpCode::Nil),
            _ => unreachable!("{:?}", self.previous.t_type),
        }
    }

    fn string(&mut self, _: bool) {
        let start = self.previous.start + 1;
        let length = self.previous.length - 2;
        let value = convert_slice_to_string(self.scanner.bytes, start, start + length);
        self.emit_constant(Value::String(Gc::new(value)));
    }

    fn print(&mut self, _: bool) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value");
        self.emit_byte(OpCode::Print);
    }

    fn variable(&mut self, msg: &str) -> usize {
        self.consume(TokenType::Identifier, msg);

        self.declare_variable();
        if self.compiler.scope_depth > 0 {
            return 0;
        }

        self.identifier_constant()
    }

    fn and(&mut self, _: bool) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse(0xff));
        self.emit_byte(OpCode::Pop);
        self.parse_precedence(Precedence::And);
        self.patch_if_false_jump(end_jump);
    }

    fn or(&mut self, _: bool) {
        let end_jump = self.emit_jump(OpCode::Jump(0xff));

        self.emit_byte(OpCode::Pop);

        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn call(&mut self, _: bool) {
        let arg_count = self.argument_list();
        self.emit_byte(OpCode::Call(arg_count));
    }

    fn argument_list(&mut self) -> usize {
        let mut arg_count = 0;
        if !self.check(TokenType::RightParen) {
            loop {
                self.expression();
                if arg_count == 255 {
                    self.error("Cannot have more than 255 arguments.");
                }
                arg_count += 1;

                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after arguments.");
        arg_count
    }

    fn define_variable(&mut self, global: usize) {
        if self.compiler.scope_depth > 0 {
            self.mark_initialized();
            return;
        }

        self.emit_byte(OpCode::DefineGlobal(global));
    }

    fn declare_variable(&mut self) {
        // Global variables are implicitly declared.
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
        let mut local = Local {
            name,
            depth: -1,
            is_captured: false,
        };

        std::mem::swap(
            &mut local,
            &mut self.compiler.locals[self.compiler.local_count],
        );
        self.compiler.local_count += 1;
    }

    fn parse_variable(&mut self, can_assign: bool) {
        self.compile_named_variable(self.previous, can_assign);
    }

    fn compile_named_variable(&mut self, name: Token, can_assign: bool) {
        let arg = self.compiler.resolve_local(self.scanner.bytes, &name);
        // Compiler walks the block scopes for the current function from innermost to outermost. If
        // it does not find the variable in the current scope, it looks for a local variable in any
        // of the surrounding functions
        match arg {
            Some(index) => {
                if self.match_token(TokenType::Equal) && can_assign {
                    self.expression();

                    self.emit_byte(OpCode::SetLocal(index));
                } else {
                    self.emit_byte(OpCode::GetLocal(index));
                }
            }
            None => match self.compiler.resolve_upvalue(self.scanner.bytes, &name) {
                Some(index) => {
                    if self.match_token(TokenType::Equal) && can_assign {
                        self.expression();
                        self.emit_byte(OpCode::SetUpvalue(index));
                    } else {
                        self.emit_byte(OpCode::GetUpvalue(index));
                    }
                }
                None => {
                    let global = self.identifier_constant();
                    if self.match_token(TokenType::Equal) && can_assign {
                        self.expression();
                        self.emit_byte(OpCode::SetGlobal(global));
                    } else {
                        self.emit_byte(OpCode::GetGlobal(global));
                    }
                }
            },
        }
    }

    fn identifier_constant(&mut self) -> usize {
        let identifier = convert_slice_to_string(
            self.scanner.bytes,
            self.previous.start,
            self.previous.start + self.previous.length,
        );

        self.compiler
            .function
            .chunk
            .push_constant(Value::String(Gc::new(identifier)))
    }

    fn emit_constant(&mut self, number: Value) {
        let index = self.current_function_chunk_mut().push_constant(number);

        self.emit_byte(OpCode::Constant(index));
    }

    fn emit_closure(&mut self, value: Value) {
        let index = self.current_function_chunk_mut().push_constant(value);
        self.emit_byte(OpCode::Closure(index));
    }

    fn emit_loop(&mut self, loop_start: u16) {
        let len =
            u16::try_from(self.current_function_chunk().code.len()).expect("Chunk code too large");

        let offset = len - loop_start - 1;
        if offset > 0xff {
            self.error("Loop body too large.");
        }

        self.emit_byte(OpCode::Loop(offset))
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Nil);
        self.emit_byte(OpCode::Return);
    }

    fn emit_byte(&mut self, code: OpCode) {
        self.compiler
            .function
            .chunk
            .write_to_chunk(code, self.previous.line);
    }

    fn emit_two_bytes(&mut self, code1: OpCode, code2: OpCode) {
        self.emit_byte(code1);
        self.emit_byte(code2);
    }

    fn emit_jump(&mut self, code: OpCode) -> usize {
        self.emit_byte(code);
        self.current_function_chunk().code.len() - 1
    }

    fn end_compiler(mut self) -> Result<ObjFunction, String> {
        self.emit_return();

        if !self.had_error {
            self.compiler
                .function
                .chunk
                .disassemble_chunk(&self.compiler.function.name.value);
            Ok(self.compiler.function)
        } else {
            Err("Compile error".to_string())
        }
    }

    fn mark_initialized(&mut self) {
        if self.compiler.scope_depth == 0 {
            return;
        }
        self.compiler.locals[self.compiler.local_count - 1].depth = self.compiler.scope_depth;
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
            if self.compiler.locals[self.compiler.local_count - 1].is_captured {
                self.emit_byte(OpCode::CloseUpvalue);
            } else {
                self.emit_byte(OpCode::Pop);
            }
            self.compiler.local_count -= 1;
        }
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    // To handle compiling multiple functions nested within each other, we create a separate
    // compiler for each function being compiled. This compiler is then pushed onto a stack
    fn function(&mut self, kind: FunctionType) {
        let compiler = Compiler::new(
            convert_slice_to_string(
                self.scanner.bytes,
                self.previous.start,
                self.previous.start + self.previous.length,
            ),
            kind,
        );
        let old_cc = std::mem::replace(&mut self.compiler, compiler);
        // set the enclosing function which is also known as the parent function
        self.compiler.enclosing = Some(Box::new(old_cc));
        self.begin_scope();

        self.consume(TokenType::LeftParen, "Expect '(' after function name.");
        if !self.check(TokenType::RightParen) {
            loop {
                self.compiler.function.arity += 1;
                if self.compiler.function.arity == u8::MAX {
                    self.error_at_current("Cannot have more than 255 parameters.");
                }
                let index = self.variable("Expect parameter name.");
                self.define_variable(index);
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after function name.");
        self.consume(TokenType::LeftBrace, "Expect '{' before function body.");
        self.block();

        self.emit_return();

        if let Some(new_cc) = self.compiler.enclosing.take() {
            let function = std::mem::replace(&mut self.compiler, *new_cc).function;
            self.emit_closure(Value::Function(Gc::new(function)));
        }
    }

    fn var_statement(&mut self) {
        let index = self.variable("Expect variable name.");
        if self.match_token(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil);
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );

        self.define_variable(index);
    }

    fn if_statement(&mut self) {
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

    fn while_statement(&mut self) {
        let loop_start = self.current_function_chunk().code.len() - 1;
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

    fn for_statement(&mut self) {
        // for loop var should be scoped
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");

        if self.match_token(TokenType::Semicolon) {
            // No initializer
        } else if self.match_token(TokenType::Var) {
            self.var_statement();
        } else {
            self.expression_statement();
        }

        let mut jump_idx = 0;

        // Condition clause
        let mut loop_start = self.current_function_chunk().code.len() - 1;
        if !self.match_token(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");

            jump_idx = self.emit_jump(OpCode::JumpIfFalse(0xff));
            self.emit_byte(OpCode::Pop);
        }

        // Increment clause
        if !self.match_token(TokenType::RightParen) {
            let body_jump_idx = self.emit_jump(OpCode::Jump(0xff));
            let increment_start = self.current_function_chunk().code.len() - 1;
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

    fn fun_statement(&mut self, kind: FunctionType) {
        let index = self.variable("Expect function name.");
        self.mark_initialized();
        self.function(kind);
        self.define_variable(index);
    }

    fn return_statement(&mut self) {
        if self.compiler.function_type == FunctionType::Script {
            self.error_at_current("Cannot return a value from an initializer.");
        }
        if self.match_token(TokenType::Semicolon) {
            self.emit_return();
        } else {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after return value.");
            self.emit_byte(OpCode::Return);
        }
    }

    fn statement(&mut self) {
        if self.match_token(TokenType::Print) {
            self.print(true);
        } else if self.match_token(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn declaration(&mut self) {
        if self.match_token(TokenType::Var) {
            self.var_statement();
        } else if self.match_token(TokenType::If) {
            self.if_statement();
        } else if self.match_token(TokenType::While) {
            self.while_statement();
        } else if self.match_token(TokenType::For) {
            self.for_statement();
        } else if self.match_token(TokenType::Fun) {
            self.fun_statement(FunctionType::Function);
        } else if self.match_token(TokenType::Return) {
            self.return_statement();
        } else {
            self.statement();
        }
        if self.panic_mode {
            self.synchronize();
        }
    }

    pub fn compile(mut self) -> Result<ObjFunction, String> {
        self.next_valid_token();

        while self.current.t_type != TokenType::Eof {
            self.declaration();
        }

        self.consume(TokenType::Eof, "Expect end of expression.");
        self.end_compiler()
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
        let parser = Parser::new(source);
        assert!(parser.compile().is_ok());
    }

    #[test]
    fn test_compile_negative() {
        let source = "-1;".as_bytes();
        let parser = Parser::new(source);
        assert!(parser.compile().is_ok());
    }

    #[test]
    fn test_compile_grouping() {
        let source = "(1);".as_bytes();
        let parser = Parser::new(source);
        assert!(parser.compile().is_ok());
    }

    #[test]
    fn test_compile_grouping_negative() {
        let source = "(-1);".as_bytes();
        let parser = Parser::new(source);
        assert!(parser.compile().is_ok());
    }

    #[test]
    fn test_compile_grouping_negative_with_plus() {
        let source = "(-1 + 1);".as_bytes();
        let parser = Parser::new(source);
        assert!(parser.compile().is_ok());
    }

    #[test]
    fn test_compile_grouping_negative_with_plus_and_multi() {
        let source = "(-1 + 1) * 2;".as_bytes();
        let parser = Parser::new(source);
        assert!(parser.compile().is_ok());
    }

    #[test]
    fn test_compile_string() {
        let source = r#""hello";"#.as_bytes();
        let parser = Parser::new(source);
        assert!(parser.compile().is_ok());
    }

    #[test]
    fn test_synchonize() {
        let source = r#"1 + &;"#.as_bytes();
        let parser = Parser::new(source);
        assert!(!parser.compile().is_ok());
    }

    #[test]
    fn test_global() {
        let source = r#"var a = 1;"#.as_bytes();
        let parser = Parser::new(source);
        let obj = parser.compile();
        assert!(obj.is_ok());
        assert_eq!(2, obj.as_ref().unwrap().chunk.constants.len());
        // Constant, DefineGlobal,Nil,Return
        assert_eq!(4, obj.as_ref().unwrap().chunk.code.len());
    }

    #[test]
    fn test_scope() {
        let source = r#"
        {
            var a = 1;
        }
        "#
        .as_bytes();
        let parser = Parser::new(source);
        let obj = parser.compile();
        assert!(obj.is_ok());
        assert_eq!(1, obj.as_ref().unwrap().chunk.constants.len());
        // Constant,Pop,Nil,Return
        assert_eq!(4, obj.as_ref().unwrap().chunk.code.len());
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
        let parser = Parser::new(source);
        let obj = parser.compile();
        assert!(obj.is_ok());
        assert_eq!(2, obj.as_ref().unwrap().chunk.constants.len());
        // Constant, Constant, Print, GetLocal,Pop, Pop, Nil,Return
        assert_eq!(8, obj.as_ref().unwrap().chunk.code.len());
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
        let parser = Parser::new(source);
        assert!(!parser.compile().is_ok());
    }

    #[test]
    fn test_if() {
        let source = r#"
        if (true) {
            print "true";
        }
        "#
        .as_bytes();
        let parser = Parser::new(source);
        let obj = parser.compile();
        assert!(obj.is_ok());
        assert_eq!(1, obj.as_ref().unwrap().chunk.constants.len());
        assert_eq!(9, obj.as_ref().unwrap().chunk.code.len());
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
        let parser = Parser::new(source);
        let obj = parser.compile();
        assert!(obj.is_ok());
        assert_eq!(2, obj.as_ref().unwrap().chunk.constants.len());
        assert_eq!(11, obj.as_ref().unwrap().chunk.code.len());
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
        let parser = Parser::new(source);
        let obj = parser.compile();
        assert!(obj.is_ok());
        assert_eq!(2, obj.as_ref().unwrap().chunk.constants.len());
        assert_eq!(14, obj.as_ref().unwrap().chunk.code.len());
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
        let parser = Parser::new(source);
        let obj = parser.compile();
        assert!(obj.is_ok());
        assert_eq!(2, obj.as_ref().unwrap().chunk.constants.len());
        assert_eq!(14, obj.as_ref().unwrap().chunk.code.len());
    }
}
