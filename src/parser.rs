use crate::parser::Expr::CallFun;
use std::process;
use crate::scanner::Token;
use crate::util::CmpType;
use crate::util::Newline;
use crate::util::ScanIter;
use crate::parser::Stmt::{CallCmd, DefCmd, DefFun, Return};

#[derive(Debug)]
pub enum Stmt {
    DefVar(String, Option<Expr>),
    DefFun(String, Vec<String>, Vec<Stmt>),
    DefCmd(String, Vec<String>, Vec<Stmt>),

    CallCmd(String, Vec<Expr>),
    Return(Expr),
}
impl CmpType for Stmt{}

#[derive(Debug)]
pub enum Expr {
    LitNum(isize),
    LitStr(String),
    Var(Var),
    CallFun(String, Vec<Expr>),
}

#[derive(Clone, Debug)]
pub struct Var {
    pub identifier: String,
    pub hops: usize,
}
impl Var {
    pub fn new(id: String) -> Self {
        Self {
            identifier: id,
            hops: 0,
        }
    }
}

macro_rules! parse_error {
    ($line: expr, $($arg:tt)*) => {
        eprint!("Parse error at line {}: ", $line);
        eprintln!($($arg)*);
        process::exit(1);
    }
}

impl ScanIter<Token> {
    fn consume(&mut self, expected: Token) {
        if self.match_item(expected.clone()) {
            return;
        } else {
            parse_error!(self.line(), "got {:?}, expect token: {:?}",
                        self.peek(), expected);
        }
    }

    fn consume_identifier(&mut self) -> String {
        if let Token::Identifier(ref name) = *self.peek() {
            let id = name.clone();
            self.advance();
            id
        } else {
            parse_error!(self.line(), "got {:?}, expect identifier", self.peek());
        }
    }

    fn consume_litstr(&mut self) -> String {
        if let Token::String(ref name) = *self.peek() {
            let id = name.clone();
            self.advance();
            id
        } else {
            parse_error!(self.line(), "got {:?}, expect literal string", self.peek());
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Vec<Stmt> {
    let mut tokens = ScanIter::new(tokens);
    let mut stmts = Vec::new();

    loop {
        while !tokens.is_at_end() && tokens.peek().is_newline() {
            tokens.advance();
        }
        if tokens.is_at_end() {
            break;
        }

        let stmt = parse_stmt(&mut tokens);
        stmts.push(stmt);
    }
    stmts
}

fn parse_stmt(tokens: &mut ScanIter<Token>) -> Stmt {
    if tokens.match_item(Token::Fun) {
        return stmt_def_fun(tokens);
    } else if tokens.match_item(Token::Cmd) {
        return stmt_def_cmd(tokens);
    } else if tokens.match_item(Token::Return) {
        return stmt_return(tokens);
    }

    parse_error!(tokens.line(), "unexpected token: {:?}", tokens.peek());
}

fn stmt_def_fun(tokens: &mut ScanIter<Token>) -> Stmt {
    let id = tokens.consume_identifier();
    tokens.consume(Token::LeftParen);
    let (parameters, body) = fun_parameters_and_body(tokens);
    DefFun(id, parameters, body)
}

fn stmt_def_cmd(tokens: &mut ScanIter<Token>) -> Stmt {
    let id = tokens.consume_identifier();
    tokens.consume(Token::LeftParen);
    let (parameters, body) = fun_parameters_and_body(tokens);
    DefCmd(id, parameters, body)
}

fn stmt_return(tokens: &mut ScanIter<Token>) -> Stmt {
    if let Token::Number(n) = *tokens.peek() {
        let ret = n.clone();
        tokens.advance();
        return Return(Expr::LitNum(ret));
    } else if let Token::String(ref s) = *tokens.peek() {
        let ret = s.clone();
        tokens.advance();
        return Return(Expr::LitStr(ret));
    }
    parse_error!(tokens.line(), "Wrong return value");
}

fn fun_parameters_and_body(tokens: &mut ScanIter<Token>) -> (Vec<String>, Vec<Stmt>) {
    let mut parameters = Vec::new();
    if *tokens.peek() != Token::RightParen {
        loop {
            let id = tokens.consume_identifier();
            parameters.push(id);
            if !tokens.match_item(Token::Comma) {
                break;
            }
        }
    }
    tokens.consume(Token::RightParen);
    tokens.consume(Token::LeftBrace);
    let body = block_statement(tokens);

    (parameters, body)
}


fn block_statement(tokens: &mut ScanIter<Token>) -> Vec<Stmt> {
    let mut stmts = Vec::new();

    while *tokens.peek() != Token::RightBrace && !tokens.is_at_end() {
        if tokens.peek().is_newline() {
            tokens.advance();
            continue;
        }
        stmts.push(cmds(tokens));
    }

    tokens.consume(Token::RightBrace);
    stmts
}

fn cmds(tokens: &mut ScanIter<Token>) -> Stmt {
    if tokens.match_item(Token::Let) {
        var_declaration(tokens)
    } else if tokens.match_item(Token::Return) {
        stmt_return(tokens)
    } else {
        call_cmd(tokens)
    }
}

fn var_declaration(tokens: &mut ScanIter<Token>) -> Stmt {
    let name = tokens.consume_identifier();

    let initializer = if tokens.match_item(Token::Equal) {
        Some(expression(tokens))
    } else {
        None
    };

    Stmt::DefVar(name, initializer)
}

fn call_cmd(tokens: &mut ScanIter<Token>) -> Stmt {
    let id = tokens.consume_identifier();
    let mut args = Vec::new();
    while *tokens.peek() != Token::Newline {
        if tokens.check_item(Token::String("".into())) {
            args.push(Expr::LitStr(tokens.consume_litstr()));
        } else {
            tokens.consume(Token::DollarSign);
            args.push(Expr::Var(Var::new(tokens.consume_identifier())));
        }
    }

    CallCmd(id, args)
}

fn expression(tokens: &mut ScanIter<Token>) -> Expr {
    if let Token::Number(n) = *tokens.peek() {
        let ret = n.clone();
        tokens.advance();
        return Expr::LitNum(ret);
    } else if let Token::String(ref s) = *tokens.peek() {
        let ret = s.clone();
        tokens.advance();
        return Expr::LitStr(ret);
    } else if tokens.match_item(Token::DollarSign) {
        if tokens.match_item(Token::LeftParen) {
            let res = call_fun(tokens);
            tokens.consume(Token::RightParen);
            return res;
        } else {
            return Expr::LitStr(tokens.consume_identifier());
        }
    }
    parse_error!(tokens.line(), "Wrong expression");
}

fn call_fun(tokens: &mut ScanIter<Token>) -> Expr {
    let id = tokens.consume_identifier();
    let mut args = Vec::new();
    while *tokens.peek() != Token::Newline {
        if tokens.check_item(Token::String("".into())) {
            args.push(Expr::LitStr(tokens.consume_litstr()));
        } else if tokens.match_item(Token::DollarSign) {
            args.push(Expr::Var(Var::new(tokens.consume_identifier())));
        } else {
            break;
        }
    }

    CallFun(id, args)
}
