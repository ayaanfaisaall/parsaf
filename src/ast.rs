use lexaf::{
    StrIntr,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Word(String),
    Num(i64),
    Str(Vec<StrIntr>),
    Bool(bool),
    Array(Vec<Stmt>),
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

