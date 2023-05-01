use aleph_syntax_tree::syntax::AlephTree as at;

use rustpython_parser::{ast, parser};
use rustpython_parser::ast::{ExprKind, StmtKind};
use crate::ast::{Arguments, Boolop, Cmpop, Constant, Located, Operator, Unaryop};

fn extract_constant(value : Constant) -> at {
    match value {
        Constant::None => at::Unit,
        Constant::Bool(b) => at::Bool{value: b.to_string()},
        Constant::Str(s) => at::String{value: format!("\"{}\"", s)},
        Constant::Bytes(b) => at::Bytes{elems: b},
        Constant::Int(i) => at::Int{value: i.to_string()},
        Constant::Tuple(v) => {
            let mut res = Vec::new();
            for c in v {
                res.push(Box::new(extract_constant(c)));
            }
            at::Tuple{elems : res}
        },
        Constant::Float(f) => at::Float{value: f.to_string()},
        Constant::Complex{real, imag} => at::Complex{real: real.to_string(), imag: imag.to_string()},
        Constant::Ellipsis => at::Ellipsis
    }
}

fn extract_name(ek: ExprKind) -> String {
    match ek {
        ExprKind::Name{id, ctx: _} => id,
        _ => {
            println!("Not Impl extract_name {:?}", ek);
            "".to_string()
        }
    }
}

// TODO translate Arguments to Vec<Box<at>>
// pub struct Arguments<U = ()> {
//    pub posonlyargs: Vec<Arg<U>>,
//    pub args: Vec<Arg<U>>,
//    pub vararg: Option<Box<Arg<U>>>,
//    pub kwonlyargs: Vec<Arg<U>>,
//    pub kw_defaults: Vec<Expr<U>>,
//    pub kwarg: Option<Box<Arg<U>>>,
//    pub defaults: Vec<Expr<U>>,
//}
fn translate_arguments_vec(_args : Arguments) -> Vec<Box<at>> {
    Vec::new()
}

fn translate_expr_kind(ek: ExprKind) -> at {
    match ek {
        ExprKind::BoolOp{op, values} => {
            let mut res = at::Unit;
            for value in values {
                res = match op {
                    Boolop::And => match res {
                        at::Unit => translate_expr_kind(value.node),
                        _ => at::And{bool_expr1: Box::new(res), bool_expr2: Box::new(translate_expr_kind(value.node))},
                    },
                    Boolop::Or => match res {
                        at::Unit => translate_expr_kind(value.node),
                        _ => at::Or{bool_expr1: Box::new(res), bool_expr2: Box::new(translate_expr_kind(value.node))}
                    }
                }
            }
            res
        },
        ExprKind::NamedExpr{target, value} => {
            println!("Not impl NamedExpr {:?} {:?}", target, value);
            at::Unit
        },
        ExprKind::BinOp{left, op, right} => {
            match op {
                Operator::Add => at::Add{number_expr1: Box::new(translate_expr_kind(left.node)), number_expr2: Box::new(translate_expr_kind(right.node))},
                Operator::Sub => at::Sub{number_expr1: Box::new(translate_expr_kind(left.node)), number_expr2: Box::new(translate_expr_kind(right.node))},
                Operator::Mult => at::Mul{number_expr1: Box::new(translate_expr_kind(left.node)), number_expr2: Box::new(translate_expr_kind(right.node))},
                Operator::Div => at::Div{number_expr1: Box::new(translate_expr_kind(left.node)), number_expr2: Box::new(translate_expr_kind(right.node))},
                _ => {
                    println!("Not impl BinOp {:?} {:?} {:?}", left, op, right);
                    at::Unit
                }
            }
        },
        ExprKind::UnaryOp{op, operand} => {
            match op {
                Unaryop::Not => at::Not{bool_expr: Box::new(translate_expr_kind(operand.node))},
                Unaryop::USub => at::Neg{expr: Box::new(translate_expr_kind(operand.node))},
                _ => {
                    println!("Not impl UnaryOp {:?} {:?}", op, operand);
                    at::Unit
                }
            }
        },
        ExprKind::Lambda{args, body} => at::LetRec{name: "lambda".to_string(), args: translate_arguments_vec(*args), body: Box::new(translate_expr_kind(body.node))},
        ExprKind::IfExp{test, body, orelse} => at::If{condition: Box::new(translate_expr_kind(test.node)), then: Box::new(translate_expr_kind(body.node)), els: Box::new(translate_expr_kind(orelse.node))},
        ExprKind::Dict{keys, values} => {
            println!("Not impl Dict {:?} {:?}", keys, values);
            at::Unit
        },
        ExprKind::Set{elts} => {
            println!("Not impl Set {:?}", elts);
            at::Unit
        },
        ExprKind::ListComp{elt, generators} => {
            println!("Not impl ListComp {:?} {:?}", elt, generators);
            at::Unit
        },
        ExprKind::SetComp{elt, generators} => {
            println!("Not impl Set {:?} {:?}", elt, generators);
            at::Unit
        },
        ExprKind::DictComp{key, value, generators} => {
            println!("Not impl DictComp {:?} {:?} {:?}", key, value, generators);
            at::Unit
        },
        ExprKind::GeneratorExp{elt, generators} => {
            println!("Not impl GeneratorExp {:?} {:?}", elt, generators);
            at::Unit
        },
        ExprKind::Await{value} => {
            println!("Not impl Await {:?}", value);
            at::Unit
        },
        ExprKind::Yield{value} => {
            println!("Not impl Yield {:?}", value);
            at::Unit
        },
        ExprKind::YieldFrom{value} => {
            println!("Not impl YieldFrom {:?}", value);
            at::Unit
        },
        ExprKind::Compare{left, ops, comparators} => {
            let mut res = translate_expr_kind(left.node);
            for (op, right) in ops.iter().zip(comparators.iter()) {
                let right_expr = translate_expr_kind(right.node.clone());
                res = match op {
                    Cmpop::Eq => at::Eq{expr1: Box::new(res), expr2: Box::new(right_expr)},
                    Cmpop::NotEq => at::Not{bool_expr: Box::new(at::Eq{expr1: Box::new(res), expr2: Box::new(right_expr)})},
                    Cmpop::Lt => at::And{bool_expr1: Box::new(at::LE{expr1: Box::new(res.clone()), expr2: Box::new(right_expr.clone())}), bool_expr2: Box::new(at::Not{bool_expr: Box::new(at::Eq{expr1: Box::new(res.clone()), expr2: Box::new(right_expr.clone())})})},
                    Cmpop::LtE => at::LE{expr1: Box::new(res), expr2: Box::new(right_expr)},
                    Cmpop::Gt => at::Not{bool_expr: Box::new(at::LE{expr1: Box::new(res), expr2: Box::new(right_expr.clone())})},
                    Cmpop::GtE => at::Or{bool_expr1: Box::new(at::LE{expr1: Box::new(res.clone()), expr2: Box::new(right_expr.clone())}), bool_expr2: Box::new(at::Eq{expr1: Box::new(res.clone()), expr2: Box::new(right_expr.clone())})},
                    Cmpop::Is => at::Eq{expr1: Box::new(res), expr2: Box::new(right_expr)},
                    Cmpop::IsNot => at::Not{bool_expr: Box::new(at::Eq{expr1: Box::new(res), expr2: Box::new(right_expr)})},
                    Cmpop::In => at::In{expr1: Box::new(res), expr2: Box::new(right_expr)},
                    Cmpop::NotIn => at::Not{bool_expr: Box::new(at::In{expr1: Box::new(res), expr2: Box::new(right_expr)})},
                }
            }
            res
        },
        ExprKind::Call{func, args, keywords: _} => {
            let name = extract_name(func.node);
            let mut param_list: Vec<Box<at>> = Vec::new();
            for arg in args {
                param_list.push(Box::new(translate_expr_kind(arg.node)));
            }
            at::App{object_name: "".to_string(), fun: Box::new(at::String{value: name}), param_list: param_list}
        },
        ExprKind::FormattedValue{value, conversion, format_spec} => {
            println!("Not impl FormattedValue {:?} {:?} {:?}", value, conversion, format_spec);
            at::Unit
        },
        ExprKind::JoinedStr{values} => {
            println!("Not impl JoinedStr {:?}", values);
            at::Unit
        },
        ExprKind::Constant{value, kind: _} => extract_constant(value),
        ExprKind::Attribute{value, attr, ctx} => {
            println!("Not impl Attribute {:?} {:?} {:?}", value, attr, ctx);
            at::Unit
        },
        ExprKind::Subscript{value, slice, ctx} => {
            println!("Not impl Subscript {:?} {:?} {:?}", value, slice, ctx);
            at::Unit
        },
        ExprKind::Starred{value, ctx} => {
            println!("Not impl Starred {:?} {:?}", value, ctx);
            at::Unit
        },
        ExprKind::Name{id, ctx: _} => {
            at::String{value: id}
        },
        ExprKind::List{elts, ctx} => {
            println!("Not impl List {:?} {:?}", elts, ctx);
            at::Unit
        },
        ExprKind::Tuple{elts, ctx} => {
            println!("Not impl Tuple {:?} {:?}", elts, ctx);
            at::Unit
        },
        ExprKind::Slice{lower, upper, step} => {
            println!("Not impl Slice {:?} {:?} {:?}", lower, upper, step);
            at::Unit
        }
    }
}

fn translate_stmt_kind(sk : StmtKind) -> at {
    match sk {
        StmtKind::FunctionDef{name, args, body, decorator_list: _, returns: _, type_comment: _} => at::LetRec{name: name, args: translate_arguments_vec(*args), body: Box::new(translate_stmt_kind_list(body))},
        StmtKind::AsyncFunctionDef{name, args, body, decorator_list: _, returns: _, type_comment: _} => at::LetRec{name: name, args: translate_arguments_vec(*args), body: Box::new(translate_stmt_kind_list(body))},
        StmtKind::ClassDef{name, bases: _, keywords: _, body, decorator_list: _} => at::Clss{name: name, attribute_list: Vec::new(), body: Box::new(translate_stmt_kind_list(body))},
        StmtKind::Return{value} => match value {
            None => at::Unit,
            Some(r) => at::Return{value: Box::new(translate_expr_kind(r.node))},
        },
        StmtKind::Delete{targets} => {
            println!("Not impl Delete {:?}", targets);
            at::Unit 
        },
        StmtKind::Assign{targets, value, type_comment} => {
            println!("Not impl Assign {:?} {:?} {:?}", targets, value, type_comment);
            at::Unit 
        },
        StmtKind::AugAssign{target, op, value} => {
            println!("Not impl AugAssign {:?} {:?} {:?}", target, op, value);
            at::Unit 
        },
        StmtKind::AnnAssign{target, annotation, value, simple} => {
            println!("Not impl AnnAssign {:?} {:?} {:?} {:?}", target, annotation, value, simple);
            at::Unit 
        },
        StmtKind::For{target, iter, body, orelse, type_comment: _} => at::While{init_expr: Box::new(translate_expr_kind(target.node)), condition: Box::new(translate_expr_kind(iter.node)), loop_expr: Box::new(translate_stmt_kind_list(body)), post_expr: Box::new(translate_stmt_kind_list(orelse))},
        StmtKind::AsyncFor{target, iter, body, orelse, type_comment: _} => at::While{init_expr: Box::new(translate_expr_kind(target.node)), condition: Box::new(translate_expr_kind(iter.node)), loop_expr: Box::new(translate_stmt_kind_list(body)), post_expr: Box::new(translate_stmt_kind_list(orelse))},
        StmtKind::While{test, body, orelse} => at::While{init_expr: Box::new(at::Unit), condition: Box::new(translate_expr_kind(test.node)), loop_expr: Box::new(translate_stmt_kind_list(body)), post_expr: Box::new(translate_stmt_kind_list(orelse))},
        StmtKind::If{test, body, orelse} => at::If{condition: Box::new(translate_expr_kind(test.node)), then: Box::new(translate_stmt_kind_list(body)), els: Box::new(translate_stmt_kind_list(orelse))},
        StmtKind::With{items, body, type_comment} => {
            println!("Not impl With {:?} {:?} {:?}", items, body, type_comment);
            at::Unit 
        },
        StmtKind::AsyncWith{items, body, type_comment} => {
            println!("Not impl AsyncWith {:?} {:?} {:?}", items, body, type_comment);
            at::Unit 
        },
        StmtKind::Match{subject, cases} => {
            println!("Not impl Match {:?} {:?}", subject, cases);
            at::Unit 
        },
        StmtKind::Raise{exc, cause} => {
            println!("Not impl Raise {:?} {:?}", exc, cause);
            at::Unit 
        },
        StmtKind::Try{body, handlers, orelse, finalbody} => {
            println!("Not impl Try {:?} {:?} {:?} {:?}", body, handlers, orelse, finalbody);
            at::Unit 
        },
        StmtKind::Assert{test, msg} => {
            println!("Not impl Assert {:?} {:?}", test, msg);
            at::Unit 
        },
        StmtKind::Import{names} => {
            println!("Not impl Import {:?}", names);
            at::Unit 
        },
        StmtKind::ImportFrom{module, names, level} => {
            println!("Not impl ImportFrom {:?} {:?} {:?}", module, names, level);
            at::Unit 
        },
        StmtKind::Global{names} => {
            println!("Not impl Global {:?}", names);
            at::Unit 
        },
        StmtKind::Nonlocal{names} => {
            println!("Not impl Nonlocal {:?}", names);
            at::Unit 
        },
        StmtKind::Expr{value} => translate_expr_kind(value.node),
        StmtKind::Pass => {
            println!("Not impl Pass");
            at::Unit 
        },
        StmtKind::Break => {
            println!("Not impl Break");
            at::Unit 
        },
        StmtKind::Continue => {
            println!("Not impl Continue");
            at::Unit 
        }
    }
}

fn translate_stmt_kind_list(skl: Vec<Located<StmtKind>>) -> at { 
    let mut res = at::Unit;
    for sk in skl {
        res = match res {
            at::Unit => translate_stmt_kind(sk.node),
            _ => at::Stmts{expr1: Box::new(res), expr2: Box::new(translate_stmt_kind(sk.node))}
        }
    }
    res
}

/// Python parser
/// #Arguments
/// `source` - String to parse
///
/// # Return
/// This function return an AlephTree
pub fn python_parse(source: String) -> at {
    let ast = parser::parse_program(&source, "<embedded>").unwrap();
    //  println!("AST: {:?}", ast);
    translate_stmt_kind_list(ast)
}


