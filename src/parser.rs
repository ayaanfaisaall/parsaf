use lexaf::{
    Token,
    SpannedToken,
};
use crate::ast::{
    Stmt,
};
use crate::error::ParsafError;

/// A Recursive Descent Parser for the custom shell language.
/// It processes a slice of `SpannedToken`s and constructs an Abstract Syntax Tree (AST).
pub struct Parser<'a> {
    tokens: &'a [SpannedToken],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [SpannedToken]) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn span_peek(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.pos)
    }

    fn peek(&self) -> Option<&Token> {
        self.span_peek().map(|st| &st.token)
    }

    fn next(&mut self) -> Option<&SpannedToken> {
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

    fn expect(&mut self, expected: Token) -> Result<(), ParsafError> {
        if self.peek() == Some(&expected) {
            self.next();
            Ok(())
        } else {
            if let Some(st) = self.span_peek() {
                Err(ParsafError::ExpectedFound {
                    expected: expected.to_string(),
                    found: st.token.clone(),
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
                        token: st.token.clone(),
                        span: (st.span.start..st.span.end).into(),
                    })
                } else {
                    Err(ParsafError::UnexpectedEof)
                }
            }
            None => Err(ParsafError::UnexpectedEof),
        }
    }
    
    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParsafError> {
        let mut stmts = Vec::new();
        loop {
            if let Some(token) = self.peek() {
                match token {
                    Token::EOF => { break; }
                    _ => {
                        let stmt = self.parse_andor()?;
                        match *stmt {
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

    fn parse_stmt(&mut self) -> Result<Box<Stmt>, ParsafError> {
        let token = self.peek();
        
        match token {
            Some(Token::True)  | Some(Token::False) |
            Some(Token::Str(_))| Some(Token::LSqr)  |
            Some(Token::LBrc)  | Some(Token::Num(_)) => {
                self.parse_base_case()
            }
            Some(Token::Print) => self.parse_print_stmt(),
            Some(Token::Let) => self.parse_let_stmt(),
            Some(Token::If) => self.parse_if_stmt(),
            Some(Token::While) => self.parse_while_stmt(),
            Some(Token::For) => self.parse_for_stmt(),
            Some(Token::Break) => {
                self.next();
                Ok(Box::new(Stmt::Break))
            }
            Some(Token::NewLine) | Some(Token::SemiCln) => {
                self.next();
                Ok(Box::new(Stmt::Empty))
            }
            Some(Token::Pipe) | Some(Token::And)  |
            Some(Token::OrOr) | Some(Token::AndAnd) => {
                if let Some(st) = self.span_peek() {
                    Err(ParsafError::NotAllowedHere {
                        token: st.token.clone(),
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
                        Token::Num(n) => Ok(Box::new(Stmt::Bang { num: Box::new(Stmt::Num(n.clone())) })),
                        _ => Err(ParsafError::ExpectedFound {
                            expected: "a number".to_string(),
                            found: st.token.clone(),
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
                Ok(Box::new(Stmt::NotImplYet))
            }
            None => Err(ParsafError::UnexpectedEof),
        }
    }

    fn parse_print_stmt(&mut self) -> Result<Box<Stmt>, ParsafError> {
        self.next();
        let value = self.parse_base_case()?;
        self.skip();
        return Ok(Box::new(Stmt::Print { val: value }))
    }

    fn parse_let_stmt(&mut self) -> Result<Box<Stmt>, ParsafError> {
        self.next();
        let name = match self.span_peek() {
            Some(st) => match &st.token {
                Token::Word(w) => w.to_string(),
                _ => return Err(ParsafError::ExpectedFound {
                    expected: "a var name".to_string(),
                    found: st.token.clone(),
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
        return Ok(Box::new(Stmt::Let { var: name, val: value }))
    }
    
    fn parse_if_stmt(&mut self) -> Result<Box<Stmt>, ParsafError> {
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
                self.expect(Token::LBrc)?;
                let block = self.parse_block()?;
                alternate = Some(Box::new(Stmt::Block { block }));
            }
            _ => {}
        }
        
        Ok(Box::new(Stmt::If { cond: condition, block: block, alter: alternate }))
    }

    fn parse_while_stmt(&mut self) -> Result<Box<Stmt>, ParsafError> {
        self.next();
        let condition = self.parse_andor()?;
        self.expect(Token::LBrc)?;
        self.skip();
        let block = self.parse_block()?;
        Ok(Box::new(Stmt::While { cond: condition, block: block }))
    }

    fn parse_for_stmt(&mut self) -> Result<Box<Stmt>, ParsafError> {
        self.next();
        let iter = match self.span_peek() {
            Some(st) => match &st.token {
                Token::Word(w) => w.to_string(),
                _ => return Err(ParsafError::ExpectedFound {
                    expected: "an iterator".to_string(),
                    found: st.token.clone(),
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
        Ok(Box::new(Stmt::For { iter, start, end, block }))
    }

    fn parse_cmd(&mut self) -> Result<Box<Stmt>, ParsafError> {
        let cmd = self.parse_base_case()?; 
        let mut args = Vec::new();
        while let Some(a) = self.peek() {
            match a {
                Token::NewLine | Token::SemiCln | Token::AndAnd | Token::OrOr | 
                Token::RBrc    | Token::LBrc    | Token::RSqr   | Token::LSqr |
                Token::EOF     | Token::Pipe => {
                    self.skip();
                    break;
                }
                Token::And => {
                    self.next();
                    let command = Box::new(Stmt::Cmd { cmd , args });
                    return Ok(Box::new(Stmt::And { cmd: command }))
                }
                _ => {
                    let arg = self.parse_base_case()?;
                    args.push(*arg);
                }
            }
        }
        self.skip();
        Ok(Box::new(Stmt::Cmd { cmd, args }))
    }

    fn parse_pipeline(&mut self) -> Result<Box<Stmt>, ParsafError> {
        let stmt = self.parse_stmt()?;
        
        if !matches!(self.peek(), Some(Token::Pipe)) {
            return Ok(stmt);
        }
        
        let mut stmts = vec![*stmt];
        while let Some(Token::Pipe) = self.peek() {
            self.next();
            stmts.push(*self.parse_stmt()?);
        }
        Ok(Box::new(Stmt::Pipe { stmts }))
    } 

    fn parse_andor(&mut self) -> Result<Box<Stmt>, ParsafError> {
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
                    stmt = Box::new(Stmt::AndAnd { stmts });
                }
                Some(Token::OrOr) => {
                    self.next(); 
                    let mut stmts = vec![*stmt];
                    stmts.push(*self.parse_pipeline()?);
                    
                    while let Some(Token::OrOr) = self.peek() {
                        self.next(); 
                        stmts.push(*self.parse_pipeline()?);
                    }
                    stmt = Box::new(Stmt::OrOr { stmts });
                }
                _ => { break; }
            }
        }
        Ok(stmt)
    }

    fn parse_base_case(&mut self) -> Result<Box<Stmt>, ParsafError> {
        let spanned = self.next();
        if let Some(st) = spanned {
            match &st.token {
                Token::Word(w) => Ok(Box::new(Stmt::Word(w.to_string()))),
                Token::Num(n) => Ok(Box::new(Stmt::Num(n.clone()))),
                Token::Str(s) => Ok(Box::new(Stmt::Str(s.clone()))),
                Token::True => Ok(Box::new(Stmt::Bool(true))),
                Token::False => Ok(Box::new(Stmt::Bool(false))),
                Token::LBrc => Ok(Box::new(Stmt::Block { block: self.parse_block()? })),
                Token::LSqr => self.parse_arrays(),
                _ => Err(ParsafError::UnexpectedToken {
                    token: st.token.clone(),
                    span: (st.span.start..st.span.end).into(),
                })
            }
        } else {
            Err(ParsafError::UnexpectedEof)
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParsafError> {
        let mut stmts = Vec::new();
        loop {
            if let Some(token) = self.peek() {
                match token {
                    Token::RBrc => {
                        self.next();
                        break;
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
                        match *stmt {
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

    fn parse_arrays(&mut self) -> Result<Box<Stmt>, ParsafError> {
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
        Ok(Box::new(Stmt::Array(stmts)))
    }
}
