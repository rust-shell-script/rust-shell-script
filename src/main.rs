mod util;
mod scanner;
mod parser;
mod resolver;
mod bash_backend;
mod rust_backend;

use std::env;
use std::process;
use std::fs;

fn usage(cmd: &str) {
    eprintln!("Usage: {} {{-t|--target bash|rust}} <source.rss>", cmd);
    process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 ||
       (args[1] != "-t" && args[1] != "--target") ||
       (args[2] != "bash" && args[2] != "rust") {
        usage(&args[0]);
    }
    let (target, filename) = (&args[2], &args[3]);
    compile(filename, target);
}

fn compile(filename: &str, target: &str)  {
    let file: Vec<char> = fs::read_to_string(filename)
        .expect("error reading file")
        .chars()
        .collect();

    let tokens = scanner::scan(file);
    let stmts = parser::parse(tokens);
    let sym_table = resolver::gen_sym_table(&stmts);

    if target == "bash" {
        bash_backend::gen_code(&stmts,
                               &String::from(filename).replace(".rss", ".sh"));
    } else {
        rust_backend::gen_code(&stmts,
                               &sym_table,
                               &String::from(filename).replace(".rss", ".rs"));
    }
}
