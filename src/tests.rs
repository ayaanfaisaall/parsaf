#[cfg(test)]
mod tests {
    use lexaf::Lexer;
    use lexaf::tokens::StrIntr;
    use parsaf::{Parser, Span, SpannedStmt, Stmt};

    #[test]
    fn test_variable_declaration() {
        let input = "let n1 = 43\n";
        let tokens = Lexer::new(input).tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            vec![SpannedStmt {
                stmt: Stmt::Let {
                    var: "n1",
                    val: Box::new(SpannedStmt {
                        stmt: Stmt::Num(43),
                        span: Span { start: 9, end: 11 }
                    }),
                },
                span: Span { start: 0, end: 11 }
            }]
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
            vec![SpannedStmt {
                stmt: Stmt::Cmd {
                    cmd: Box::new(SpannedStmt {
                        stmt: Stmt::Word("git"),
                        span: Span { start: 0, end: 3 }
                    }),
                    args: vec![
                        SpannedStmt {
                            stmt: Stmt::Word("add"),
                            span: Span { start: 4, end: 7 }
                        },
                        SpannedStmt {
                            stmt: Stmt::Word("."),
                            span: Span { start: 8, end: 9 }
                        },
                    ],
                },
                span: Span { start: 0, end: 9 }
            }]
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
            vec![SpannedStmt {
                stmt: Stmt::Pipe {
                    stmts: vec![
                        SpannedStmt {
                            stmt: Stmt::Cmd {
                                cmd: Box::new(SpannedStmt {
                                    stmt: Stmt::Word("cat"),
                                    span: Span { start: 0, end: 3 }
                                }),
                                args: vec![SpannedStmt {
                                    stmt: Stmt::Word("~/Downloads/abc/dc.jpg"),
                                    span: Span { start: 4, end: 26 }
                                }],
                            },
                            span: Span { start: 0, end: 26 }
                        },
                        SpannedStmt {
                            stmt: Stmt::Cmd {
                                cmd: Box::new(SpannedStmt {
                                    stmt: Stmt::Word("grep"),
                                    span: Span { start: 29, end: 33 }
                                }),
                                args: vec![SpannedStmt {
                                    stmt: Stmt::Word("abc"),
                                    span: Span { start: 34, end: 37 }
                                }],
                            },
                            span: Span { start: 29, end: 37 }
                        }
                    ],
                },
                span: Span { start: 0, end: 37 }
            }]
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
            vec![SpannedStmt {
                stmt: Stmt::For {
                    iter: "i",
                    start: Box::new(SpannedStmt {
                        stmt: Stmt::Num(0),
                        span: Span { start: 9, end: 10 }
                    }),
                    end: Box::new(SpannedStmt {
                        stmt: Stmt::Num(10),
                        span: Span { start: 14, end: 16 }
                    }),
                    block: vec![SpannedStmt {
                        stmt: Stmt::Break,
                        span: Span { start: 19, end: 24 }
                    }],
                },
                span: Span { start: 0, end: 26 }
            }]
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
                SpannedStmt {
                    stmt: Stmt::While {
                        cond: Box::new(SpannedStmt {
                            stmt: Stmt::Bool(true),
                            span: Span { start: 6, end: 10 }
                        }),
                        block: vec![SpannedStmt {
                            stmt: Stmt::Break,
                            span: Span { start: 13, end: 18 }
                        }],
                    },
                    span: Span { start: 0, end: 20 }
                },
                SpannedStmt {
                    stmt: Stmt::While {
                        cond: Box::new(SpannedStmt {
                            stmt: Stmt::Bool(false),
                            span: Span { start: 27, end: 32 }
                        }),
                        block: vec![],
                    },
                    span: Span { start: 21, end: 36 }
                }
            ]
        );
    }

    #[test]
    fn test_if_elif_else_flow() {
        let input =
            "if true { print \"yes\" } elif false { print \"no\" } else { print \"maybe\" }";
        let tokens = Lexer::new(input).tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            vec![SpannedStmt {
                stmt: Stmt::If {
                    cond: Box::new(SpannedStmt {
                        stmt: Stmt::Bool(true),
                        span: Span { start: 3, end: 7 }
                    }),
                    block: vec![SpannedStmt {
                        stmt: Stmt::Print {
                            val: Box::new(SpannedStmt {
                                stmt: Stmt::Str(&vec![StrIntr::Literal("yes")]),
                                span: Span { start: 16, end: 21 }
                            }),
                        },
                        span: Span { start: 10, end: 21 }
                    }],
                    alter: Some(Box::new(SpannedStmt {
                        stmt: Stmt::If {
                            cond: Box::new(SpannedStmt {
                                stmt: Stmt::Bool(false),
                                span: Span { start: 29, end: 34 }
                            }),
                            block: vec![SpannedStmt {
                                stmt: Stmt::Print {
                                    val: Box::new(SpannedStmt {
                                        stmt: Stmt::Str(&vec![StrIntr::Literal("no")]),
                                        span: Span { start: 43, end: 47 }
                                    }),
                                },
                                span: Span { start: 37, end: 47 }
                            }],
                            alter: Some(Box::new(SpannedStmt {
                                stmt: Stmt::Block {
                                    block: vec![SpannedStmt {
                                        stmt: Stmt::Print {
                                            val: Box::new(SpannedStmt {
                                                stmt: Stmt::Str(&vec![StrIntr::Literal("maybe")]),
                                                span: Span { start: 63, end: 70 }
                                            }),
                                        },
                                        span: Span { start: 57, end: 70 }
                                    }]
                                },
                                span: Span { start: 55, end: 72 }
                            })),
                        },
                        span: Span { start: 24, end: 72 }
                    })),
                },
                span: Span { start: 0, end: 72 }
            }]
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
            vec![SpannedStmt {
                stmt: Stmt::OrOr {
                    stmts: vec![
                        SpannedStmt {
                            stmt: Stmt::AndAnd {
                                stmts: vec![
                                    SpannedStmt {
                                        stmt: Stmt::Cmd {
                                            cmd: Box::new(SpannedStmt {
                                                stmt: Stmt::Word("cmd1"),
                                                span: Span { start: 0, end: 4 }
                                            }),
                                            args: vec![],
                                        },
                                        span: Span { start: 0, end: 4 }
                                    },
                                    SpannedStmt {
                                        stmt: Stmt::Cmd {
                                            cmd: Box::new(SpannedStmt {
                                                stmt: Stmt::Word("cmd2"),
                                                span: Span { start: 8, end: 12 }
                                            }),
                                            args: vec![],
                                        },
                                        span: Span { start: 8, end: 12 }
                                    }
                                ]
                            },
                            span: Span { start: 0, end: 12 }
                        },
                        SpannedStmt {
                            stmt: Stmt::Cmd {
                                cmd: Box::new(SpannedStmt {
                                    stmt: Stmt::Word("cmd3"),
                                    span: Span { start: 16, end: 20 }
                                }),
                                args: vec![],
                            },
                            span: Span { start: 16, end: 20 }
                        }
                    ]
                },
                span: Span { start: 0, end: 20 }
            }]
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
            vec![SpannedStmt {
                stmt: Stmt::AndAnd {
                    stmts: vec![
                        SpannedStmt {
                            stmt: Stmt::Bang {
                                num: Box::new(SpannedStmt {
                                    stmt: Stmt::Num(38),
                                    span: Span { start: 1, end: 3 }
                                }),
                            },
                            span: Span { start: 0, end: 3 }
                        },
                        SpannedStmt {
                            stmt: Stmt::And {
                                cmd: Box::new(SpannedStmt {
                                    stmt: Stmt::Cmd {
                                        cmd: Box::new(SpannedStmt {
                                            stmt: Stmt::Word("server"),
                                            span: Span { start: 7, end: 13 }
                                        }),
                                        args: vec![SpannedStmt {
                                            stmt: Stmt::Word("start"),
                                            span: Span { start: 14, end: 19 }
                                        }],
                                    },
                                    span: Span { start: 7, end: 19 }
                                }),
                            },
                            span: Span { start: 7, end: 21 }
                        }
                    ]
                },
                span: Span { start: 0, end: 21 }
            }]
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
            vec![SpannedStmt {
                stmt: Stmt::If {
                    cond: Box::new(SpannedStmt {
                        stmt: Stmt::Let {
                            var: "a",
                            val: Box::new(SpannedStmt {
                                stmt: Stmt::Block {
                                    block: vec![SpannedStmt {
                                        stmt: Stmt::Cmd {
                                            cmd: Box::new(SpannedStmt {
                                                stmt: Stmt::Word("cat"),
                                                span: Span { start: 13, end: 16 }
                                            }),
                                            args: vec![SpannedStmt {
                                                stmt: Stmt::Word("main.rs"),
                                                span: Span { start: 17, end: 24 }
                                            }],
                                        },
                                        span: Span { start: 13, end: 24 }
                                    }]
                                },
                                span: Span { start: 11, end: 26 }
                            })
                        },
                        span: Span { start: 3, end: 26 }
                    }),
                    block: vec![SpannedStmt {
                        stmt: Stmt::Print {
                            val: Box::new(SpannedStmt {
                                stmt: Stmt::Word("a"),
                                span: Span { start: 35, end: 36 }
                            }),
                        },
                        span: Span { start: 29, end: 36 }
                    }],
                    alter: None,
                },
                span: Span { start: 0, end: 38 }
            }]
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
            vec![SpannedStmt {
                stmt: Stmt::AndAnd {
                    stmts: vec![
                        SpannedStmt {
                            stmt: Stmt::OrOr {
                                stmts: vec![
                                    SpannedStmt {
                                        stmt: Stmt::AndAnd {
                                            stmts: vec![
                                                SpannedStmt {
                                                    stmt: Stmt::Let {
                                                        var: "a",
                                                        val: Box::new(SpannedStmt {
                                                            stmt: Stmt::Num(3),
                                                            span: Span { start: 8, end: 9 }
                                                        }),
                                                    },
                                                    span: Span { start: 0, end: 9 }
                                                },
                                                SpannedStmt {
                                                    stmt: Stmt::Print {
                                                        val: Box::new(SpannedStmt {
                                                            stmt: Stmt::Word("a"),
                                                            span: Span { start: 19, end: 20 }
                                                        }),
                                                    },
                                                    span: Span { start: 13, end: 20 }
                                                },
                                            ],
                                        },
                                        span: Span { start: 0, end: 20 }
                                    },
                                    SpannedStmt {
                                        stmt: Stmt::Cmd {
                                            cmd: Box::new(SpannedStmt {
                                                stmt: Stmt::Word("test"),
                                                span: Span { start: 24, end: 28 }
                                            }),
                                            args: vec![
                                                SpannedStmt {
                                                    stmt: Stmt::Str(&vec![StrIntr::Variable("a"),]),
                                                    span: Span { start: 29, end: 34 }
                                                },
                                                SpannedStmt {
                                                    stmt: Stmt::Word("-eq"),
                                                    span: Span { start: 35, end: 38 }
                                                },
                                                SpannedStmt {
                                                    stmt: Stmt::Num(8),
                                                    span: Span { start: 39, end: 40 }
                                                },
                                            ],
                                        },
                                        span: Span { start: 24, end: 40 }
                                    },
                                ],
                            },
                            span: Span { start: 0, end: 40 }
                        },
                        SpannedStmt {
                            stmt: Stmt::Pipe {
                                stmts: vec![
                                    SpannedStmt {
                                        stmt: Stmt::Cmd {
                                            cmd: Box::new(SpannedStmt {
                                                stmt: Stmt::Word("history"),
                                                span: Span { start: 44, end: 51 }
                                            }),
                                            args: vec![],
                                        },
                                        span: Span { start: 44, end: 51 }
                                    },
                                    SpannedStmt {
                                        stmt: Stmt::Cmd {
                                            cmd: Box::new(SpannedStmt {
                                                stmt: Stmt::Word("grep"),
                                                span: Span { start: 54, end: 58 }
                                            }),
                                            args: vec![
                                                SpannedStmt {
                                                    stmt: Stmt::Word("-i"),
                                                    span: Span { start: 59, end: 61 }
                                                },
                                                SpannedStmt {
                                                    stmt: Stmt::Word("fd"),
                                                    span: Span { start: 62, end: 64 }
                                                },
                                            ],
                                        },
                                        span: Span { start: 54, end: 64 }
                                    },
                                ],
                            },
                            span: Span { start: 44, end: 64 }
                        },
                    ],
                },
                span: Span { start: 0, end: 64 }
            }]
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
            vec![SpannedStmt {
                stmt: Stmt::Array(vec![
                    SpannedStmt {
                        stmt: Stmt::Num(1),
                        span: Span { start: 1, end: 2 }
                    },
                    SpannedStmt {
                        stmt: Stmt::Num(2),
                        span: Span { start: 4, end: 5 }
                    },
                    SpannedStmt {
                        stmt: Stmt::Num(3),
                        span: Span { start: 7, end: 8 }
                    },
                ]),
                span: Span { start: 0, end: 9 }
            }]
        );
    }
}
