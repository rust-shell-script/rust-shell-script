use std::process;
use crate::util::Newline;
use crate::util::CmpType;
use crate::util::ScanIter;
use crate::util::char;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    DollarSign,
    Slash,
    Comma,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,

    Identifier(String),
    String(String),
    Number(isize),

    Fun,
    Cmd,
    Main,
    Return,
    Let,
    If,
    While,
    For,

    Newline,
    Eof,
}

impl CmpType for Token {}
impl Newline for Token {
    fn is_newline(&self) -> bool {
        *self == Token::Newline
    }
}

macro_rules! scan_error {
    ($line: expr, $($arg:tt)*) => {
        eprint!("Scan error at line {}: ", $line);
        eprintln!($($arg)*);
        process::exit(1);
    }
}

pub fn scan(src: Vec<char>) -> Vec<Token> {
    let mut src = ScanIter::new(src);
    let mut tokens: Vec<Token> = Vec::new();

    loop {
        let token = scan_token(&mut src);
        tokens.push(token);
        if src.is_at_end() {
            break;
        }
    }
    tokens
}

fn scan_token(src: &mut ScanIter<char>) -> Token {
    if src.is_at_end() {
        return Token::Eof;
    }

    let line = src.line();
    let c = src.peek().clone();
    src.advance();
    match c {
        '(' => Token::LeftParen,
        ')' => Token::RightParen,
        '{' => Token::LeftBrace,
        '}' => Token::RightBrace,
        '$' => Token::DollarSign,
        ',' => Token::Comma,
        '!' => if src.match_item('=') {
            Token::BangEqual
        } else {
            Token::Bang
        },
        '=' => if src.match_item('=') {
            Token::EqualEqual
        } else {
            Token::Equal
        },
        '/' => {
            if src.match_item('/') {
                while *src.peek() != '\n' && !src.is_at_end() {
                    src.advance();
                }
                scan_token(src)
            } else {
                Token::Slash
            }
        },
        //  skip whitespaces
        ' '|'\r'|'\t' => scan_token(src),
        '\n' => Token::Newline,
        '"' => scan_string(src),
        _ =>  {
            if char::is_digit(c) {
                scan_number(src)
            } else if char::is_alpha_or_underscore(c) {
                scan_identifier_or_keyword(src)
            } else {
                scan_error!(line, "unexpected character '{}'", c);
            }
        }
    }
}

fn scan_string(src: &mut ScanIter<char>) -> Token {
    let mut s = String::new();
    let mut c = *src.peek();

    while c != '"' && !src.is_at_end() {
        if c == '\n' {
            scan_error!(src.line(), "do not support multi-line string");
        }
        s.push(c);
        src.advance();
        c = *src.peek();
    }

    if src.is_at_end() {
        scan_error!(src.line(), "unterminated string");
    }

    // Closing ".
    src.advance();
    Token::String(s)
}

fn scan_number(src: &mut ScanIter<char>) -> Token {
    let mut s = String::new();
    let mut c = *src.peek();

    s.push(*src.previous());
    while char::is_digit(c) {
        s.push(c);
        src.advance();
        c = *src.peek();
    }
    Token::Number(s.parse().unwrap())
}

fn scan_identifier_or_keyword(src: &mut ScanIter<char>) -> Token {
    let mut s = String::new();
    let mut c = *src.peek();

    s.push(*src.previous());
    while char::is_alpha_numeric_or_underscore(c) {
        s.push(c);
        src.advance();
        c = *src.peek();
    }

    if c == '!' {
        s.push(c);
        src.advance();
        return Token::Identifier(s);
    }

    match s.as_ref() {
        "fun"       => Token::Fun,
        "cmd"       => Token::Cmd,
        "let"       => Token::Let,
        "return"    => Token::Return,
        _           => Token::Identifier(s),
    }
}
