use lexaf::{
    Lexer,
    Token,
    StrIntr,
};

#[derive(Debug, Clone, PartialEq)]
enum Stmt {
    //
    // base_case: the statements which consists a value,
    //
    Word(String),
    Str(Vec<StrIntr>),
    ExitCode(u8),
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
        pipe: Vec<Stmt>,
    },
    NotImplYet,
    Break,
    Empty,
}

#[derive(Debug)]
struct Parser <'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl <'a> Parser <'a> {
    fn new (tokens: &'a [Token]) -> Self {
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

    fn expect (&mut self, expected: Token) -> Result<(),String> {
        if self.peek() == Some(&expected) {
            self.next();
            Ok(())
        } else {
            Err(format!("parsaf: expected: {:?}, found: {:?}", expected, self.peek()))
        } 
    }

    fn parse (&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        while let Some(token) = self.peek() {
            if token == &Token::EOF {
                break;
            }
            let stmt = self.parse_stmt()?;
            stmts.push(*stmt);
        }
        Ok(stmts)
    }

    fn parse_stmt (&mut self) -> Result<Box<Stmt>, String> {
        let token = self.peek();
        match token {
            Some(Token::True) | Some(Token::False) => {
                self.parse_base_case()
            }
            Some(Token::Print) => {
                self.parse_print_stmt()
            }
            Some(Token::Let) => {
                self.parse_let_stmt()
            }
            Some(Token::If) => {
                self.parse_if_stmt()
            }
            Some(Token::While) => {
                self.parse_while_stmt()
            }
            Some(Token::For) => {
                self.parse_for_stmt()
            }
            _ => {
                self.next();
                Ok(Box::new(Stmt::NotImplYet))
            }
        }
    }

    fn parse_print_stmt(&mut self) -> Result<Box<Stmt>, String> {
        self.next();
        let value = self.parse_base_case()?;
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
        return Ok(Box::new(Stmt::Let { var: name, val: value }))
    }
    
    fn parse_if_stmt (&mut self) -> Result<Box<Stmt>, String> {
        self.next();
        let condition = self.parse_stmt()?;
        self.expect(Token::LBrc)?;
        let block = self.parse_block()?;
        let mut alternate = None;
        if let token = self.peek() {
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
        }
        Ok(Box::new(Stmt::If { cond: condition, block: block, alter: alternate }))
    }

    fn parse_while_stmt (&mut self) -> Result<Box<Stmt>, String> {
        self.next();
        let condition = self.parse_stmt()?;
        self.expect(Token::LBrc)?;
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
        let block = self.parse_block()?;
        Ok(Box::new(Stmt::For { iter, start, end, block }))
    }

    fn parse_block (&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        while let Some(token) = self.peek() {
            match token {
                Token::RBrc => {
                    self.next();
                    break;
                }
                _ => {
                    let stmt = self.parse_stmt()?;
                    stmts.push(*stmt);
                }
            }
        }
        Ok(stmts)
    } 

    fn parse_base_case (&mut self) -> Result<Box<Stmt>, String> {
        let token = self.next();
        match token {
            Some(Token::Word(w)) => {
                Ok(Box::new(Stmt::Word(w.clone()))) 
            }
            Some(Token::Str(s)) => {
                Ok(Box::new(Stmt::Str(s.clone())))
            }
            Some(Token::True) => {
                Ok(Box::new(Stmt::ExitCode(0)))
            }
            Some(Token::False) => {
                Ok(Box::new(Stmt::ExitCode(1)))
            }
            _ => {
                Err(format!("parsaf: expected base_case, found: {:?}", token))
            }
        }
    }
   
}

fn main() {
    let name = String::from(r#" let b = "ayaan"
                                if let a = "my name is {b}" {
                                    print "{a}"
                                    print true 
                                    print this
                                } else {
                                    print false
                                }
                                if true {
                                    print true
                                } elif false {
                                    print false
                                } else {
                                    print "i did it!"
                                }
                                while true {
                                    print this
                                }
                                for i in 1 to 10 {
                                    print ayaan
                                }"#);
    let tokens = Lexer::new(&name).tokenize();
    println!("{:?}", tokens);

    let ast = Parser::new(&tokens).parse();
    println!("{:#?}", ast);
}
