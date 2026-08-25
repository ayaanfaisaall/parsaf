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
    Break,
    Empty,
    NotImplYet,
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
            Some(Token::RBrc) | Some(Token::Pipe) |
            Some(Token::LBrc) => {
                Ok(())
            }
            _ => {
                Err(format!("parsaf: unexpected token found: {:?}", token))
            }
        }
    }
    
    fn parse (&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        loop {
            if let Some(token) = self.peek() {
                match token {
                    Token::EOF => {
                        break;
                    }
                    _ => {
                        let stmt = self.parse_stmt()?;
                        match self.peek() {
                            Some(Token::Pipe) => {
                                self.next();
                                let pipe = self.parse_pipeline(Some(stmt))?;
                                stmts.push(*pipe);
                            }
                            _ => {
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
            Some(Token::NewLine) |
            Some(Token::SemiCln) => {
                self.next();
                Ok(Box::new(Stmt::Empty))
            }
            Some(Token::Pipe) => {
                Err(format!("parsaf: token: {:?} not allowed in start", token))
            }
            Some(Token::Word(_)) => {
                self.parse_cmd()
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
                Token::RBrc    |
                Token::LBrc    |
                Token::Pipe => {
                    self.skip();
                    break;
                }
                _ => {
                    let arg = self.parse_base_case()?;
                    args.push(*arg);
                }
            }
        }
        self.skip();
        Ok(Box::new(Stmt::Cmd { cmd , args }))
    }

    fn parse_pipeline (&mut self, cmd: Option<Box<Stmt>>) -> Result<Box<Stmt>, String> {
        let mut commands = Vec::new();
        match cmd {
            Some(c) => commands.push(*c),
            None => {}
        }
        loop {
            let next_cmd = self.parse_stmt()?;
            commands.push(*next_cmd);
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
                _ => {
                    break;
                }
            }
        }
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
        loop {
            if let Some(token) = self.peek() {
                match token {
                    Token::RBrc => {
                        self.next();
                        self.skip();
                        break;
                    }
                    _ => {
                        let stmt = self.parse_stmt()?;
                        match self.peek() {
                            Some(Token::Pipe) => {
                                self.next();
                                let pipe = self.parse_pipeline(Some(stmt))?;
                                stmts.push(*pipe);
                            }
                            _ => {
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
            }
        }
        Ok(stmts)
    } 
}

fn main() {
    let name = String::from(r#" cmd this | cmd that | these those
                                let b = "ayaan"
                                let a = 48; 
                                let a = 58
                                echo "{a}" | tr "a-z" "A-Z"
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
                                if let a = "{cat ~/parsaf/src/main.rs}" { echo "{a}" | echo true } | tr "a-z" "A-Z" | runitctl enable sshd | theme 3 
                                if theme 3 { echo | this } | for i in 10 to 39 { print "{a}" } | runitctl enable sshd
                                while let a = "{curl https://ayaanfaisaall.cc/downloads/cv.pdf}" {
                                    print true
                                    echo true
                                    break
                                }
                                || &&
                                "#);

    let tokens = Lexer::new(&name).tokenize();
    println!("{:?}", tokens);

    let ast = Parser::new(&tokens).parse();
    println!("{:#?}", ast);
}
