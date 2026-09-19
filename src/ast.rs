use lexaf::{
    StrIntr,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RdrctOp {
    In,
    Out,
    Err,
    Both,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt<'a> {
    Word(&'a str),
    Num(i64),
    Float(f64),
    Str(&'a Vec<StrIntr<'a>>),
    Bool(bool),
    Array(Vec<Stmt<'a>>),
    Cmd {
        cmd: Box<Stmt<'a>>,
        args: Vec<Stmt<'a>>,
    },
    Let {
        var: &'a str,
        val: Box<Stmt<'a>>,
    }, 
    Print {
        val: Box<Stmt<'a>>,
    },
    If {
        cond: Box<Stmt<'a>>,
        block: Vec<Stmt<'a>>,
        alter: Option<Box<Stmt<'a>>>, 
    },
    For {
        iter: &'a str,
        start: Box<Stmt<'a>>,
        end: Box<Stmt<'a>>,
        block: Vec<Stmt<'a>>,
    },
    While {
        cond: Box<Stmt<'a>>,
        block: Vec<Stmt<'a>>,
    },
    Block {
        block: Vec<Stmt<'a>>,
    },
    Rdrct {
        op: RdrctOp,
        append: bool,
        stmt: Box<Stmt<'a>>,
        target: Box<Stmt<'a>>,
    },
    Pipe {
        stmts: Vec<Stmt<'a>>,
    },
    AndAnd {
        stmts: Vec<Stmt<'a>>,
    },
    OrOr {
        stmts: Vec<Stmt<'a>>,
    },
    And {
        cmd: Box<Stmt<'a>>,
    },
    Bang {
        num: Box<Stmt<'a>>,
    },
    Break,
    Empty,
    NotImplYet,
}
