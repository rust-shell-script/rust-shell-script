use std::collections::HashSet;
use crate::parser::Stmt;
use crate::parser::Stmt::{DefCmd, DefFun};

pub fn gen_sym_table(stmts: &Vec<Stmt>) -> HashSet<&String> {
    let mut sym_table = HashSet::new();
    for stmt in stmts {
        match stmt {
            DefFun(fun_name, _, _) => {
                sym_table.insert(fun_name);
            }
            DefCmd(cmd_name, _, _) => {
                sym_table.insert(cmd_name);
            }
            _ => (),
        }
    }

    sym_table
}
