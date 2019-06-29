//! Alphatization source transform
//! This transform makes sure all variable names are unique within the source.

use super::SourceTransformer;
use crate::error::{ErrorKind, Result};
use crate::object::{ListBuilder, Object};
use crate::runtime::Symbol;
use crate::SchemeExpression;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Alphatizer {
    symbol_counter: usize,
}

impl SourceTransformer for Alphatizer {
    fn transform(&mut self, input: &Object) -> Result<Object> {
        let global_scope = Scope::new();
        self.transform_recursive(input, &global_scope)
    }
}

impl Alphatizer {
    pub fn new() -> Self {
        Alphatizer { symbol_counter: 0 }
    }

    fn transform_recursive(&mut self, input: &Object, scope: &Scope) -> Result<Object> {
        switch! {input,
            [(lambda ?params . ?body)] => {
                let mut inner_scope = scope.extend();
                let inserted = inner_scope.insert_vars(params, self);
                let new_params = self.transform_varlist(params, &inner_scope);
                let new_body = self.transform_sequence(body, &inner_scope);
                inserted
                    .and(new_params)
                    .and_then(|p| new_body.map(|b| (p, b)))
                    .map(|(p, b)| list!(lambda, @p, . @b))
            },
            [(quote . _)] => Ok(input.clone()),
            [(?first . ?tail)] => {
                self.transform_recursive(first, scope)
                    .and_then(|f| self.transform_sequence(tail, scope)
                        .map(|t| cons!(@f, @t)))
            },
            [?x] => if let Some(s) = x.as_symbol() {
                Ok(scope.lookup(s).into())
            } else {
                Ok(input.clone())
            },
        }
    }

    fn transform_sequence(&mut self, exps: &Object, scope: &Scope) -> Result<Object> {
        exps.map(|x| self.transform_recursive(x, scope))
    }

    fn transform_varlist(&mut self, vars: &Object, scope: &Scope) -> Result<Object> {
        vars.map(|x| Ok(scope.lookup(x.as_symbol().unwrap()).into()))
    }

    fn make_unique_symbol(&mut self, s: Symbol) -> Symbol {
        let id = self.symbol_counter;
        self.symbol_counter += 1;
        Symbol::new(format!("{}{}", s.name(), id))
    }
}

struct Scope<'a> {
    rename: HashMap<Symbol, Symbol>,
    parent: Option<&'a Scope<'a>>,
}

impl<'a> Scope<'a> {
    pub fn new() -> Self {
        Scope {
            rename: HashMap::new(),
            parent: None,
        }
    }

    pub fn extend(&'a self) -> Scope<'a> {
        Scope {
            rename: HashMap::new(),
            parent: Some(self),
        }
    }

    pub fn lookup(&self, var: Symbol) -> Symbol {
        match self.rename.get(&var) {
            Some(x) => *x,
            None => self.parent.map(|p| p.lookup(var)).unwrap_or(var),
        }
    }

    pub fn insert_vars(&mut self, vars: &Object, a: &mut Alphatizer) -> Result<()> {
        let mut cursor = vars;
        while let Some(var) = cursor.car() {
            let var = var.as_symbol().ok_or_else(|| {
                ErrorKind::SyntaxError(format!("function parameter is not a symbol: {:?}", var))
            })?;
            if self.rename.insert(var, a.make_unique_symbol(var)).is_some() {
                return Err(ErrorKind::SyntaxError(format!(
                    "duplicate function parameter: {:?}",
                    var
                ))
                .into());
            }
            cursor = cursor.cdr().unwrap();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_datum;

    macro_rules! assert_source_eq {
        ($transformer:expr, $actual:expr, $expected:expr) => {
            assert_eq!(
                $transformer
                    .transform(&parse_datum($actual).unwrap())
                    .unwrap(),
                parse_datum($expected).unwrap()
            )
        };
    }

    impl SourceTransformer for () {
        fn transform(&mut self, x: &Object) -> Result<Object> {
            Ok(x.clone())
        }
    }

    #[test]
    fn testing_framework_equal() {
        assert_source_eq!((), "(+ 1 2)", "(+ 1 2)")
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn testing_framework_unequal() {
        assert_source_eq!((), "(+ 1 2)", "(* 3 4)")
    }

    #[test]
    fn procedure_application_is_unchanged() {
        let mut alphatizer = Alphatizer::new();
        assert_source_eq!(
            alphatizer,
            "(cons (+ 1 2) '(3 4 5))",
            "(cons (+ 1 2) '(3 4 5))"
        );

        assert_source_eq!(
            alphatizer,
            "(begin (+ 1 2) '(3 4 5))",
            "(begin (+ 1 2) '(3 4 5))"
        );
    }

    #[test]
    fn alphatize_lambda() {
        let mut alphatizer = Alphatizer::new();
        assert_source_eq!(
            alphatizer,
            "(lambda (x y) (sqrt (+ (* x x) (* y y))))",
            "(lambda (x0 y1) (sqrt (+ (* x0 x0) (* y1 y1))))"
        );
    }

    #[test]
    fn alphatize_lambda_call() {
        let mut alphatizer = Alphatizer::new();
        assert_source_eq!(
            alphatizer,
            "((lambda (x y) (+ x y)) y x)",
            "((lambda (x0 y1) (+ x0 y1)) y x)"
        );
    }

    #[test]
    fn alphatize_nested_lambda() {
        let mut alphatizer = Alphatizer::new();
        assert_source_eq!(
            alphatizer,
            "(lambda (x y) ((lambda (x) (* x y) (+ x y)) y))",
            "(lambda (x0 y1) ((lambda (x2) (* x2 y1) (+ x2 y1)) y1))"
        );
    }

    #[test]
    fn alphatize_preserve_quote() {
        let mut alphatizer = Alphatizer::new();
        assert_source_eq!(
            alphatizer,
            "(lambda (x y) '(x y z))",
            "(lambda (x0 y1) '(x y z))"
        );
    }

    #[test]
    fn alphatize_program() {
        let mut alphatizer = Alphatizer::new();
        assert_source_eq!(
            alphatizer,
            "(begin
                (define x 3)
                (define y 4)
                (define sqr (lambda (x) (* x x)))
                (define sqrs (lambda (x y) (+ (sqr x) (sqr y))))
                (sqrs x y))",
            "(begin
                (define x 3)
                (define y 4)
                (define sqr (lambda (x0) (* x0 x0)))
                (define sqrs (lambda (x1 y2) (+ (sqr x1) (sqr y2))))
                (sqrs x y))"
        );
    }
}
