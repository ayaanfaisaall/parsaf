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
                                if let a = "{cat ~/parsaf/src/main.rs}" { echo "{a}" | echo true } | tr "a-z" "A-Z" | runitctl enable sshd | theme 3 
                                if theme 3 { echo | this } | for i in 10 to 39 { print "{a}" } | runitctl enable sshd
                                while let a = "{curl https://ayaanfaisaall.cc/downloads/cv.pdf}" {
                                    print true
                                    echo true
                                    break
                                }
                                !38
                                let a = 3 && echo a || test a -eq 8 && history | grep -i fd
                                   # &
                                cmd arg1 arg2 &
                                "#);

    let tokens = Lexer::new(&name).tokenize();
    println!("{:?}", tokens);

    let ast = Parser::new(&tokens).parse();
    println!("{:#?}", ast);
}

