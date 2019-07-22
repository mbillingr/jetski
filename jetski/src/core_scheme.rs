use crate::object::TaggedValue;
use crate::runtime::Symbol;
use crate::Object;
use crate::SchemeExpression;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone)]
pub enum Expression {
    Nil,
    Integer(i64),
    Float(f64),
    Variable(Symbol),
    Lambda(Vec<Symbol>, Box<Expression>),
    Primitive,

    Let(Symbol, Box<Expression>, Box<Expression>),
    If(Box<Expression>, Box<Expression>, Box<Expression>),
    Apply(Box<Expression>, Vec<Expression>),

    //Begin(Vec<Expression>),
    DefVar(Symbol, Box<Expression>),
    DeFunc(Symbol, Vec<Symbol>, Box<Expression>),
}

impl Expression {
    fn is_atomic(&self) -> bool {
        match self {
            Expression::Integer(_)
            | Expression::Float(_)
            | Expression::Variable(_)
            | Expression::Lambda(_, _)
            | Expression::Primitive => true,
            _ => false,
        }
    }
}

impl std::fmt::Debug for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expression::Nil => write!(f, "'()"),
            Expression::Integer(x) => write!(f, "{}", x),
            Expression::Float(x) => write!(f, "{}", x),
            Expression::Variable(x) => write!(f, "{}", x),
            Expression::Lambda(params, body) => write!(
                f,
                "(lambda ({}) {:?})",
                params
                    .iter()
                    .map(|a| format!("{:?}", a))
                    .collect::<Vec<_>>()
                    .join(" "),
                body
            ),
            Expression::Primitive => write!(f, "<primitive>"),
            Expression::Let(var, init, body) => write!(f, "(let ({} {:?}) {:?})", var, init, body),
            Expression::If(cond, yes, no) => write!(f, "(if {:?} {:?} {:?})", cond, yes, no),
            Expression::Apply(proc, args) => write!(
                f,
                "({:?} {})",
                proc,
                args.iter()
                    .map(|a| format!("{:?}", a))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            /*Expression::Begin(exprs) => {
                write!(f, "(begin")?;
                for x in exprs {
                    write!(f, " {:?}", x)?;
                }
                write!(f, ")")
            }*/
            Expression::DeFunc(name, params, body) => write!(
                f,
                "(define ({}, {}) {:?})",
                name,
                params
                    .iter()
                    .map(|a| format!("{:?}", a))
                    .collect::<Vec<_>>()
                    .join(" "),
                body
            ),
            Expression::DefVar(name, expr) => write!(f, "(define {} {:?})", name, expr),
        }
    }
}

impl From<Object> for Expression {
    fn from(obj: Object) -> Self {
        (&obj).into()
    }
}

impl From<&Object> for Expression {
    fn from(obj: &Object) -> Self {
        match obj.as_value() {
            TaggedValue::Undef => unimplemented!(),
            TaggedValue::Nil => Expression::Nil,
            TaggedValue::Integer(x) => Expression::Integer(*x),
            TaggedValue::Float(x) => Expression::Float(*x),
            TaggedValue::Symbol(s) => Expression::Variable(*s),
            TaggedValue::String(_) => unimplemented!(),
            TaggedValue::Function(_) => unimplemented!(),
            TaggedValue::Pair(_, _) => switch! {obj,
                [(define (?f . ?params) ?body)] => Expression::DeFunc(f.as_symbol().unwrap(),
                                                                      params.list_to_vec()
                                                                            .unwrap()
                                                                            .into_iter()
                                                                            .map(|p|p.as_symbol().unwrap())
                                                                            .collect(),
                                                                      Box::new(body.into())),
                [(define ?var ?exp)] => unimplemented!(),
                // lambda with single expression body
                [(lambda ?params ?body)] => Expression::Lambda(params.list_to_vec()
                                                                     .unwrap()
                                                                     .into_iter()
                                                                     .map(|p|p.as_symbol().unwrap())
                                                                     .collect(),
                                                               Box::new(body.into())),
                // lambda with sequence body
                [(lambda ?params . ?body)] => unimplemented!(),
                // procedure application
                [(?proc . ?args)] => Expression::Apply(Box::new(proc.into()),
                                                       args.list_to_vec()
                                                           .unwrap()
                                                           .into_iter()
                                                           .map(From::from)
                                                           .collect()),
                [_] => unimplemented!()
            },
        }
    }
}

struct AnormalTransform {}

fn normalize_program(decs: Vec<Expression>) -> Vec<Expression> {
    decs.into_iter()
        .map(|dec| match dec {
            Expression::DefVar(_, _) | Expression::DeFunc(_, _, _) => normalize_define(dec),
            exp => normalize_term(exp),
        })
        .collect()
}

fn normalize_term(expr: Expression) -> Expression {
    normalize(expr, Box::new(|x| x))
}

fn normalize_define(def: Expression) -> Expression {
    use Expression::*;
    match def {
        DeFunc(name, params, body) => DefVar(name, Box::new(normalize_term(Lambda(params, body)))),
        //DefVar(name, expr) => Begin(flatten_top(normalize_term(*expr), name)),
        DefVar(name, expr) => DefVar(name, Box::new(normalize_term(*expr))),
        _ => panic!("Syntax error: Expected definition, got {:?}", def),
    }
}

fn normalize(expr: Expression, k: Box<dyn FnOnce(Expression) -> Expression>) -> Expression {
    use Expression::*;
    match expr {
        Lambda(params, body) => k(Lambda(params, Box::new(normalize_term(*body)))),
        Let(var, init, body) => normalize(
            *init,
            Box::new(move |n| Let(var, Box::new(n), Box::new(normalize(*body, k)))),
        ),
        If(cond, yes, no) => normalize_name(
            *cond,
            Box::new(|t| {
                k(Expression::If(
                    Box::new(t),
                    Box::new(normalize_term(*yes)),
                    Box::new(normalize_term(*no)),
                ))
            }),
        ),
        Apply(proc, args) => {
            if let Primitive = *proc {
                normalize_names(
                    args,
                    Box::new(move |ts| k(Expression::Apply(proc.clone(), ts))),
                )
            } else {
                normalize_name(
                    *proc,
                    Box::new(|t| {
                        normalize_names(
                            args,
                            Box::new(move |mut ts| k(Expression::Apply(Box::new(t), ts))),
                        )
                    }),
                )
            }
        }
        _ => k(expr),
    }
}

fn normalize_names(
    exprs: Vec<Expression>,
    k: Box<dyn FnOnce(Vec<Expression>) -> Expression>,
) -> Expression {
    if exprs.is_empty() {
        k(vec![])
    } else {
        normalize_name(
            exprs[0].clone(),
            Box::new(move |t| {
                normalize_names(
                    exprs[1..].to_vec(),
                    Box::new(move |mut ts| {
                        ts.insert(0, t);
                        k(ts)
                    }),
                )
            }),
        )
    }
}

fn normalize_name(expr: Expression, k: Box<dyn FnOnce(Expression) -> Expression>) -> Expression {
    normalize(
        expr,
        Box::new(|n| {
            if n.is_atomic() {
                k(n)
            } else {
                let t = new_var();
                Expression::Let(t, Box::new(n), Box::new(k(Expression::Variable(t))))
            }
        }),
    )
}

// The implementation by Matt Might uses this function to convert nested let expressions into a
// sequence of defines at the top level. I'm not sure why it is necessary to put variables into
// the global environment that were not explicitly put there by the programmer.
// I just disabled this behavior for now to see how things turn out...
/*fn flatten_top(expr: Expression, v: Symbol) -> Vec<Expression> {
    use Expression::*;
    match expr {
        Let(x, cexp, body) => {
            let mut defs = vec![DefVar(x, cexp)];
            defs.extend(flatten_top(*body, v));
            defs
        }
        _ => vec![DefVar(v, Box::new(expr))],
    }
}*/

static VAR_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn new_var() -> Symbol {
    let id = VAR_COUNTER.fetch_add(1, Ordering::SeqCst);
    Symbol::new(format!("newvar-{}", id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_datum;

    #[test]
    fn it_works() {
        let n = Symbol::new("n");
        /*println!(
        "{:?}",
        alt_impl::normalize_term(Expression::Lambda(
            vec![n],
            Box::new(Expression::If(
                Box::new(Expression::Apply(
                    Box::new(Expression::Primitive),
                    vec![Expression::Variable(n), Expression::Integer(0)]
                )),
                Box::new(Expression::Integer(1)),
                Box::new(Expression::Apply(
                    Box::new(Expression::Primitive),
                    vec![
                        Expression::Variable(n),
                        Expression::Apply(
                            Box::new(Expression::Variable(Symbol::new("fact"))),
                            vec![Expression::Apply(
                                Box::new(Expression::Primitive),
                                vec![Expression::Variable(n), Expression::Integer(1)]
                            )]
                        )
                    ]
                ))
            ))
        )));*/
        println!(
            "{:?}",
            normalize_term(Expression::Lambda(
                vec![n],
                Box::new(Expression::If(
                    Box::new(Expression::Apply(
                        Box::new(Expression::Primitive),
                        vec![Expression::Variable(n), Expression::Integer(0)]
                    )),
                    Box::new(Expression::Integer(1)),
                    Box::new(Expression::Apply(
                        Box::new(Expression::Primitive),
                        vec![
                            Expression::Variable(n),
                            Expression::Apply(
                                Box::new(Expression::Variable(Symbol::new("fact"))),
                                vec![Expression::Apply(
                                    Box::new(Expression::Primitive),
                                    vec![Expression::Variable(n), Expression::Integer(1)]
                                )]
                            )
                        ]
                    ))
                ))
            )) /*normalize_term(Expression::Apply(
                   Box::new(Expression::Lambda(
                       vec![x],
                       Box::new(Expression::Apply(
                           Box::new(Expression::Primitive),
                           vec![Expression::Variable(x), Expression::Variable(x),]
                       ))
                   )),
                   vec![Expression::Apply(
                       Box::new(Expression::Primitive),
                       vec![Expression::Integer(1), Expression::Integer(2),]
                   )]
               )) */
                   /*normalize_term(Expression::Apply(
                          Box::new(Expression::Primitive),
                          vec![
                              Expression::Apply(
                                  Box::new(Expression::Primitive),
                                  vec![Expression::Variable(x), Expression::Variable(x),]
                              ),
                              Expression::Variable(x),
                          ]
                      ))*/
        );
        println!(
            "{:?}",
            normalize_term(Expression::Let(
                Symbol::new("x"),
                Box::new(Expression::If(
                    Box::new(Expression::Variable(Symbol::new("c1"))),
                    Box::new(Expression::Apply(
                        Box::new(Expression::Primitive),
                        vec![Expression::Integer(1)]
                    )),
                    Box::new(Expression::Apply(
                        Box::new(Expression::Primitive),
                        vec![Expression::Integer(2)]
                    )),
                )),
                Box::new(Expression::Let(
                    Symbol::new("y"),
                    Box::new(Expression::If(
                        Box::new(Expression::Variable(Symbol::new("c1"))),
                        Box::new(Expression::Apply(
                            Box::new(Expression::Primitive),
                            vec![Expression::Variable(Symbol::new("x"))]
                        )),
                        Box::new(Expression::Apply(
                            Box::new(Expression::Primitive),
                            vec![Expression::Variable(Symbol::new("x"))]
                        )),
                    )),
                    Box::new(Expression::Variable(Symbol::new("y")))
                ))
            ))
        );

        println!(
            "{:?}",
            normalize_define(Expression::DefVar(
                Symbol::new("z"),
                Box::new(Expression::Let(
                    Symbol::new("x"),
                    Box::new(Expression::If(
                        Box::new(Expression::Variable(Symbol::new("c1"))),
                        Box::new(Expression::Apply(
                            Box::new(Expression::Primitive),
                            vec![Expression::Integer(1)]
                        )),
                        Box::new(Expression::Apply(
                            Box::new(Expression::Primitive),
                            vec![Expression::Integer(2)]
                        )),
                    )),
                    Box::new(Expression::Let(
                        Symbol::new("y"),
                        Box::new(Expression::If(
                            Box::new(Expression::Variable(Symbol::new("c1"))),
                            Box::new(Expression::Apply(
                                Box::new(Expression::Primitive),
                                vec![Expression::Variable(Symbol::new("x"))]
                            )),
                            Box::new(Expression::Apply(
                                Box::new(Expression::Primitive),
                                vec![Expression::Variable(Symbol::new("x"))]
                            )),
                        )),
                        Box::new(Expression::Variable(Symbol::new("y")))
                    ))
                ))
            ))
        );

        println!(
            "{:?}",
            normalize_program(vec![parse_datum(
                "(define (sillyfunc x) (+ x (sqr (- x ref))))"
            )
            .unwrap()
            .into()])
        );

        panic!()
    }
}
