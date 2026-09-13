use lexaf::{
    Token,
};
use crate::ast::{
    Stmt,
};

/// A Recursive Descent Parser for the custom shell language.
/// It processes a slice of `Token`s and constructs an Abstract Syntax Tree (AST).
/// 
/// Precedence Hierarchy (Top to Bottom):
/// 1. Logical Operators (`&&`, `||`) -> `parse_andor`
/// 2. Pipes (`|`) -> `parse_pipeline`
/// 3. Statements & Commands -> `parse_stmt`
pub struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    /// Creates a new Parser instance from a slice of tokens.
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser {
            tokens,
            pos: 0,
        }
    }

    /// Returns a reference to the current token without advancing the pointer.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    /// Advances the pointer and returns the current token.
    /// Safely prevents the pointer from incrementing beyond EOF (`None`).
    fn next(&mut self) -> Option<&Token> {
        let next = self.tokens.get(self.pos);
        if next.is_some() {
            self.pos += 1;
        }
        next
    }

    /// Skips purely structural tokens like NewLines and Semicolons.
    fn skip(&mut self) {
        let to_be_skipped = self.peek();
        match to_be_skipped {
            Some(Token::NewLine) |
            Some(Token::SemiCln) => {
                self.next();
            }
            _ => {} 
        }
    }

    /// Asserts that the next token matches the expected one, advancing if true.
    /// Returns an error if the expected token is not found.
    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if self.peek() == Some(&expected) {
            self.next();
            Ok(())
        } else {
            Err(format!("parsaf: expected: {:?}, found: {:?}", expected, self.peek()))
        } 
    }

    /// Validates that the current token is a safe boundary/terminator.
    fn unexpected(&mut self) -> Result<(), String> {
        let token = self.peek();
        match token {
            Some(Token::NewLine) | Some(Token::SemiCln) |
            Some(Token::RBrc) | Some(Token::Pipe)   |
            Some(Token::LBrc) | Some(Token::AndAnd) | 
            Some(Token::OrOr) | Some(Token::RSqr)   |
            Some(Token::LSqr) | Some(Token::EOF) => {
                Ok(())
            }
            _ => {
                Err(format!("parsaf: unexpected token found: {:?}", token))
            }
        }
    }
    
    /// Main entry point for the parser.
    /// Loops through tokens until EOF, evaluating top-level expressions.
    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        loop {
            if let Some(token) = self.peek() {
                match token {
                    Token::EOF => {
                        break;
                    }
                    _ => {
                        // Start evaluating from the highest precedence (And/Or)
                        let stmt = self.parse_andor()?;
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

    /// Routes the evaluation to the appropriate statement type (If, While, Let, Cmd).
    /// This represents the highest precedence (single command block) in the AST.
    fn parse_stmt(&mut self) -> Result<Box<Stmt>, String> {
        let token = self.peek();
        
        match token {
            Some(Token::True)  | 
            Some(Token::False) |
            Some(Token::Str(_))|
            Some(Token::LSqr)  |
            Some(Token::LBrc)  |
            Some(Token::Num(_)) => {
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
            Some(Token::NewLine) |
            Some(Token::SemiCln) => {
                self.next();
                Ok(Box::new(Stmt::Empty))
            }
            Some(Token::Pipe) |
            Some(Token::And)  |
            Some(Token::OrOr) |
            Some(Token::AndAnd) => {
                Err(format!("parsaf: token: {:?} not allowed in start", token))
            }
            Some(Token::Bang) => {
                self.next();
                match self.next() {
                    Some(Token::Num(n)) => {
                        Ok(Box::new(Stmt::Bang { num: Box::new(Stmt::Num(n.clone())) }))
                    }
                    _ => Err(format!("parsaf: expected number, found: {:?}", self.peek()))
                }
            }
            Some(Token::Word(_)) => self.parse_cmd(),
            _ => {
                self.next();
                Ok(Box::new(Stmt::NotImplYet))
            }
        }
    }

    fn parse_print_stmt(&mut self) -> Result<Box<Stmt>, String> {
        self.next();
        let value = self.parse_base_case()?;
        self.skip();
        return Ok(Box::new(Stmt::Print { val: value }))
    }

    fn parse_let_stmt(&mut self) -> Result<Box<Stmt>, String> {
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
    
    /// Parses an `if` statement.
    /// Evaluates the condition using `parse_andor` to support logical operators inside conditions.
    fn parse_if_stmt(&mut self) -> Result<Box<Stmt>, String> {
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

    fn parse_while_stmt(&mut self) -> Result<Box<Stmt>, String> {
        self.next();
        let condition = self.parse_andor()?;
        self.expect(Token::LBrc)?;
        self.skip();
        let block = self.parse_block()?;
        Ok(Box::new(Stmt::While { cond: condition, block: block }))
    }

    fn parse_for_stmt(&mut self) -> Result<Box<Stmt>, String> {
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

    /// Parses a standard shell command and its arguments.
    /// Handles background operators `&` and stops at safe delimiters.
    fn parse_cmd(&mut self) -> Result<Box<Stmt>, String> {
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

    /// Parses pipelines (`a | b`).
    /// Precedence Level: Middle (Evaluated after And/Or, before single Stmt).
    /// Safely avoids allocating a Vec/Pipe node if only one command is found.
    fn parse_pipeline(&mut self) -> Result<Box<Stmt>, String> {
        let stmt = self.parse_stmt()?;
        
        // If there's no pipe, just return the single statement safely (0 unwraps, 0 panics)
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

    /// Parses logical `&&` and `||` operations.
    /// Precedence Level: Lowest (Top of the AST chain).
    /// Uses inner loops to flatten multiple consecutive operators of the same type 
    /// (e.g., `a && b && c`) into a single node array, avoiding deep nested recursion trees.
    fn parse_andor(&mut self) -> Result<Box<Stmt>, String> {
        let mut stmt = self.parse_pipeline()?;
        
        loop {
            match self.peek() {
                Some(Token::AndAnd) => {
                    self.next(); 
                    let mut stmts = vec![*stmt];
                    stmts.push(*self.parse_pipeline()?);
                    
                    // Keep consuming && to keep the AST array completely flat
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
                    
                    // Keep consuming || to keep the AST array completely flat
                    while let Some(Token::OrOr) = self.peek() {
                        self.next(); 
                        stmts.push(*self.parse_pipeline()?);
                    }
                    stmt = Box::new(Stmt::OrOr { stmts });
                }
                _ => {
                    break;
                }
            }
        }
        Ok(stmt)
    }

    /// Evaluates base primitives like literals, variables, sub-blocks, and arrays.
    fn parse_base_case(&mut self) -> Result<Box<Stmt>, String> {
        let token = self.next();
        match token {
            Some(Token::Word(w)) => Ok(Box::new(Stmt::Word(w.to_string()))),
            Some(Token::Num(n)) => Ok(Box::new(Stmt::Num(n.clone()))),
            Some(Token::Str(s)) => Ok(Box::new(Stmt::Str(s.clone()))),
            Some(Token::True) => Ok(Box::new(Stmt::Bool(true))),
            Some(Token::False) => Ok(Box::new(Stmt::Bool(false))),
            Some(Token::LBrc) => Ok(Box::new(Stmt::Block { block: self.parse_block()? })),
            Some(Token::LSqr) => return self.parse_arrays(),
            _ => Err(format!("parsaf: expected base_case, found: {:?}", token))
        }
    }

    /// Parses a block of statements enclosed in `{ }`.
    /// Re-enters the parser via `parse_andor` to allow full expressions inside blocks.
    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        loop {
            if let Some(token) = self.peek() {
                match token {
                    Token::RBrc => {
                        self.next();
                        break;
                    }
                    Token::EOF => {
                        return Err(format!("parsaf: expected '}}', found: {:?}", token))
                    }
                    _ => {
                        // Re-enter top-level evaluation for statements inside the block
                        let stmt = self.parse_andor()?;
                        match *stmt {
                            Stmt::Empty => {}
                            _ => stmts.push(*stmt),
                        }
                    }
                }
            }
        }
        Ok(stmts)
    } 

    /// Parses a comma or space-separated array enclosed in `[ ]`.
    fn parse_arrays(&mut self) -> Result<Box<Stmt>, String> {
        let mut stmts = Vec::new();
        loop {
            let token = self.peek();
            match token {
                Some(Token::Comma) => {
                    self.next();
                }
                Some(Token::RSqr) => {
                    self.next();
                    break;
                }
                Some(Token::NewLine) => {
                    self.skip();
                }
                Some(Token::EOF) => {
                    return Err(format!("parsaf: expected ']', found: {:?}", token))
                }
                _ => {
                    let base_case = self.parse_base_case()?;
                    stmts.push(*base_case);
                }
            }
        }
        Ok(Box::new(Stmt::Array(stmts)))
    }
}
