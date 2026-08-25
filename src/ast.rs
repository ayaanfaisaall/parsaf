//
// brah this is not a bad one actually, this parser is 
// very powerful, i know its not optimised, it just 
// clones the strings from the tokenizer, 
// todo!():
// change String to &'a str, in the tokenizer and then 
// use the string slices all across the program, 
// lets see if it works, 
//
use lexaf::{
    StrIntr,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    //
    // base_case: the statements which consists a value,
    //
    Word(String),
    Num(i64),
    Str(Vec<StrIntr>),
    ExitCode(bool),
    //
    // stmt: the statements which returns a value (ExitCode)
    // during evaluation, as well as perform an action.
    //
    Cmd {
        cmd: Box<Stmt>,
        args: Vec<Stmt>,
    },
    Let {
        var: String,
        val: Box<Stmt>,
    }, 
    Print {
        val: Box<Stmt>,
    },
    If {
        cond: Box<Stmt>,
        block: Vec<Stmt>,
        alter: Option<Box<Stmt>>, 
    },
    For {
        iter: String,
        start: Box<Stmt>,
        end: Box<Stmt>,
        block: Vec<Stmt>,
    },
    While {
        cond: Box<Stmt>,
        block: Vec<Stmt>,
    },
    Block {
        block: Vec<Stmt>,
    },
    Pipe {
        stmts: Vec<Stmt>,
    },
    AndAnd {
        stmts: Vec<Stmt>,
    },
    OrOr {
        stmts: Vec<Stmt>,
    },
    And {
        cmd: Box<Stmt>,
    },
    Bang {
        num: Box<Stmt>,
    },
    Break,
    Empty,
    NotImplYet,
}

