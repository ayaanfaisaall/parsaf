use lexaf::{
    Token,
};
use crate::ast::{
    Stmt,
};

pub struct Parser <'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl <'a> Parser <'a> {
    pub fn new (tokens: &'a [Token]) -> Self {
        Parser {
            tokens,
            pos: 0,
        }
    }

    fn peek (&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next (&mut self) -> Option<&Token> {
        let next = self.tokens.get(self.pos);
        self.pos += 1;
        next
    }

    fn skip (&mut self) {
        let to_be_skipped = self.peek();
        match to_be_skipped {
            Some(Token::NewLine) |
            Some(Token::SemiCln) => {
                self.next();
            }
            _ => {} 
        }
    }

    fn expect (&mut self, expected: Token) -> Result<(),String> {
        if self.peek() == Some(&expected) {
            self.next();
            Ok(())
        } else {
            Err(format!("parsaf: expected: {:?}, found: {:?}", expected, self.peek()))
        } 
    }

    fn unexpected (&mut self) -> Result<(), String> {
        let token = self.peek();
        match token {
            Some(Token::NewLine) | Some(Token::SemiCln) |
            Some(Token::RBrc) | Some(Token::Pipe)   |
            Some(Token::LBrc) | Some(Token::AndAnd) | 
            Some(Token::OrOr) | Some(Token::RSqr)   |
            Some(Token::LSqr) => {
                Ok(())
            }
            _ => {
                Err(format!("parsaf: unexpected token found: {:?}", token))
            }
        }
    }
    
    pub fn parse (&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        loop {
            if let Some(token) = self.peek() {
                match token {
                    Token::EOF => {
                        break;
                    }
                    _ => {
                        let stmt = self.parse_stmt(0)?;
                        match *stmt {
                            Stmt::Empty => {}
                            _ => {
                                stmts.push(*stmt);
                            }
                        }
                    }
                }
            }
        }
        Ok(stmts)
    }

    fn parse_stmt (&mut self, p: u8) -> Result<Box<Stmt>, String> {
        let token = self.peek();
        let mut stmt = None;
        match token {
            Some(Token::True)  | 
            Some(Token::False) |
            Some(Token::Str(_))|
            Some(Token::Num(_)) => {
                match stmt {
                    Some(_) => {},
                    None => stmt = Some(self.parse_base_case()?),
                }
            }
            Some(Token::Print) => {
                stmt = Some(self.parse_print_stmt()?);
            }
            Some(Token::Let) => {
                stmt = Some(self.parse_let_stmt()?);
            }
            Some(Token::If) => {
                stmt = Some(self.parse_if_stmt()?);
            }
            Some(Token::While) => {
                stmt = Some(self.parse_while_stmt()?);
            }
            Some(Token::For) => {
                stmt = Some(self.parse_for_stmt()?);
            }
            Some(Token::LBrc) => {
                self.next();
                stmt = Some(Box::new(Stmt::Block { block: self.parse_block()? }));
            }
            // Some(Token::LSqr) => {
            //     self.next();
            //     stmt = Some(Box::new(Stmt::SqBlock { block: self.parse_sq_block()? }));
            // }
            Some(Token::Break) => {
                self.next();
                stmt = Some(Box::new(Stmt::Break));
            }
            Some(Token::NewLine) |
            Some(Token::SemiCln) => {
                self.next();
                stmt = Some(Box::new(Stmt::Empty));
            }
            Some(Token::Pipe) |
            Some(Token::And)  |
            Some(Token::OrOr) |
            Some(Token::AndAnd) => {
                return Err(format!("parsaf: token: {:?} not allowed in start", token));
            }
            Some(Token::Bang) => {
                self.next();
                let number = match self.next() {
                    Some(Token::Num(n)) => {
                        return Ok(Box::new(Stmt::Bang { num: Box::new(Stmt::Num(n.clone())) }));
                    }
                    _ => Err(format!("parsaf: expected number, found: {:?}", self.peek()))
                }; 
                stmt = number?
            }
            Some(Token::Word(_)) => {
                stmt = Some(self.parse_cmd()?);
            }
            _ => {
                self.next();
                stmt = Some(Box::new(Stmt::NotImplYet));
            }
        }
        let stmt = match stmt {
            Some(s) => s,
            None => Box::new(Stmt::Empty),
        };
        let token = self.peek();
        if p == 0 {
            match token {
                Some(Token::Pipe) => {
                    self.next();
                    self.parse_pipeline(Some(stmt))
                }
                Some(Token::AndAnd) => {
                    self.next();
                    self.parse_andand(Some(stmt))
                }
                Some(Token::OrOr) => {
                    self.next();
                    self.parse_oror(Some(stmt))
                }
                _ => Ok(stmt),
            }
        } else {
            return Ok(stmt);
        }
    //
    // programming without any internet!, is just rejecting your own logic 
    // continuously, which took hours or even days to even process, until 
    // some logic is acceptable enough, an example is this:
    //
        // match token {
        //     Some(Token::Pipe)   | 
        //     Some(Token::AndAnd) |
        //     Some(Token::OrOr) => {
        //         if (p == 0 || p == 2 || p == 3) && token == Some(&Token::Pipe) {
        //             self.next();
        //             return self.parse_pipeline(Some(stmt));
        //         } else if (p == 0 || p == 1 || p == 3) && token == Some(&Token::AndAnd) {
        //             self.next();
        //             return self.parse_andand(Some(stmt));
        //         } else if (p == 0 || p == 1 || p == 2) && token == Some(&Token::OrOr) {
        //             self.next();                
        //             return self.parse_oror(Some(stmt));
        //         } else {
        //             return Ok(stmt);
        //         }
        //     }
        //     _ => Ok(stmt),
        // }
    }

    fn parse_print_stmt(&mut self) -> Result<Box<Stmt>, String> {
        self.next();
        let value = self.parse_base_case()?;
        self.skip();
        return Ok(Box::new(Stmt::Print { val: value }))
    }

    fn parse_let_stmt (&mut self) -> Result<Box<Stmt>, String> {
        self.next();
        let name = match self.peek() {
            Some(Token::Word(w)) => w.to_string(),
            _ => return Err(format!("parsaf: expected: name, found: {:?}", self.peek())), 
        };
        self.next();
        self.expect(Token::Assign)?;
        let value = self.parse_base_case()?;
        self.unexpected()?;
        self.skip();
        return Ok(Box::new(Stmt::Let { var: name, val: value }))
    }
    
    fn parse_if_stmt (&mut self) -> Result<Box<Stmt>, String> {
        self.next();
        let condition = self.parse_stmt(0)?;
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

    fn parse_while_stmt (&mut self) -> Result<Box<Stmt>, String> {
        self.next();
        let condition = self.parse_stmt(0)?;
        self.expect(Token::LBrc)?;
        self.skip();
        let block = self.parse_block()?;
        Ok(Box::new(Stmt::While { cond: condition, block: block }))
    }

    fn parse_for_stmt (&mut self) -> Result<Box<Stmt>, String> {
        self.next();
        let iter = match self.peek() {
            Some(Token::Word(w)) => w.to_string(),
            _ => return Err(format!("parsaf: expected iterator, found {:?}", self.peek()))
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

    fn parse_cmd (&mut self) -> Result<Box<Stmt>, String> {
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

    fn parse_pipeline (&mut self, stmt: Option<Box<Stmt>>) -> Result<Box<Stmt>, String> {
        let mut stmts = Vec::new();
        match stmt {
            Some(c) => stmts.push(*c),
            None => {}
        }
        loop {
            let next_stmt = self.parse_stmt(1)?;
            stmts.push(*next_stmt);
            let token = self.peek();
            match token {
                Some(Token::Pipe) => {
                    self.next();
                    let some = self.peek();
                    match some {
                        Some(Token::EOF) | Some(Token::NewLine) |
                        Some(Token::SemiCln) | None => {
                            return Err(format!("parsaf: expected something after '|', found: {:?}", some));
                        }
                        _ => {}
                    }
                }
                Some(Token::AndAnd) => {
                    self.next();
                    let pipe = Box::new(Stmt::Pipe { stmts });
                    return self.parse_andand(Some(pipe));
                }
                Some(Token::OrOr) => {
                    self.next();
                    let pipe = Box::new(Stmt::Pipe { stmts });
                    return self.parse_oror(Some(pipe));
                }
                _ => {
                    break;
                }
            }
        }
        Ok(Box::new(Stmt::Pipe { stmts }))
    }

    fn parse_andand (&mut self, stmt: Option<Box<Stmt>>) -> Result<Box<Stmt>, String> {
        let mut stmts = Vec::new();
        match stmt {
            Some(c) => stmts.push(*c),
            None => {}
        }
        loop {
            let next_stmt = self.parse_stmt(1)?;
            stmts.push(*next_stmt);
            let token = self.peek();
            match token {
                Some(Token::AndAnd) => {
                    self.next();
                    let some = self.peek();
                    match some {
                        Some(Token::EOF) | Some(Token::NewLine) |
                        Some(Token::SemiCln) | None => {
                            return Err(format!("parsaf: expected something after '&&', found: {:?}", some));
                        }
                        _ => {}
                    }
                }
                Some(Token::Pipe) => {
                    self.next();
                    let andand = Box::new(Stmt::AndAnd { stmts });
                    return self.parse_pipeline(Some(andand));
                }
                Some(Token::OrOr) => {
                    self.next();
                    let andand = Box::new(Stmt::AndAnd { stmts });
                    return self.parse_oror(Some(andand));
                }
                _ => {
                    break;
                }
            }
        }
        Ok(Box::new(Stmt::AndAnd { stmts }))
    }

    fn parse_oror (&mut self, stmt: Option<Box<Stmt>>) -> Result<Box<Stmt>, String> {
        let mut stmts = Vec::new();
        match stmt {
            Some(c) => stmts.push(*c),
            None => {}
        }
        loop {
            let next_stmt = self.parse_stmt(1)?;
            stmts.push(*next_stmt);
            let token = self.peek();
            match token {
                Some(Token::OrOr) => {
                    self.next();
                    let some = self.peek();
                    match some {
                        Some(Token::EOF) | Some(Token::NewLine) |
                        Some(Token::SemiCln) | None => {
                            return Err(format!("parsaf: expected something after '||', found: {:?}", some));
                        }
                        _ => {}
                    }
                }
                Some(Token::Pipe) => {
                    self.next();
                    let oror = Box::new(Stmt::OrOr { stmts });
                    return self.parse_pipeline(Some(oror));
                }
                Some(Token::AndAnd) => {
                    self.next();
                    let oror = Box::new(Stmt::OrOr { stmts });
                    return self.parse_andand(Some(oror));
                }
                _ => {
                    break;
                }
            }
        }
        Ok(Box::new(Stmt::OrOr { stmts }))
    }

    fn parse_base_case (&mut self) -> Result<Box<Stmt>, String> {
        let token = self.next();
        match token {
            Some(Token::Word(w)) => {
                Ok(Box::new(Stmt::Word(w.to_string()))) 
            }
            Some(Token::Num(n)) => {
                Ok(Box::new(Stmt::Num(n.clone())))
            }
            Some(Token::Str(s)) => {
                Ok(Box::new(Stmt::Str(s.clone())))
            }
            Some(Token::True) => {
                Ok(Box::new(Stmt::ExitCode(true)))
            }
            Some(Token::False) => {
                Ok(Box::new(Stmt::ExitCode(false)))
            }
            Some(Token::LSqr) => {
                Ok(Box::new(Stmt::SqBlock { block: self.parse_sq_block()? }))
            }
            _ => {
                Err(format!("parsaf: expected base_case, found: {:?}", token))
            }
        }
    }

    fn parse_block (&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        loop {
            if let Some(token) = self.peek() {
                match token {
                    Token::RBrc => {
                        self.next();
                        self.skip();
                        break;
                    }
                    _ => {
                        let stmt = self.parse_stmt(0)?;
                        match *stmt {
                            Stmt::Empty => {}
                            _ => {
                                stmts.push(*stmt);
                            }
                        }
                    }
                }
            }
        }
        Ok(stmts)
    } 

    fn parse_sq_block (&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        loop {
            if let Some(token) = self.peek() {
                match token {
                    Token::RSqr => {
                        self.next();
                        self.skip();
                        break;
                    }
                    _ => {
                        let stmt = self.parse_stmt(0)?;
                        match *stmt {
                            Stmt::Empty => {}
                            _ => {
                                stmts.push(*stmt);
                            }
                        }
                    }
                }
            }
        }
        Ok(stmts)
    } 

}
