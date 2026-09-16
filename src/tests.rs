#[cfg(test)]
mod tests {
    use parsaf::{
        Stmt,
        Parser
    };
    use lexaf::{Lexer};
    use lexaf::tokens::StrIntr;

    #[test]
    fn test_variable_declaration() {
        let input = "let n1 = 43\n";
        let tokens = Lexer::new(input).tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            vec![
                Stmt::Let {
                    var: String::from("n1"),
                    val: Box::new(Stmt::Num(43)),
                }
            ]
        );
    }

    #[test]
    fn test_simple_command_with_args() {
        let input = "git add .";
        let tokens = Lexer::new(input).tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            vec![
                Stmt::Cmd {
                    cmd: Box::new(Stmt::Word(String::from("git"))),
                    args: vec![
                        Stmt::Word(String::from("add")),
                        Stmt::Word(String::from(".")),
                    ],
                }
            ]
        );
    }

    #[test]
    fn test_pipeline_execution() {
        let input = "cat ~/Downloads/abc/dc.jpg | grep abc";
        let tokens = Lexer::new(input).tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            vec![
                Stmt::Pipe {
                    stmts: vec![
                        Stmt::Cmd {
                            cmd: Box::new(Stmt::Word(String::from("cat"))),
                            args: vec![Stmt::Word(String::from("~/Downloads/abc/dc.jpg"))],
                        },
                        Stmt::Cmd {
                            cmd: Box::new(Stmt::Word(String::from("grep"))),
                            args: vec![Stmt::Word(String::from("abc"))],
                        }
                    ],
                }
            ]
        );
    }

    #[test]
    fn test_for_loop_with_to() {
        let input = "for i in 0 to 10 { break }";
        let tokens = Lexer::new(input).tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            vec![
                Stmt::For {
                    iter: String::from("i"),
                    start: Box::new(Stmt::Num(0)),
                    end: Box::new(Stmt::Num(10)),
                    block: vec![Stmt::Break],
                }
            ]
        );
    }

    #[test]
    fn test_while_loop_with_booleans() {
        let input = "while true { break } while false { }";
        let tokens = Lexer::new(input).tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            vec![
                Stmt::While {
                    cond: Box::new(Stmt::Bool(true)),
                    block: vec![Stmt::Break],
                },
                Stmt::While {
                    cond: Box::new(Stmt::Bool(false)),
                    block: vec![],
                }
            ]
        );
    }

    #[test]
    fn test_if_elif_else_flow() {
        let input = "if true { print \"yes\" } elif false { print \"no\" } else { print \"maybe\" }";
        let tokens = Lexer::new(input).tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            vec![
                Stmt::If {
                    cond: Box::new(Stmt::Bool(true)),
                    block: vec![
                        Stmt::Print {
                            val: Box::new(Stmt::Str(vec![StrIntr::Literal(String::from("yes"))])),
                        }
                    ],
                    alter: Some(Box::new(Stmt::If {
                        cond: Box::new(Stmt::Bool(false)),
                        block: vec![
                            Stmt::Print {
                                val: Box::new(Stmt::Str(vec![StrIntr::Literal(String::from("no"))])),
                            }
                        ],
                        alter: Some(Box::new(Stmt::Block {
                            block: vec![
                                Stmt::Print {
                                    val: Box::new(Stmt::Str(vec![StrIntr::Literal(String::from("maybe"))])),
                                }
                            ]
                        })),
                    })),
                }
            ]
        );
    }

    #[test]
    fn test_logical_and_or_chaining() {
        let input = "cmd1 && cmd2 || cmd3";
        let tokens = Lexer::new(input).tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            vec![
                Stmt::OrOr {
                    stmts: vec![
                        Stmt::AndAnd {
                            stmts: vec![
                                Stmt::Cmd {
                                    cmd: Box::new(Stmt::Word(String::from("cmd1"))),
                                    args: vec![],
                                },
                                Stmt::Cmd {
                                    cmd: Box::new(Stmt::Word(String::from("cmd2"))),
                                    args: vec![],
                                }
                            ]
                        },
                        Stmt::Cmd {
                            cmd: Box::new(Stmt::Word(String::from("cmd3"))),
                            args: vec![],
                        }
                    ]
                }
            ]
        );
    }

    #[test]
    fn test_background_operator_and_bang() {
        let input = "!38 && server start &";
        let tokens = Lexer::new(input).tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            vec![
                Stmt::AndAnd {
                    stmts: vec![
                        Stmt::Bang {
                            num: Box::new(Stmt::Num(38)),
                        },
                        Stmt::And {
                            cmd: Box::new(Stmt::Cmd {
                                cmd: Box::new(Stmt::Word(String::from("server"))),
                                args: vec![Stmt::Word(String::from("start"))],
                            }),
                        }
                    ]
                }
            ]
        );
    }

    #[test]
    fn test_block_assignment_in_if_let() {
        let input = "if let a = { cat main.rs } { print a }";
        let tokens = Lexer::new(input).tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            vec![
                Stmt::If {
                    cond: Box::new(Stmt::Let {
                        var: String::from("a"),
                        val: Box::new(Stmt::Block {
                            block: vec![
                                Stmt::Cmd {
                                    cmd: Box::new(Stmt::Word(String::from("cat"))),
                                    args: vec![Stmt::Word(String::from("main.rs"))],
                                }
                            ]
                        })
                    }),
                    block: vec![
                        Stmt::Print {
                            val: Box::new(Stmt::Word(String::from("a"))),
                        }
                    ],
                    alter: None,
                }
            ]
        );
    }

    #[test]
    fn test_and_or_and_pipes_together() {
        let input = "let a = 3 && print a || test \"{a}\" -eq 8 && history | grep -i fd";
        let tokens = Lexer::new(input).tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            vec![
                Stmt::AndAnd {
                    stmts: vec![
                        Stmt::OrOr {
                            stmts: vec![
                                Stmt::AndAnd {
                                    stmts: vec![
                                        Stmt::Let {
                                            var: String::from("a"),
                                            val: Box::new(Stmt::Num(3)),
                                        },
                                        Stmt::Print {
                                            val: Box::new(Stmt::Word(String::from("a"))),
                                        },
                                    ],
                                },
                                Stmt::Cmd {
                                    cmd: Box::new(Stmt::Word(String::from("test"))),
                                    args: vec![
                                        Stmt::Str(vec![
                                            StrIntr::Variable(String::from("a")),
                                        ]),
                                        Stmt::Word(String::from("-eq")),
                                        Stmt::Num(8),
                                    ],
                                },
                            ],
                        },
                        Stmt::Pipe {
                            stmts: vec![
                                Stmt::Cmd {
                                    cmd: Box::new(Stmt::Word(String::from("history"))),
                                    args: vec![],
                                },
                                Stmt::Cmd {
                                    cmd: Box::new(Stmt::Word(String::from("grep"))),
                                    args: vec![
                                        Stmt::Word(String::from("-i")),
                                        Stmt::Word(String::from("fd")),
                                    ],
                                },
                            ],
                        },
                    ],
                }
            ]
        );
    }

    #[test]
    fn test_arrays() {
        let input = "[1, 2, 3]";
        let tokens = Lexer::new(input).tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            vec![
                Stmt::Array(vec![
                    Stmt::Num(1),
                    Stmt::Num(2),
                    Stmt::Num(3),
                ])
            ]
        );
    }
}
