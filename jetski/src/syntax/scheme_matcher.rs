#[macro_export]
macro_rules! switch {
    ($exp:expr, [$($template:tt)*] => $action:expr) => {
        switch!($exp, [$($template)*] => $action,)
    };

    ($exp:expr, [$($template:tt)*] => $action:expr,) => {
        scheme_match!($exp, {$action}, $($template)*)
            .expect("Last clause in switch must never fail to match")
    };

    ($exp:expr, [$($template:tt)*] => $action:expr, $($rest:tt)*) => {
        scheme_match!($exp, {$action}, $($template)*)
            .unwrap_or_else(|| switch!($exp, $($rest)*))
    };

    ($exp:expr, $predicate:expr => $action:expr,) => {
        if $predicate($exp) {
            $action
        } else {
            panic!("Last clause in switch must never fail to match")
        }
    };

    ($exp:expr, $predicate:expr => $action:expr, $($rest:tt)*) => {
        if $predicate($exp) {
            $action
        } else {
            switch!($exp, $($rest)*)
        }
    };
}

#[macro_export]
macro_rules! scheme_match {
    ($exp:expr, $action:block, ($single:tt)) => {
        $exp.decons()
            .filter(|(_, cdr)| cdr.is_nil())
            .and_then(|(_car, _)| scheme_match!(_car, $action, $single))
    };

    ($exp:expr, $action:block, ($car:tt . $($cdr:tt)*)) => {
        $exp.decons()
            .and_then(|(_car, _cdr)| scheme_match!(_car, {
                scheme_match!(_cdr, $action, $($cdr)*)
            }, $car)).unwrap_or(None)
    };

    ($exp:expr, $action:block, (? $car:tt . $($cdr:tt)*)) => {
        $exp.decons()
            .and_then(|(_car, _cdr)| scheme_match!(_car, {
                scheme_match!(_cdr, $action, $($cdr)*)
            }, ?$car)).unwrap_or(None)
    };

    ($exp:expr, $action:block, (? $var:tt)) => {
        $exp.decons()
            .filter(|(_, cdr)| cdr.is_nil())
            .and_then(|(car, _)| scheme_match!(car, $action, ?$var))
    };

    ($exp:expr, $action:block, (? $first:tt $($rest:tt)*)) => {
        $exp.decons()
            .and_then(|(car, cdr)| {
                scheme_match!(car, {
                    scheme_match!(cdr, $action, ($($rest)*))
                }, ?$first).unwrap_or(None)
            })
    };

    ($exp:expr, $action:block, ($first:tt $($rest:tt)*)) => {
        $exp.decons()
            .and_then(|(_car, _cdr)| {
                scheme_match!(_car, {
                    scheme_match!(_cdr, $action, ($($rest)*))
                }, $first).unwrap_or(None)
            })
    };

    ($exp:expr, $action:block, ()) => {
        if $exp.is_nil() {
            Some($action)
        } else {
            None
        }
    };

    ($exp:expr, $action:block, _) => {
        Some($action)
    };

    ($exp:expr, $action:block, ? $var:ident) => {
        {
            let $var = $exp;
            Some($action)
        }
    };

    ($exp:expr, $action:block, $sym:ident) => {
        if let Some(stringify!($sym)) = $exp.symbol_name() {
            Some($action)
        } else {
            None
        }
    };

    ($exp:expr, $action:block, $val:expr) => {
        if $exp == &$val {
            Some($action)
        } else {
            None
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::SchemeExpression;
    use Expr::*;

    #[derive(Debug, PartialEq)]
    enum Expr {
        Nil,
        Symbol(&'static str),
        Int(i64),
        Pair(Box<Expr>, Box<Expr>),
    }

    impl Expr {
        fn cons(car: Expr, cdr: Expr) -> Self {
            Pair(Box::new(car), Box::new(cdr))
        }
    }

    impl crate::SchemeExpression for Expr {
        fn symbol_name(&self) -> Option<&'static str> {
            if let Symbol(name) = self {
                Some(name)
            } else {
                None
            }
        }

        fn car(&self) -> Option<&Self> {
            if let Pair(a, _) = self {
                Some(a)
            } else {
                None
            }
        }

        fn cdr(&self) -> Option<&Self> {
            if let Pair(_, d) = self {
                Some(d)
            } else {
                None
            }
        }

        fn is_nil(&self) -> bool {
            *self == Nil
        }
    }

    impl PartialEq<i64> for Expr {
        fn eq(&self, rhs: &i64) -> bool {
            if let Int(i) = self {
                i == rhs
            } else {
                false
            }
        }
    }

    #[test]
    fn simple_mismatch_sconstant() {
        assert_eq!(None, scheme_match!(&Symbol("xyz"), {}, abc));
        assert_eq!(None, scheme_match!(&Int(666), {}, 42));
        assert_eq!(None, scheme_match!(&Int(42), {}, ()));
    }

    #[test]
    fn simple_match_constant() {
        assert_eq!(Some(()), scheme_match!(&Symbol("xyz"), {}, xyz));
        assert_eq!(Some(()), scheme_match!(&Int(42), {}, 42));
        assert_eq!(Some(()), scheme_match!(&Nil, {}, ()));
        assert_eq!(Some(()), scheme_match!(&Symbol("xyz"), {}, _));
    }

    #[test]
    fn simple_match_bound() {
        assert_eq!(
            Some(&Symbol("xyz")),
            scheme_match!(&Symbol("xyz"), { x }, ?x)
        );
        assert_eq!(Some(&Int(42)), scheme_match!(&Int(42), { x }, ?x));
        assert_eq!(Some(&Nil), scheme_match!(&Nil, { x }, ?x));
    }

    #[test]
    fn unary_list() {
        let list = Expr::cons(Int(42), Nil);
        assert_eq!(Some(()), scheme_match!(&list, {}, (42)));

        let list = Expr::cons(Symbol("alpha"), Nil);
        assert_eq!(Some(()), scheme_match!(&list, {}, (alpha)));
    }

    #[test]
    fn unary_list_bound() {
        let list = Expr::cons(Int(42), Nil);
        assert_eq!(Some(&Int(42)), scheme_match!(&list, { x }, (?x)));
    }

    #[test]
    fn list_match() {
        let list = Expr::cons(Int(1), Expr::cons(Int(2), Expr::cons(Int(3), Nil)));
        assert_eq!(None, scheme_match!(&list, {}, (_ _)));
        assert_eq!(Some(()), scheme_match!(&list, {}, (_ _ _)));
        assert_eq!(None, scheme_match!(&list, {}, (_ _ _ _)));
        assert_eq!(Some(&Int(1)), scheme_match!(&list, {x}, (?x _ _)));
        assert_eq!(Some(&Int(2)), scheme_match!(&list, {y}, (_ ?y _)));
        assert_eq!(Some(&Int(3)), scheme_match!(&list, {z}, (_ _ ?z)));
    }

    #[test]
    fn nested_list() {
        let list = Expr::cons(
            Int(1),
            Expr::cons(
                Expr::cons(Symbol("a2"), Expr::cons(Symbol("b2"), Nil)),
                Expr::cons(Int(3), Nil),
            ),
        );
        assert_eq!(
            Some((&Int(1), &Symbol("b2"))),
            scheme_match!(&list, { (x, y) }, (?x (a2 ?y) _))
        );
    }

    #[test]
    fn pair_match() {
        let pair = Expr::cons(Int(1), Int(2));
        assert_eq!(None, scheme_match!(&pair, {}, (_ _)));
        assert_eq!(Some(()), scheme_match!(&pair, {}, (_ . _)));
        assert_eq!(None, scheme_match!(&pair, {}, (_ _ _)));
        assert_eq!(Some(&Int(1)), scheme_match!(&pair, {x}, (?x . _)));
        assert_eq!(Some(&Int(2)), scheme_match!(&pair, {y}, (_ . ?y)));
    }

    #[test]
    #[should_panic(expected = "Last clause in switch must never fail to match")]
    fn switch_error() {
        switch! { &Int(5),
            [4] => { }
        };
    }

    #[test]
    fn switch_one_clause() {
        assert_eq!(
            switch! { &Int(5),
                [?x] => { x }
            },
            &Int(5)
        );

        assert_eq!(
            switch! { &Nil,
                [_] => { 42 }
            },
            42
        );
    }

    #[test]
    #[should_panic(expected = "Last clause in switch must never fail to match")]
    fn switch_n_error() {
        switch! { &Int(5),
            [1] => { },
            [2] => { },
            [3] => { },
        };
    }

    #[test]
    fn switch_n_clauses() {
        assert_eq!(
            switch! { &Int(5),
                [4] => 42,
                [_] => 0
            },
            0
        );
    }

    #[test]
    fn switch_predicate() {
        assert_eq!(
            switch! { &Int(4),
                |_| false => 1,
                |_| true => 2,
            },
            2
        )
    }
}
