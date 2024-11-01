use std::fmt;

use crate::lexer::*;
use super::{syntax_err, unexpected_token_err, exit_failure};

pub type ExprIdx = usize;
pub type Block<'a> = Vec<Stmt<'a>>;



#[derive(Debug)]
pub struct Ast<'a> {
    pub fn_decls: Vec<FnDecl<'a>>,
}


#[derive(Debug)]
pub struct Stmt<'a> {
    pub loc: Loc,
    pub kind: StmtKind<'a>
}

#[derive(Debug)]
pub struct FnDecl<'a> {
    pub name: &'a str,
    pub params: Vec<&'a str>,
    pub has_result: bool,
    pub body: Block<'a>,
    pub loc: Loc,
}

#[derive(Debug)]
pub enum StmtKind<'a> {
    VarAssign { name: &'a str, expr: Expr },
    VarDecl(&'a str),
    ReturnVal(Expr),
    Return,
    If { cond: Expr, then: Block<'a>, elze: Block<'a> },
    For { cond: Expr, body: Block<'a> },
}

#[derive(Debug, Clone)]
pub struct ExprRange {
    pub loc: Loc,
    pub start: ExprIdx,
    pub end: ExprIdx,
}

// TODO: add loc to better error reporting
#[derive(Debug, PartialEq)]
pub enum Expr {
    FnCall { name: &'static str, args: Vec<Expr> },
    BinOp { lhs: Box<Expr>, rhs: Box<Expr>, op: BinOpKind },
    Var(&'static str),
    Num(i32),
}



pub fn parse<'a>(lex: &mut Lexer<'a>) -> Ast<'a> {
    let mut ast = Ast {
        fn_decls: Vec::new(),
    };

    while let Some(token) = lex.next_any() {
        // TODO: global variables
        if token != Token::Keyword(Keyword::Fn) {
            unexpected_token_err!(lex.loc, token);
        } 

        let loc = lex.loc.clone();
        let name = lex.expect_ident();

        // parameters
        lex.expect_punct(Punct::OpenParen);
        let mut params: Vec<&str> = Vec::new();
        match lex.expect_any() {
            Token::Punct(Punct::CloseParen) => {},
            Token::Ident(param_name) => {
                params.push(param_name);
                loop {
                    match lex.expect_any() {
                        Token::Punct(Punct::CloseParen) => break,
                        Token::Punct(Punct::Comma) => {
                            params.push(lex.expect_ident());
                        },
                        t @ _ => { unexpected_token_err!(lex.loc, t); }
                    }
                }
            },
            t @ _ => { unexpected_token_err!(lex.loc, t); }
        }

        // result
        let has_result = match lex.expect_peek_any() {
            Token::Punct(Punct::OpenCurly) => false,
            Token::Keyword(Keyword::Int) => {
                lex.next_any();
                true
            },
            t @ _ => { unexpected_token_err!(lex.loc, t); }
        };

        // body
        let body = parse_block(lex);

        ast.fn_decls.push(FnDecl {
            name, params, loc,
            has_result, body
        });
    }

    ast
}

fn parse_block<'a>( lex: &mut Lexer<'a>) -> Block<'a> {
    let mut block = Block::new();

    lex.expect_punct(Punct::OpenCurly);
    let mut token = lex.expect_any();
    loop {
        let loc = lex.loc.clone();
        match token {
            Token::Keyword(Keyword::For) => {
                block.push(Stmt {
                    loc, kind: StmtKind::For {
                        cond: parse_expr(lex, 0),
                        body: parse_block(lex),
                    }
                });
            },

            Token::Keyword(Keyword::If) => {
                let cond = parse_expr(lex, 0);
                let then = parse_block(lex);
                token = lex.expect_any();
                if let Token::Keyword(Keyword::Else) = token {
                    let elze = parse_block(lex);
                    block.push(Stmt {
                        loc,
                        kind: StmtKind::If {
                            cond, then, elze
                        }
                    });
                } else {
                    block.push(Stmt {
                        loc,
                        kind: StmtKind::If {
                            cond, then, elze: Vec::new(),
                        }
                    });
                    continue;
                }
            },

            Token::Keyword(Keyword::Return) => {
                if let Token::Punct(Punct::Semicolon) = lex.expect_peek_any() {
                    lex.next_any();
                    block.push(Stmt {
                        loc: lex.loc.clone(),
                        kind: StmtKind::Return
                    })
                } else {
                    block.push(Stmt {
                        loc: lex.loc.clone(),
                        kind: StmtKind::ReturnVal(parse_expr(lex, 0))
                    });
                    lex.expect_punct(Punct::Semicolon);
                }
            },

            Token::Ident(var_name) => {
                match lex.expect_any() {
                    Token::Punct(Punct::Semicolon) => {
                        block.push(Stmt {
                            loc,
                            kind: StmtKind::VarDecl(var_name)
                        });
                    },

                    Token::Punct(Punct::Colon) => {
                        block.push(Stmt {
                            loc: loc.clone(),
                            kind: StmtKind::VarDecl(var_name)
                        });

                        lex.expect_punct(Punct::Eq);
                        block.push(Stmt {
                            loc,
                            kind: StmtKind::VarAssign {
                                name: var_name,
                                expr: parse_expr(lex, 0),
                            }
                        });
                        lex.expect_punct(Punct::Semicolon);
                    },

                    Token::Punct(Punct::Eq) => {
                        block.push(Stmt {
                            loc,
                            kind: StmtKind::VarAssign {
                                name: var_name,
                                expr: parse_expr(lex, 0),
                            }
                        });
                        lex.expect_punct(Punct::Semicolon);
                    },

                    t @ _ => { unexpected_token_err!(lex.loc, t); }
                }
            },

            Token::Punct(Punct::CloseCurly) => break,

            t @ _ => { unexpected_token_err!(lex.loc, t); }
        }
        token = lex.expect_any();
    }

    block
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Var(nam) => write!(f, "{nam}"),
            Expr::Num(n)   => write!(f, "{n}"),
            Expr::BinOp { lhs, rhs, op } => {
                write!(f, "[{op} ")?;
                write!(f, "{lhs} ")?;
                write!(f, "{rhs}]")
            },
            Expr::FnCall { name, args } => {
                write!(f, "{name}(")?;
                write!(f, "{}", args[0])?;
                for i in 1..args.len() {
                    write!(f, ", {}", args[i])?;
                }
                write!(f, ")")
            },
        }
    }
}

pub fn parse_expr(lex: &mut Lexer, prec: u8) -> Expr {
    // the implementation based on the Pratt Parsing algorithm
    let token: Token;
    let mut lhs = match lex.expect_any() {
        Token::Number(n) => Expr::Num(n),
        Token::Ident(name) => {
            if lex.expect_peek_any() == Token::Punct(Punct::OpenParen) {
                lex.next_any();
                let mut args: Vec<Expr> = Vec::new();
                if lex.expect_peek_any() == Token::Punct(Punct::CloseParen) {
                    lex.next_any();
                } else {
                    args.push(parse_expr(lex, 0));
                    while lex.expect_any() != Token::Punct(Punct::CloseParen) {
                        args.push(parse_expr(lex, 0));
                    }
                }
                Expr::FnCall { name, args }
            } else {
                Expr::Var(name)
            }
        },
        Token::Punct(Punct::OpenParen) => {
            let lhs = parse_expr(lex, 0);
            token = lex.expect_any();
            if token != Token::Punct(Punct::CloseParen) {
                unexpected_token_err!(lex.loc, token);
            }
            lhs
        },
        t @ _ => { unexpected_token_err!(lex.loc, t); }
    };

    loop {
        match lex.expect_peek_any() {
            Token::Punct(
                Punct::Semicolon  |
                Punct::CloseParen |
                Punct::Comma      |
                Punct::OpenCurly
            ) => break,

            Token::BinOp(kind) => {
                let this_prec = bin_op_prec(kind.clone());
                if  this_prec < prec {
                    break
                } else {
                    lex.next_any();
                    let rhs = parse_expr(lex, this_prec);
                    lhs = Expr::BinOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                        op:  kind
                    };
                }
            },

            t @ _ => { unexpected_token_err!(lex.loc, t); },
        }
    }

    lhs
}

fn bin_op_prec(bin_op_kind: BinOpKind) -> u8 {
    match bin_op_kind {
        BinOpKind::Or  => 0,
        BinOpKind::And => 1,
        BinOpKind::Eq  | BinOpKind::Ne  => 2,
        BinOpKind::Add | BinOpKind::Sub => 4,
        BinOpKind::Mul | BinOpKind::Div => 5,
        BinOpKind::Gt
        | BinOpKind::Ge
        | BinOpKind::Lt
        | BinOpKind::Le => 3,
    }
}

// TODO: expression location

//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    #[test]
//    #[should_panic]
//    fn mismatched_parentheses() {
//        let mut expr_buf: Vec<Expr> = Vec::new();
//        let range = parse_expr(&mut expr_buf, &mut Lexer::new(b"(a));"), Punct::Semicolon);
//    }
//
//    #[test]
//    #[should_panic]
//    fn unexpected_comma() {
//        let mut expr_buf: Vec<Expr> = Vec::new();
//        let range = parse_expr(&mut expr_buf, &mut Lexer::new(b"(a + a,b);"), Punct::Semicolon);
//    }
//
//    #[test]
//    #[should_panic]
//    fn unexpected_paren() {
//        let mut expr_buf: Vec<Expr> = Vec::new();
//        let range = parse_expr(&mut expr_buf, &mut Lexer::new(b"a + ();"), Punct::Semicolon);
//    }
//
//    #[test]
//    fn expr() {
//        use super::Expr::*;
//
//        // Program: v1;v2;op
//        // 1 + 2          =>   12+
//        // 1 + 2 + 3      =>   12+ 3 +
//        // 1 + 2*3        =>   1 23* +
//        // 1 * 2 * 3      =>   1 23* *
//        // 1 + 2*3*4      =>   1 23* 4* +
//        // 1 + 2*3*4 + 5  =>   1 23* 4* + 5+
//        // 1 + 2*3 + 4*5  =>   1 23* + 45* +
//        // 1 / 2 * 3      =>   12/ 3*
//        // 1 / 2 * 3 * 4  =>   12/ 34**
//        // 1 / 2 * 3 / 4  =>   12/ 3* 4/
//        // f(1, f(2 + 3)); => 1 23+ sa f sa sa
//        let map: &[(&str, &[Expr])] = &[
//            ("1 + 2 == 3 - 1;", &[Num(1), Num(2), BinOp(BinOpKind::Add), Num(3), Num(1), BinOp(BinOpKind::Sub), BinOp(BinOpKind::Eq)]),
//            ("f();",           &[FnCall("f")]),
//            ("f(1, 2);",       &[Num(1), SetArg(0), Num(2), SetArg(1), FnCall("f")]),
//            ("f(1, 2 + 3);",   &[Num(1), SetArg(0), Num(2), Num(3), BinOp(BinOpKind::Add), SetArg(1), FnCall("f")]),
//            ("f(1, f(2, 3));", &[Num(1), SetArg(0), Num(2), SetArg(0), Num(3), SetArg(1), FnCall("f"), SetArg(1), FnCall("f")]),
//            ("f(1, f(2, 3 + 4));", &[Num(1), SetArg(0), Num(2), SetArg(0), Num(3), Num(4), BinOp(BinOpKind::Add), SetArg(1), FnCall("f"), SetArg(1), FnCall("f")]),
//            ("f(f(1, 2), f(3, 4));", &[Num(1), SetArg(0), Num(2), SetArg(1), FnCall("f"), SetArg(0), Num(3), SetArg(0), Num(4), SetArg(1), FnCall("f"), SetArg(1), FnCall("f")]),
//            ("1 + 2;",         &[Num(1), Num(2), BinOp(BinOpKind::Add)]),
//            ("1 + 2 + 3;",     &[Num(1), Num(2), BinOp(BinOpKind::Add), Num(3), BinOp(BinOpKind::Add)]),
//            ("1 + 2*3;",       &[Num(1), Num(2), Num(3), BinOp(BinOpKind::Mul), BinOp(BinOpKind::Add)]),
//            ("1 * 2 * 3;",     &[Num(1), Num(2), BinOp(BinOpKind::Mul), Num(3), BinOp(BinOpKind::Mul)]),
//            ("1 + 2*3*4;",     &[Num(1), Num(2), Num(3), BinOp(BinOpKind::Mul), Num(4), BinOp(BinOpKind::Mul), BinOp(BinOpKind::Add)]),
//            ("1 + 2*3*4 + 5;", &[Num(1), Num(2), Num(3), BinOp(BinOpKind::Mul), Num(4), BinOp(BinOpKind::Mul), BinOp(BinOpKind::Add), Num(5), BinOp(BinOpKind::Add)]),
//            ("1 + 2*3 + 4*5;", &[Num(1), Num(2), Num(3), BinOp(BinOpKind::Mul), BinOp(BinOpKind::Add), Num(4), Num(5), BinOp(BinOpKind::Mul), BinOp(BinOpKind::Add)]),
//            ("1 / 2 * 3;",     &[Num(1), Num(2), BinOp(BinOpKind::Div), Num(3), BinOp(BinOpKind::Mul)]),
//            ("1 / 2 * 3 * 4;", &[Num(1), Num(2), BinOp(BinOpKind::Div), Num(3), BinOp(BinOpKind::Mul), Num(4), BinOp(BinOpKind::Mul)]),
//            ("1 / 2 * 3 / 4;", &[Num(1), Num(2), BinOp(BinOpKind::Div), Num(3), BinOp(BinOpKind::Mul), Num(4), BinOp(BinOpKind::Div)]),
//            ("1 * (2 + 3);",   &[Num(1), Num(2), Num(3), BinOp(BinOpKind::Add), BinOp(BinOpKind::Mul)]),
//            ("1 * (2 + 3) + 2;",   &[Num(1), Num(2), Num(3), BinOp(BinOpKind::Add), BinOp(BinOpKind::Mul), Num(2), BinOp(BinOpKind::Add)]),
//            ("3 + 4 * 2 / (1 - 5);", &[Num(3), Num(4), Num(2), BinOp(BinOpKind::Mul), Num(1), Num(5), BinOp(BinOpKind::Sub), BinOp(BinOpKind::Div), BinOp(BinOpKind::Add)])
//        ];
//
//        let mut exprs: Vec<Expr> = Vec::new();
//        for test in map {
//            let range = parse_expr(&mut exprs, &mut Lexer::new(test.0.as_bytes()), Punct::Semicolon);
//            for x in range {
//                assert_eq!(exprs[x], test.1[x], "{:?}", test);
//            }
//
//            exprs.clear();
//        }
//    }
//}
