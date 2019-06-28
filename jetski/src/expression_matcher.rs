#[macro_export]
macro_rules! match_template {
    ($exp:expr, $action:block, [$single:tt]) => {
        $exp.cdr()
            .filter(|cdr| cdr.is_nil())
            .and_then(|_| $exp.car())
            .and_then(|car| match_template!(car, $action, $single))
    };

    ($exp:expr, $action:block, [$single:tt, ..]) => {
        $exp.car().and_then(|car| {
            match_template!(car, $action, $single)
        })
    };

    ($exp:expr, $action:block, [$single:tt, .. $($tail:tt)*]) => {
        $exp.decons().map(|(car, cdr)|{
            match_template!(car, {
                match_template!(cdr, $action, $($tail)*)
            }, $single).unwrap_or(None)
        }).unwrap_or(None)
    };

    ($exp:expr, $action:block, [$single_variant:tt $single_value:tt, ..]) => {
        $exp.car().and_then(|car| {
            match_template!(car, $action, $single_variant $single_value)
        })
    };

    ($exp:expr, $action:block, [$single_variant:tt $single_value:tt, .. $($tail:tt)*]) => {
        $exp.decons().map(|(car, cdr)|{
            match_template!(car, {
                match_template!(cdr, $action, $($tail)*)
            }, $single_variant $single_value).unwrap_or(None)
        }).unwrap_or(None)
    };

    ($exp:expr, $action:block, [$single_variant:tt $single_value:tt]) => {
        $exp.cdr()
            .filter(|cdr| cdr.is_nil())
            .and_then(|_| $exp.car())
            .and_then(|car| match_template!(car, $action, $single_variant $single_value))
    };

    ($exp:expr, $action:block, [$first:tt, $($rest:tt)*]) => {
        $exp.car()
            .and_then(|car| match_template!(car, {
                match_template!($exp.cdr().unwrap(), $action, [$($rest)*])
            }, $first))
            .unwrap_or(None)
    };

    ($exp:expr, $action:block, [$first_variant:tt $first_value:tt, $($rest:tt)*]) => {
        $exp.car()
            .and_then(|car| match_template!(car, {
                match_template!($exp.cdr().unwrap(), $action, [$($rest)*])
            }, $first_variant $first_value))
            .unwrap_or(None)
    };

    ($exp:expr, $action:block, $atom:pat) => {
        if let $atom = $exp {
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

    #[test]
    fn simple_match_constant() {
        assert_eq!(Some(()), match_template!(&Symbol("xyz"), {}, Symbol("xyz")));
        assert_eq!(Some(()), match_template!(&Int(42), {}, Int(42)));
    }

    #[test]
    fn simple_mismatch_constant() {
        assert_eq!(None, match_template!(&Symbol("abc"), {}, Symbol("xyz")));
        assert_eq!(None, match_template!(&Symbol("abc"), {}, Int(1)));
        assert_eq!(None, match_template!(&Int(42), {}, Symbol("xyz")));
        assert_eq!(None, match_template!(&Int(42), {}, Int(2)));
    }

    #[test]
    fn simple_match_any() {
        assert_eq!(Some(()), match_template!(&Symbol("xyz"), {}, _));
    }

    #[test]
    fn simple_match_any_bound() {
        assert_eq!(Some(&Symbol("y")), match_template!(&Symbol("y"), { x }, x));
        assert_eq!(Some(&Int(42)), match_template!(&Int(42), { x }, x));
    }

    #[test]
    fn simple_match_variant_bound() {
        assert_eq!(Some("y"), match_template!(&Symbol("y"), { *x }, Symbol(x)));
        assert_eq!(Some(42), match_template!(&Int(42), { *y }, Int(y)));
    }

    #[test]
    fn unarylist_mismatch() {
        assert_eq!(None, match_template!(&Symbol("abc"), {}, [_]));
        let pair = Expr::cons(Int(1), Int(2));
        assert_eq!(None, match_template!(&pair, {}, [_]));
    }

    #[test]
    fn unarylist_match_any() {
        let pair = Expr::cons(Int(5), Nil);
        assert_eq!(Some(()), match_template!(&pair, {}, [_]));
    }

    #[test]
    fn unarylist_match_any_bound() {
        let pair = Expr::cons(Int(5), Nil);
        assert_eq!(Some(&Int(5)), match_template!(&pair, { x }, [x]));
    }

    #[test]
    fn unarylist_match_variant_bound() {
        let pair = Expr::cons(Symbol("hello"), Nil);
        assert_eq!(Some("hello"), match_template!(&pair, { *x }, [Symbol(x)]));
    }

    #[test]
    fn list_mismatch() {
        assert_eq!(None, match_template!(&Symbol("abc"), {}, [_, _]));
        let pair = Expr::cons(Int(1), Nil);
        assert_eq!(None, match_template!(&pair, {}, [_, _]));
    }

    #[test]
    fn list_match_any() {
        let list = Expr::cons(Int(1), Expr::cons(Int(2), Expr::cons(Int(3), Nil)));
        assert_eq!(Some(()), match_template!(&list, {}, [_, _, _]));
    }

    #[test]
    fn list_match_any_bound() {
        let list = Expr::cons(Int(1), Expr::cons(Int(2), Expr::cons(Int(3), Nil)));
        assert_eq!(
            Some((&Int(1), &Int(2), &Int(3))),
            match_template!(&list, { (x, y, z) }, [x, y, z])
        );
    }

    #[test]
    fn list_match_variant_bound() {
        let list = Expr::cons(Int(1), Expr::cons(Symbol("two"), Expr::cons(Int(3), Nil)));
        assert_eq!(
            Some((1, "two", 3)),
            match_template!(&list, { (*x, *y, *z) }, [Int(x), Symbol(y), Int(z)])
        );
    }

    #[test]
    fn nested_list() {
        let list = Expr::cons(
            Int(1),
            Expr::cons(
                Expr::cons(Symbol("2a"), Expr::cons(Symbol("2b"), Nil)),
                Expr::cons(Int(3), Nil),
            ),
        );
        assert_eq!(
            Some((&Int(1), "2b")),
            match_template!(&list, { (x, *y) }, [x, [Symbol("2a"), Symbol(y)], _])
        );
    }

    #[test]
    fn list_match_tail() {
        let list = Expr::cons(
            Int(0),
            Expr::cons(Int(1), Expr::cons(Int(2), Expr::cons(Int(3), Nil))),
        );
        assert_eq!(
            Some(&Expr::cons(Int(2), Expr::cons(Int(3), Nil))),
            match_template!(&list, { rest }, [Int(0), Int(1), ..rest])
        );
        assert_eq!(
            Some(&Expr::cons(Int(2), Expr::cons(Int(3), Nil))),
            match_template!(&list, { rest }, [_, _, ..rest])
        );
    }

    #[test]
    fn list_match_ignore_tail() {
        let list = Expr::cons(
            Int(0),
            Expr::cons(Int(1), Expr::cons(Int(2), Expr::cons(Int(3), Nil))),
        );
        assert_eq!(
            Some(1),
            match_template!(&list, { *x }, [Int(0), Int(x), ..])
        );
        assert_eq!(Some(&Int(1)), match_template!(&list, { one }, [_, one, ..]));
    }

    #[test]
    fn pair_match() {
        let list = Expr::cons(Int(0), Int(1));
        assert_eq!(
            Some((0, 1)),
            match_template!(&list, { (*a, *b) }, [Int(a), ..Int(b)])
        );
        assert_eq!(
            Some((&Int(0), &Int(1))),
            match_template!(&list, { (a, b) }, [a, ..b])
        );
    }
}
