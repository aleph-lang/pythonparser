use aleph_syntax_tree::syntax::AlephTree as at;
use rustpython_parser::ast::{
    Expr, Stmt, BoolOp, CmpOp, Operator, UnaryOp, Arguments, ExprName, StmtReturn,
    ExprBoolOp, ExprBinOp, ExprUnaryOp, ExprLambda, ExprIfExp, ExprCompare, ExprCall,
    StmtFunctionDef, StmtAsyncFunctionDef, StmtClassDef, StmtFor, StmtAsyncFor, StmtWhile,
    StmtIf, StmtAssert, StmtImport, StmtExpr
};
use rustpython_parser::Parse;

/// Extracts a constant value from a Python `Constant` into an `AlephTree` node.
fn extract_constant(value: rustpython_parser::ast::Constant) -> at {
    match value {
        rustpython_parser::ast::Constant::None => at::Unit,
        rustpython_parser::ast::Constant::Bool(b) => at::Bool { value: b.to_string() },
        rustpython_parser::ast::Constant::Str(s) => at::String { value: format!("\"{}\"", s) },
        rustpython_parser::ast::Constant::Bytes(b) => at::Bytes { elems: b },
        rustpython_parser::ast::Constant::Int(i) => at::Int { value: i.to_string() },
        rustpython_parser::ast::Constant::Tuple(v) => at::Tuple {
            elems: v.into_iter().map(|c| Box::new(extract_constant(c))).collect(),
        },
        rustpython_parser::ast::Constant::Float(f) => at::Float { value: f.to_string() },
        rustpython_parser::ast::Constant::Complex { real, imag } => at::Complex {
            real: real.to_string(),
            imag: imag.to_string(),
        },
        rustpython_parser::ast::Constant::Ellipsis => at::Ellipsis,
    }
}

/// Extracts the identifier string from a Python `Expr::Name` node.
fn extract_name(expr: &Expr) -> String {
    match expr {
        Expr::Name(ExprName { id, .. }) => id.to_string(),
        _ => {
            println!("Not implemented: extract_name for {:?}", expr);
            String::new()
        }
    }
}

/// Translates Python function arguments into a vector of `AlephTree` nodes.
/// Currently a stub, returns an empty vector.
fn translate_arguments_vec(_args: &Arguments) -> Vec<Box<at>> {
    Vec::new()
}

/// Translates a Python expression AST node into an `AlephTree`.
fn translate_expr(expr: &Expr) -> at {
    match expr {
        Expr::BoolOp(ExprBoolOp { op, values, .. }) => {
            let mut res = at::Unit;
            for value in values {
                res = match op {
                    BoolOp::And => match res {
                        at::Unit => translate_expr(value),
                        _ => at::And {
                            bool_expr1: Box::new(res),
                            bool_expr2: Box::new(translate_expr(value)),
                        },
                    },
                    BoolOp::Or => match res {
                        at::Unit => translate_expr(value),
                        _ => at::Or {
                            bool_expr1: Box::new(res),
                            bool_expr2: Box::new(translate_expr(value)),
                        },
                    },
                }
            }
            res
        }
        Expr::BinOp(ExprBinOp { left, op, right, .. }) => match op {
            Operator::Add => at::Add {
                number_expr1: Box::new(translate_expr(left)),
                number_expr2: Box::new(translate_expr(right)),
            },
            Operator::Sub => at::Sub {
                number_expr1: Box::new(translate_expr(left)),
                number_expr2: Box::new(translate_expr(right)),
            },
            Operator::Mult => at::Mul {
                number_expr1: Box::new(translate_expr(left)),
                number_expr2: Box::new(translate_expr(right)),
            },
            Operator::Div => at::Div {
                number_expr1: Box::new(translate_expr(left)),
                number_expr2: Box::new(translate_expr(right)),
            },
            _ => {
                println!("Not implemented: BinOp {:?} {:?} {:?}", left, op, right);
                at::Unit
            }
        },
        Expr::UnaryOp(ExprUnaryOp { op, operand, .. }) => match op {
            UnaryOp::Not => at::Not {
                bool_expr: Box::new(translate_expr(operand)),
            },
            UnaryOp::USub => at::Neg {
                expr: Box::new(translate_expr(operand)),
            },
            _ => {
                println!("Not implemented: UnaryOp {:?} {:?}", op, operand);
                at::Unit
            }
        },
        Expr::Lambda(ExprLambda { args, body, .. }) => at::LetRec {
            name: "lambda".to_string(),
            args: translate_arguments_vec(args),
            body: Box::new(translate_expr(body)),
        },
        Expr::IfExp(ExprIfExp { test, body, orelse, .. }) => at::If {
            condition: Box::new(translate_expr(test)),
            then: Box::new(translate_expr(body)),
            els: Box::new(translate_expr(orelse)),
        },
        Expr::Compare(ExprCompare { left, ops, comparators, .. }) => {
            let mut res = translate_expr(left);
            for (op, right) in ops.iter().zip(comparators.iter()) {
                let right_expr = translate_expr(right);
                res = match op {
                    CmpOp::Eq => at::Eq {
                        expr1: Box::new(res),
                        expr2: Box::new(right_expr),
                    },
                    CmpOp::NotEq => at::Not {
                        bool_expr: Box::new(at::Eq {
                            expr1: Box::new(res),
                            expr2: Box::new(right_expr),
                        }),
                    },
                    CmpOp::Lt => at::And {
                        bool_expr1: Box::new(at::LE {
                            expr1: Box::new(res.clone()),
                            expr2: Box::new(right_expr.clone()),
                        }),
                        bool_expr2: Box::new(at::Not {
                            bool_expr: Box::new(at::Eq {
                                expr1: Box::new(res.clone()),
                                expr2: Box::new(right_expr.clone()),
                            }),
                        }),
                    },
                    CmpOp::LtE => at::LE {
                        expr1: Box::new(res),
                        expr2: Box::new(right_expr),
                    },
                    CmpOp::Gt => at::Not {
                        bool_expr: Box::new(at::LE {
                            expr1: Box::new(res),
                            expr2: Box::new(right_expr.clone()),
                        }),
                    },
                    CmpOp::GtE => at::Or {
                        bool_expr1: Box::new(at::LE {
                            expr1: Box::new(res.clone()),
                            expr2: Box::new(right_expr.clone()),
                        }),
                        bool_expr2: Box::new(at::Eq {
                            expr1: Box::new(res.clone()),
                            expr2: Box::new(right_expr.clone()),
                        }),
                    },
                    CmpOp::Is => at::Eq {
                        expr1: Box::new(res),
                        expr2: Box::new(right_expr),
                    },
                    CmpOp::IsNot => at::Not {
                        bool_expr: Box::new(at::Eq {
                            expr1: Box::new(res),
                            expr2: Box::new(right_expr),
                        }),
                    },
                    CmpOp::In => at::In {
                        expr1: Box::new(res),
                        expr2: Box::new(right_expr),
                    },
                    CmpOp::NotIn => at::Not {
                        bool_expr: Box::new(at::In {
                            expr1: Box::new(res),
                            expr2: Box::new(right_expr),
                        }),
                    },
                }
            }
            res
        }
        Expr::Call(ExprCall { func, args, .. }) => {
            let name = extract_name(func);
            let param_list = args.iter().map(|arg| Box::new(translate_expr(arg))).collect();
            at::App {
                object_name: "".to_string(),
                fun: Box::new(at::String { value: name }),
                param_list,
            }
        }
        Expr::Constant(expr_constant) => extract_constant(expr_constant.value.clone()),
        Expr::Name(ExprName { id, .. }) => at::Ident { value: id.to_string() },
        _ => {
            println!("Not implemented expression {:?}", expr);
            at::Unit
        }
    }
}

/// Translates a Python statement AST node into an `AlephTree`.
fn translate_stmt(stmt: &Stmt) -> at {
    match stmt {
        Stmt::FunctionDef(StmtFunctionDef { name, args, body, .. }) |
        Stmt::AsyncFunctionDef(StmtAsyncFunctionDef { name, args, body, .. }) => at::LetRec {
            name: name.to_string(),
            args: translate_arguments_vec(args),
            body: Box::new(translate_stmt_list(body)),
        },
        Stmt::ClassDef(StmtClassDef { name, body, .. }) => at::Clss {
            name: name.to_string(),
            attribute_list: Vec::new(),
            extends: None,
            implements: Vec::new(),
            body: Box::new(translate_stmt_list(body)),
        },
        Stmt::Return(StmtReturn { value, .. }) => match value {
            None => at::Unit,
            Some(expr) => at::Return {
                value: Box::new(translate_expr(expr)),
            },
        },
        Stmt::For(StmtFor { target, iter, body, orelse, .. }) |
        Stmt::AsyncFor(StmtAsyncFor { target, iter, body, orelse, .. }) => at::While {
            init_expr: Box::new(translate_expr(target)),
            condition: Box::new(translate_expr(iter)),
            loop_expr: Box::new(translate_stmt_list(body)),
            post_expr: Box::new(translate_stmt_list(orelse)),
        },
        Stmt::While(StmtWhile { test, body, orelse, .. }) => at::While {
            init_expr: Box::new(at::Unit),
            condition: Box::new(translate_expr(test)),
            loop_expr: Box::new(translate_stmt_list(body)),
            post_expr: Box::new(translate_stmt_list(orelse)),
        },
        Stmt::If(StmtIf { test, body, orelse, .. }) => at::If {
            condition: Box::new(translate_expr(test)),
            then: Box::new(translate_stmt_list(body)),
            els: Box::new(translate_stmt_list(orelse)),
        },
        Stmt::Assert(StmtAssert { test, msg, .. }) => at::Assert {
            condition: Box::new(translate_expr(test)),
            message: msg
                .as_ref()
                .map_or_else(|| Box::new(at::Unit), |m| Box::new(translate_expr(m))),
        },
        Stmt::Import(StmtImport { names, .. }) => {
            let items = names.iter().map(|alias| alias.name.to_string()).collect();
            at::Iprt {
                name: "".to_string(),
                items,
            }
        }
        Stmt::Expr(StmtExpr { value, .. }) => translate_expr(value),
        Stmt::Pass(_) => at::Unit,
        Stmt::Break(_) => at::Break,
        Stmt::Continue(_) => at::Continue,
        _ => {
            println!("Not implemented statement {:?}", stmt);
            at::Unit
        }
    }
}

/// Translates a list of Python statements into a sequence of `AlephTree` nodes.
fn translate_stmt_list(stmts: &[Stmt]) -> at {
    let mut res = at::Unit;
    for stmt in stmts {
        let current = translate_stmt(stmt);
        res = match res {
            at::Unit => current,
            _ => at::Stmts {
                expr1: Box::new(res),
                expr2: Box::new(current),
            },
        };
    }
    res
}

/// Parses a Python source string into an `AlephTree`.
pub fn python_parse(source: String) -> at {
    let ast = rustpython_parser::ast::Suite::parse(&source, "<embedded>")
        .expect("Failed to parse Python source.");
    translate_stmt_list(&ast)
}

