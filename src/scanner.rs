use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::token::{Token, TokenType};

lazy_static! {
    static ref KEYWORDS: HashMap<String, TokenType> = {
        let mut m = HashMap::new();
        m.insert(String::from("and"), TokenType::And);
        m.insert(String::from("class"), TokenType::Class);
        m.insert(String::from("else"), TokenType::Else);
        m.insert(String::from("false"), TokenType::False);
        m.insert(String::from("for"), TokenType::For);
        m.insert(String::from("fun"), TokenType::Fun);
        m.insert(String::from("if"), TokenType::If);
        m.insert(String::from("nil"), TokenType::Nil);
        m.insert(String::from("or"), TokenType::Or);
        m.insert(String::from("print"), TokenType::Print);
        m.insert(String::from("return"), TokenType::Return);
        m.insert(String::from("super"), TokenType::Super);
        m.insert(String::from("this"), TokenType::This);
        m.insert(String::from("true"), TokenType::True);
        m.insert(String::from("var"), TokenType::Var);
        m.insert(String::from("while"), TokenType::While);
        m
    };
}

pub struct Scanner<'bytes> {
    bytes: &'bytes [u8],
    start: usize,
    current: usize,
    line: usize,
}

impl<'bytes> Scanner<'bytes> {
    pub fn new(source: &'bytes [u8]) -> Self {
        Self {
            bytes: source,
            start: 0,
            current: 0,
            line: 1,
        }
    }
    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.next();

        match c {
            b'(' => self.make_token(TokenType::LeftParen),
            b')' => self.make_token(TokenType::RightParen),
            b'{' => self.make_token(TokenType::LeftBrace),
            b'}' => self.make_token(TokenType::RightBrace),
            b',' => self.make_token(TokenType::Comma),
            b'.' => self.make_token(TokenType::Dot),
            b'-' => self.make_token(TokenType::Minus),
            b'+' => self.make_token(TokenType::Plus),
            b';' => self.make_token(TokenType::Semicolon),
            b'*' => self.make_token(TokenType::Star),
            b'/' => self.make_token(TokenType::Slash),
            b'!' => {
                if let true = self.match_type(b'=') {
                    self.make_token(TokenType::BangEqual)
                } else {
                    self.make_token(TokenType::Bang)
                }
            }
            b'=' => {
                if let true = self.match_type(b'=') {
                    self.make_token(TokenType::EqualEqual)
                } else {
                    self.make_token(TokenType::Equal)
                }
            }
            b'<' => {
                if let true = self.match_type(b'=') {
                    self.make_token(TokenType::LessEqual)
                } else {
                    self.make_token(TokenType::Less)
                }
            }
            b'>' => {
                if let true = self.match_type(b'=') {
                    self.make_token(TokenType::GreaterEqual)
                } else {
                    self.make_token(TokenType::Greater)
                }
            }
            b'"' => self.string(),
            c if is_digit(c) => self.number(),
            c if is_alphabet(c) => self.identifier(),
            _ => self.make_token(TokenType::Error),
        }
    }

    fn make_token(&self, t_type: TokenType) -> Token {
        Token {
            t_type,
            start: self.start,
            length: self.current - self.start,
            line: self.line,
        }
    }

    fn error_token(&self, message: &str) -> Token {
        Token {
            t_type: TokenType::Error,
            start: self.start,
            length: message.len(),
            line: self.line,
        }
    }

    fn next(&mut self) -> u8 {
        self.current += 1;
        self.bytes[self.current - 1]
    }

    fn match_type(&mut self, expected: u8) -> bool {
        if self.is_end() || self.peek() != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }
    fn is_end(&self) -> bool {
        self.peek() != b'\0'
    }

    fn skip_whitespace(&mut self) {
        while !self.is_end() {
            match self.peek() {
                b' ' | b'\r' | b'\t' => {
                    self.next();
                }
                b'\n' => {
                    self.next();
                    self.line += 1;
                }
                b'/' => {
                    if self.peek_next() == b'/' {
                        while self.peek() != b'\n' || self.is_end() {
                            self.next();
                        }
                    }
                }
                _ => return,
            }
        }
    }

    fn number(&mut self) -> Token {
        while is_digit(self.peek()) {
            self.next();
        }

        if self.peek() == b'.' && is_digit(self.peek_next()) {
            self.next();
            while is_digit(self.peek()) {
                self.next();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn identifier(&mut self) -> Token {
        while is_alphabet(self.peek()) || is_digit(self.peek()) {
            self.next();
        }

        let identifier = self
            .bytes
            .get(self.start..self.current)
            .expect("cannot find the expected index byte");

        let key = String::from_utf8(identifier.to_vec()).expect("cannot get string from bytes");
        match KEYWORDS.get(&key) {
            Some(t) => self.make_token(*t),
            None => self.make_token(TokenType::Error),
        }
    }

    fn string(&mut self) -> Token {
        while self.peek() != b'"' && self.is_end() {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            self.next();
        }

        if self.is_end() {
            return self.error_token("Unterminated string");
        }

        // Locate the closing quote.
        self.next();
        self.make_token(TokenType::Strings)
    }

    fn peek_next(&self) -> u8 {
        if self.is_end() || self.current + 1 >= self.bytes.len() {
            b'\0'
        } else {
            self.bytes[self.current + 1]
        }
    }

    fn peek(&self) -> u8 {
        if self.is_end() {
            b'\0'
        } else {
            self.bytes[self.current]
        }
    }
}

fn is_digit(c: u8) -> bool {
    c.is_ascii_digit()
}

fn is_alphabet(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}
