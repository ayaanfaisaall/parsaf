use lexaf::{
    Token,
    SpannedToken, Span,
};
use crate::ast::{
    Stmt, SpannedStmt,
    RdrctOp,
};
use crate::error::ParsafError;

/// A Recursive Descent Parser for the custom shell language.
/// It processes a slice of `SpannedToken`s and constructs an Abstract Syntax Tree (AST).
pub struct Parser<'a> {
    tokens: &'a [SpannedToken<'a>],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [SpannedToken<'a>]) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn spanned(&self, start: usize, stmt: Stmt<'a>) -> Box<SpannedStmt<'a>> {
        let consumed = &self.tokens[start..self.pos];
        let first = consumed.first().expect("AST nodes consume at least one token");
        let last = consumed.iter().rev().find(|token| {
            !matches!(token.token, Token::NewLine | Token::SemiCln | Token::EOF)
        }).unwrap_or(first);
        Box::new(SpannedStmt {
            stmt,
            span: Span { start: first.span.start, end: last.span.end },
        })
    }

    fn span_peek(&self) -> Option<&'a SpannedToken<'a>> {
        self.tokens.get(self.pos)
    }

    fn peek(&self) -> Option<&'a Token<'a>> {
        self.span_peek().map(|st| &st.token)
    }

    fn next(&mut self) -> Option<&'a SpannedToken<'a>> {
        let next = self.tokens.get(self.pos);
        if next.is_some() {
            self.pos += 1;
        }
        next
    }

    fn skip(&mut self) {
        let to_be_skipped = self.peek();
        match to_be_skipped {
            Some(Token::NewLine) | Some(Token::SemiCln) => {
                self.next();
            }
            _ => {}
        }
    }

    fn expect(&mut self, expected: Token<'a>) -> Result<(), ParsafError> {
        if self.peek() == Some(&expected) {
            self.next();
            Ok(())
        } else {
            if let Some(st) = self.span_peek() {
                Err(ParsafError::ExpectedFound {
                    expected: expected.to_string(),
                    found: st.token.to_string(),
                    span: (st.span.start..st.span.end).into(),
                })
            } else {
                Err(ParsafError::UnexpectedEof)
            }
        }
    }

    fn unexpected(&mut self) -> Result<(), ParsafError> {
        let token = self.peek();
        match token {
            Some(Token::NewLine) | Some(Token::SemiCln) |
            Some(Token::RBrc)    | Some(Token::Pipe)    |
            Some(Token::LBrc)    | Some(Token::AndAnd)  |
            Some(Token::OrOr)    | Some(Token::RSqr)    |
            Some(Token::LSqr)    | Some(Token::EOF) => {
                Ok(())
            }
            Some(_) => {
                if let Some(st) = self.span_peek() {
                    Err(ParsafError::UnexpectedToken {
                        token: st.token.to_string(),
                        span: (st.span.start..st.span.end).into(),
                    })
                } else {
                    Err(ParsafError::UnexpectedEof)
                }
            }
            None => Err(ParsafError::UnexpectedEof),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<SpannedStmt<'a>>, ParsafError> {
        let mut stmts = Vec::new();
        loop {
            if let Some(token) = self.peek() {
                match token {
                    Token::EOF => { break; }
                    _ => {
                        let stmt = self.parse_andor()?;
                        match stmt.stmt {
                            Stmt::Empty => {}
                            _ => { stmts.push(*stmt); }
                        }
                    }
                }
            } else {
                break;
            }
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Box<SpannedStmt<'a>>, ParsafError> {
        let node_start = self.pos;
        let token = self.peek();

        match token {
            Some(Token::True)  | Some(Token::False) |
            Some(Token::Str(_))| Some(Token::LSqr)  |
            Some(Token::LBrc)  | Some(Token::Num(_))|
            Some(Token::Float(_))=> {
                self.parse_base_case()
            }
            Some(Token::Print) => self.parse_print_stmt(),
            Some(Token::Let) => self.parse_let_stmt(),
            Some(Token::If) => self.parse_if_stmt(),
            Some(Token::While) => self.parse_while_stmt(),
            Some(Token::For) => self.parse_for_stmt(),
            Some(Token::Break) => {
                self.next();
                Ok(self.spanned(node_start, Stmt::Break))
            }
            Some(Token::NewLine) | Some(Token::SemiCln) => {
                self.next();
                Ok(self.spanned(node_start, Stmt::Empty))
            }
            Some(Token::Pipe) | Some(Token::And)   |
            Some(Token::OrOr) | Some(Token::AndAnd)|
            Some(Token::RSqr) | Some(Token::RBrc) => {
                if let Some(st) = self.span_peek() {
                    Err(ParsafError::NotAllowedHere {
                        token: st.token.to_string(),
                        span: (st.span.start..st.span.end).into(),
                    })
                } else {
                    Err(ParsafError::UnexpectedEof)
                }
            }
            Some(Token::Bang) => {
                self.next();
                if let Some(st) = self.next() {
                    match &st.token {
                        Token::Num(n) => Ok(self.spanned(node_start, Stmt::Bang { num: Box::new(SpannedStmt { stmt: Stmt::Num(*n), span: st.span.clone() }) })),
                        _ => Err(ParsafError::ExpectedFound {
                            expected: "a number".to_string(),
                            found: st.token.to_string(),
                            span: (st.span.start..st.span.end).into(),
                        })
                    }
                } else {
                    Err(ParsafError::UnexpectedEof)
                }
            }
            Some(Token::Word(_)) => self.parse_cmd(),
            Some(_) => {
                self.next();
                Ok(self.spanned(node_start, Stmt::NotImplYet))
            }
            None => Err(ParsafError::UnexpectedEof),
        }
    }

    fn parse_print_stmt(&mut self) -> Result<Box<SpannedStmt<'a>>, ParsafError> {
        let node_start = self.pos;
        self.next();
        let value = self.parse_base_case()?;
        self.unexpected()?;
        self.skip();
        return Ok(self.spanned(node_start, Stmt::Print { val: value }))
    }

    fn parse_let_stmt(&mut self) -> Result<Box<SpannedStmt<'a>>, ParsafError> {
        let node_start = self.pos;
        self.next();
        let name = match self.span_peek() {
            Some(st) => match &st.token {
                Token::Word(w) => *w,
                _ => return Err(ParsafError::ExpectedFound {
                    expected: "a var name".to_string(),
                    found: st.token.to_string(),
                    span: (st.span.start..st.span.end).into(),
                })
            },
            None => return Err(ParsafError::UnexpectedEof),
        };
        self.next();
        self.expect(Token::Assign)?;
        let value = self.parse_base_case()?;
        self.unexpected()?;
        self.skip();
        return Ok(self.spanned(node_start, Stmt::Let { var: name, val: value }))
    }

    fn parse_if_stmt(&mut self) -> Result<Box<SpannedStmt<'a>>, ParsafError> {
        let node_start = self.pos;
        self.next();
        let condition = self.parse_andor()?;
        self.expect(Token::LBrc)?;
        self.skip();
        let block = self.parse_block()?;
        let mut alternate = None;
        let token = self.peek();

        match token {
            Some(Token::Elif) => {
                let elif = self.parse_if_stmt()?;
                alternate = Some(elif);
            }
            Some(Token::Else) => {
                self.next();
                let block_start = self.pos;
                self.expect(Token::LBrc)?;
                let block = self.parse_block()?;
                alternate = Some(self.spanned(block_start, Stmt::Block { block }));
            }
            _ => {}
        }

        Ok(self.spanned(node_start, Stmt::If { cond: condition, block: block, alter: alternate }))
    }

    fn parse_while_stmt(&mut self) -> Result<Box<SpannedStmt<'a>>, ParsafError> {
        let node_start = self.pos;
        self.next();
        let condition = self.parse_andor()?;
        self.expect(Token::LBrc)?;
        self.skip();
        let block = self.parse_block()?;
        Ok(self.spanned(node_start, Stmt::While { cond: condition, block: block }))
    }

    fn parse_for_stmt(&mut self) -> Result<Box<SpannedStmt<'a>>, ParsafError> {
        let node_start = self.pos;
        self.next();
        let iter = match self.span_peek() {
            Some(st) => match &st.token {
                Token::Word(w) => *w,
                _ => return Err(ParsafError::ExpectedFound {
                    expected: "an iterator".to_string(),
                    found: st.token.to_string(),
                    span: (st.span.start..st.span.end).into(),
                })
            },
            None => return Err(ParsafError::UnexpectedEof),
        };
        self.next();
        self.expect(Token::In)?;
        let start = self.parse_base_case()?;
        self.expect(Token::To)?;
        let end = self.parse_base_case()?;
        self.expect(Token::LBrc)?;
        self.skip();
        let block = self.parse_block()?;
        Ok(self.spanned(node_start, Stmt::For { iter, start, end, block }))
    }

    fn parse_cmd(&mut self) -> Result<Box<SpannedStmt<'a>>, ParsafError> {
        let node_start = self.pos;
        let cmd = self.parse_base_case()?;
        let mut args = Vec::new();
        while let Some(a) = self.peek() {
            match a {
                Token::NewLine | Token::SemiCln | Token::AndAnd | Token::OrOr |
                Token::RBrc    | Token::LBrc    | Token::RSqr   | Token::LSqr |
                Token::EOF     | Token::Pipe    | Token::RdrctOut | Token::RdrctErr |
                Token::RdrctBoth | Token::AppendOut | Token::AppendErr | Token::AppendBoth |
                Token::RdrctIn => {
                    self.skip();
                    break;
                }
                Token::And => {
                    let command = self.spanned(node_start, Stmt::Cmd { cmd , args });
                    self.next();
                    return Ok(self.spanned(node_start, Stmt::And { cmd: command }))
                }
                _ => {
                    let arg = self.parse_base_case()?;
                    args.push(*arg);
                }
            }
        }
        self.skip();
        Ok(self.spanned(node_start, Stmt::Cmd { cmd, args }))
    }

    fn parse_redirects(&mut self) -> Result<Box<SpannedStmt<'a>>, ParsafError> {
        let node_start = self.pos;
        let mut stmt = self.parse_stmt()?;
        loop {
            match self.peek() {
                Some(Token::RdrctOut) => {
                    self.next();
                    let target = self.parse_base_case()?;
                    stmt = self.spanned(node_start, Stmt::Rdrct { op: RdrctOp::Out, append: false , stmt, target });
                }
                Some(Token::RdrctErr) => {
                    self.next();
                    let target = self.parse_base_case()?;
                    stmt = self.spanned(node_start, Stmt::Rdrct { op: RdrctOp::Err, append: false , stmt, target });
                }
                Some(Token::RdrctBoth) => {
                    self.next();
                    let target = self.parse_base_case()?;
                    stmt = self.spanned(node_start, Stmt::Rdrct { op: RdrctOp::Both, append: false , stmt, target });
                }
                Some(Token::AppendOut) => {
                    self.next();
                    let target = self.parse_base_case()?;
                    stmt = self.spanned(node_start, Stmt::Rdrct { op: RdrctOp::Out, append: true , stmt, target });
                }
                Some(Token::AppendErr) => {
                    self.next();
                    let target = self.parse_base_case()?;
                    stmt = self.spanned(node_start, Stmt::Rdrct { op: RdrctOp::Err, append: true , stmt, target });
                }
                Some(Token::AppendBoth) => {
                    self.next();
                    let target = self.parse_base_case()?;
                    stmt = self.spanned(node_start, Stmt::Rdrct { op: RdrctOp::Both, append: true , stmt, target });
                }
                Some(Token::RdrctIn) => {
                    self.next();
                    let target = self.parse_base_case()?;
                    stmt = self.spanned(node_start, Stmt::Rdrct { op:RdrctOp::In, append: false, stmt, target });
                }
                _ => break,
            }
        }
        Ok(stmt)
    }

    fn parse_pipeline(&mut self) -> Result<Box<SpannedStmt<'a>>, ParsafError> {
        let node_start = self.pos;
        let stmt = self.parse_redirects()?;

        if !matches!(self.peek(), Some(Token::Pipe)) {
            return Ok(stmt);
        }

        let mut stmts = vec![*stmt];
        while let Some(Token::Pipe) = self.peek() {
            self.next();
            stmts.push(*self.parse_redirects()?);
        }
        Ok(self.spanned(node_start, Stmt::Pipe { stmts }))
    }

    fn parse_andor(&mut self) -> Result<Box<SpannedStmt<'a>>, ParsafError> {
        let node_start = self.pos;
        let mut stmt = self.parse_pipeline()?;

        loop {
            match self.peek() {
                Some(Token::AndAnd) => {
                    self.next();
                    let mut stmts = vec![*stmt];
                    stmts.push(*self.parse_pipeline()?);

                    while let Some(Token::AndAnd) = self.peek() {
                        self.next();
                        stmts.push(*self.parse_pipeline()?);
                    }
                    stmt = self.spanned(node_start, Stmt::AndAnd { stmts });
                }
                Some(Token::OrOr) => {
                    self.next();
                    let mut stmts = vec![*stmt];
                    stmts.push(*self.parse_pipeline()?);

                    while let Some(Token::OrOr) = self.peek() {
                        self.next();
                        stmts.push(*self.parse_pipeline()?);
                    }
                    stmt = self.spanned(node_start, Stmt::OrOr { stmts });
                }
                _ => { break; }
            }
        }
        Ok(stmt)
    }

    fn parse_base_case(&mut self) -> Result<Box<SpannedStmt<'a>>, ParsafError> {
        let node_start = self.pos;
        let spanned = self.next();
        if let Some(st) = spanned {
            match &st.token {
                Token::Word(w) => Ok(self.spanned(node_start, Stmt::Word(*w))),
                Token::Num(n) => Ok(self.spanned(node_start, Stmt::Num(*n))),
                Token::Float(f) => Ok(self.spanned(node_start, Stmt::Float(*f))),
                Token::Str(s) => Ok(self.spanned(node_start, Stmt::Str(s))),
                Token::True => Ok(self.spanned(node_start, Stmt::Bool(true))),
                Token::False => Ok(self.spanned(node_start, Stmt::Bool(false))),
                Token::LBrc => {
                    let block = self.parse_block()?;
                    Ok(self.spanned(node_start, Stmt::Block { block }))
                },
                Token::LSqr => self.parse_arrays(),
                _ => Err(ParsafError::BaseCase {
                    token: st.token.to_string(),
                    span: (st.span.start..st.span.end).into(),
                })
            }
        } else {
            Err(ParsafError::UnexpectedEof)
        }
    }

    fn parse_block(&mut self) -> Result<Vec<SpannedStmt<'a>>, ParsafError> {
        let mut stmts = Vec::new();
        loop {
            if let Some(token) = self.peek() {
                match token {
                    Token::RBrc => {
                        self.next();
                        break;
                    }
                    Token::RSqr => {
                        if let Some(st) = self.span_peek() {
                            return Err(ParsafError::UnexpectedToken {
                                token: st.token.to_string(),
                                span: (st.span.start..st.span.end).into(),
                            });
                        } else {
                            return Err(ParsafError::UnexpectedEof);
                        }
                    }
                    Token::EOF => {
                        if let Some(st) = self.span_peek() {
                            return Err(ParsafError::UnclosedDelimiter {
                                delimiter: "}".to_string(),
                                span: (st.span.start..st.span.end).into(),
                            });
                        } else {
                            return Err(ParsafError::UnexpectedEof);
                        }
                    }
                    _ => {
                        let stmt = self.parse_andor()?;
                        match stmt.stmt {
                            Stmt::Empty => {}
                            _ => stmts.push(*stmt),
                        }
                    }
                }
            } else {
                break;
            }
        }
        Ok(stmts)
    }

    fn parse_arrays(&mut self) -> Result<Box<SpannedStmt<'a>>, ParsafError> {
        let node_start = self.pos - 1;
        let mut stmts = Vec::new();
        loop {
            let token = self.peek();
            match token {
                Some(Token::Comma) => { self.next(); }
                Some(Token::RSqr) => {
                    self.next();
                    break;
                }
                Some(Token::NewLine) => { self.skip(); }
                Some(Token::RBrc) => {
                    if let Some(st) = self.span_peek() {
                        return Err(ParsafError::UnexpectedToken {
                            token: st.token.to_string(),
                            span: (st.span.start..st.span.end).into(),
                        });
                    } else {
                        return Err(ParsafError::UnexpectedEof);
                    }
                }
                Some(Token::EOF) => {
                    if let Some(st) = self.span_peek() {
                        return Err(ParsafError::UnclosedDelimiter {
                            delimiter: "]".to_string(),
                            span: (st.span.start..st.span.end).into(),
                        });
                    } else {
                        return Err(ParsafError::UnexpectedEof);
                    }
                }
                None => { return Err(ParsafError::UnexpectedEof); }
                _ => {
                    let base_case = self.parse_base_case()?;
                    stmts.push(*base_case);
                }
            }
        }
        Ok(self.spanned(node_start, Stmt::Array(stmts)))
    }
}
