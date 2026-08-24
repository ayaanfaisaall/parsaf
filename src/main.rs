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

    fn skip (&mut self) {
        let to_be_skipped = self.peek();
        match to_be_skipped {
            Some(Token::NewLine) | Some(Token::SemiCln) => {
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
    
    fn parse (&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        while let Some(token) = self.peek() {
            if token == &Token::EOF {
                break;
            } 
            let stmt = self.parse_stmt()?;
            match *stmt {
                Stmt::Empty => {}
                _ => {
                    stmts.push(*stmt);
                }
            }
        }
        Ok(stmts)
    }

    fn parse_stmt (&mut self) -> Result<Box<Stmt>, String> {
        let token = self.peek();
        match token {
            Some(Token::True)  | 
            Some(Token::False) |
            Some(Token::Str(_))|
            Some(Token::Num(_)) => {
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
            Some(Token::LBrc) => {
                self.next();
                Ok(Box::new(Stmt::Block { block: self.parse_block()? }))
            }
            Some(Token::Break) => {
                self.next();
                Ok(Box::new(Stmt::Break))
            }
            Some(Token::NewLine) => {
                self.next();
                Ok(Box::new(Stmt::Empty))
            }
            _ => {
                self.parse_cmd()
            }
        }
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
        self.skip();
        return Ok(Box::new(Stmt::Let { var: name, val: value }))
    }
    
    fn parse_if_stmt (&mut self) -> Result<Box<Stmt>, String> {
        self.next();
        let condition = self.parse_stmt()?;
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
        let condition = self.parse_stmt()?;
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
                Token::NewLine | 
                Token::SemiCln |
                Token::RBrc     => {
                    self.skip();
                    break;
                }
                Token::Pipe => {
                    let command = Box::new(Stmt::Cmd { cmd , args });
                    return self.parse_pipeline(command);
                }
                _ => {
                    let arg = self.parse_base_case()?;
                    args.push(*arg);
                }
            }
        }
        Ok(Box::new(Stmt::Cmd { cmd , args }))
    }

    fn parse_pipeline (&mut self, cmd: Box<Stmt>) -> Result<Box<Stmt>, String> {
        self.next();
        let mut commands = Vec::new();
        commands.push(*cmd);
        let next_cmd = self.parse_stmt()?;
        commands.push(*next_cmd);
        Ok(Box::new(Stmt::Pipe { pipe: commands }))
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
            _ => {
                Err(format!("parsaf: expected base_case, found: {:?}", token))
            }
        }
    }

    fn parse_block (&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        while let Some(token) = self.peek() {
            match token {
                Token::RBrc => {
                    self.next();
                    self.skip();
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
   
}

fn main() {
    let name = String::from(r#" "my name is {t}"
                                true
                                88
                                let b = "ayaan"
                                let a = 48
                                if let a = "my name is {b}" {
                                    print "{a}"
                                    print true 
                                    print this
                                } else {
                                    print false
                                }
                                {
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
                                    }
                                }
                                theme 18
                                runitctl enable sshd | theme 28 | echo true | if let a = 38 { echo true }
                                "#);

    let tokens = Lexer::new(&name).tokenize();
    println!("{:?}", tokens);

    let ast = Parser::new(&tokens).parse();
    println!("{:#?}", ast);
}
