use lexaf::{
    StrIntr, Span,
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
    Array(Vec<SpannedStmt<'a>>),
    Cmd {
        cmd: Box<SpannedStmt<'a>>,
        args: Vec<SpannedStmt<'a>>,
    },
    Let {
        var: &'a str,
        val: Box<SpannedStmt<'a>>,
    },
    Print {
        val: Box<SpannedStmt<'a>>,
    },
    If {
        cond: Box<SpannedStmt<'a>>,
        block: Vec<SpannedStmt<'a>>,
        alter: Option<Box<SpannedStmt<'a>>>,
    },
    For {
        iter: &'a str,
        start: Box<SpannedStmt<'a>>,
        end: Box<SpannedStmt<'a>>,
        block: Vec<SpannedStmt<'a>>,
    },
    While {
        cond: Box<SpannedStmt<'a>>,
        block: Vec<SpannedStmt<'a>>,
    },
    Block {
        block: Vec<SpannedStmt<'a>>,
    },
    Rdrct {
        op: RdrctOp,
        append: bool,
        stmt: Box<SpannedStmt<'a>>,
        target: Box<SpannedStmt<'a>>,
    },
    Pipe {
        stmts: Vec<SpannedStmt<'a>>,
    },
    AndAnd {
        stmts: Vec<SpannedStmt<'a>>,
    },
    OrOr {
        stmts: Vec<SpannedStmt<'a>>,
    },
    And {
        cmd: Box<SpannedStmt<'a>>,
    },
    Bang {
        num: Box<SpannedStmt<'a>>,
    },
    Break,
    Empty,
    NotImplYet,
}

/// A statement with its half-open UTF-8 byte range in the source.
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedStmt<'a> {
    pub stmt: Stmt<'a>,
    pub span: Span,
}
