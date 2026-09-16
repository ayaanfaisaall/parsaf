#[cfg(test)]
mod tests;

use miette::Report;
use::lexaf::{
    Lexer,
};
use::parsaf::parser::{
    Parser,
};

fn main() {
    let name = String::from(r#" cmd this | cmd that | these those
                                8888
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
                                if let a = {cat ~/parsaf/src/main.rs} { echo "{a}" | echo true } | tr "a-z" "A-Z" | runitctl enable sshd | theme 3 
                                if theme 3 { echo | this } | for i in 10 to 39 { print "{a}" } | runitctl enable sshd
                                while let a = {curl https://ayaanfaisaall.cc/downloads/cv.pdf} {
                                    print true
                                    echo true
                                    break
                                }
                                !38
                                let a = 3 && print a || test "{a}" -eq 8 && history | grep -i fd
                                   # &
                                cmd arg1 arg2 &
                                # this is the best use of if let: the command will carry out the moment let is evaluated in ast, if it fails 
                                # it will not throw an error, balky let ka exit code 1 hojiay ga aur jese hi let ka exit code 1 hoga to if 
                                # will be failed because 0 is true and 1 is false, 
                                if let a = {cat ~/parsaf/src/ast.rs} {
                                    print a
                                } else {
                                    print "cat failed: error: file might not be present"
                                } 
                                "#);
    let mut lexer = Lexer::new(&name);
    match lexer.tokenize() {
        Ok(tokens) => {
            let mut parser = Parser::new(&tokens);
            match parser.parse() {
                Ok(ast) => println!("{:#?}", ast),
                Err(e) => {
                    let error = Report::new(e).with_source_code(name.to_string());
                    println!("{:?}", error);
                }
            }
        }
        Err(e) => {
            let error = Report::new(e).with_source_code(name.to_string());
            println!("{:?}", error);
        }
    }

}

