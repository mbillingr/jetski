macro_rules! cons {
    ($car:ident, $($cdr:tt)*) => {
        Object::cons(primitive!($car), primitive!($($cdr)*))
    };

    ($car:expr, $($cdr:tt)*) => {
        Object::cons(primitive!($car), primitive!($($cdr)*))
    };

    (@$car:expr, $($cdr:tt)*) => {
        Object::cons(primitive!($car), primitive!($($cdr)*))
    };
}

macro_rules! list {
    (($($inner:tt)*), . $($cdr:tt)*) => { cons!(list!($($inner)*), $($rest)*) };
    (($($inner:tt)*), $($rest:tt)*) => { Object::cons(list!($($inner)*), list!($($rest)*)) };
    (($($inner:tt)*)) => { Object::cons(list!($($inner)*), primitive!(nil)) };

    ($car:ident, . $($cdr:tt)*) => { cons!(primitive!($car), $($cdr)*) };
    ($car:expr, . $($cdr:tt)*) => { cons!(primitive!($car), $($cdr)*) };
    (@$car:expr, . $($cdr:tt)*) => { cons!(primitive!($car), $($cdr)*) };
    ($car:ident, $($rest:tt)*) => { Object::cons(primitive!($car), list!($($rest)*)) };
    ($car:expr, $($rest:tt)*) => { Object::cons(primitive!($car), list!($($rest)*)) };
    (@$car:expr, $($rest:tt)*) => { Object::cons(primitive!($car), list!($($rest)*)) };

    ($car:ident) => { Object::cons(primitive!($car), primitive!(nil)) };
    ($car:expr) => { Object::cons(primitive!($car), primitive!(nil)) };
    (@$car:expr) => { Object::cons(primitive!($car), primitive!(nil)) };
}

macro_rules! primitive {
    (nil) => {
        Object::nil()
    };

    ($x:ident) => {
        Object::symbol(stringify!($x))
    };

    ($x:expr) => {
        Object::from($x)
    };

    (@$x:expr) => {
        Object::from($x)
    };
}

#[cfg(test)]
mod tests {
    use crate::object::Object;
    use crate::SchemeExpression;

    #[test]
    fn cons_symbols() {
        assert_eq!(
            cons!(abc, xyz),
            Object::cons(Object::symbol("abc"), Object::symbol("xyz"))
        )
    }

    #[test]
    fn cons_numbers() {
        assert_eq!(
            cons!(2, 1),
            Object::cons(Object::integer(2), Object::integer(1))
        )
    }

    #[test]
    fn cons_object() {
        assert_eq!(
            cons!(Object::nil(), Object::nil()),
            Object::cons(Object::nil(), Object::nil())
        )
    }

    #[test]
    fn cons_mixed() {
        assert_eq!(
            cons!(abc, 123),
            Object::cons(Object::symbol("abc"), Object::integer(123))
        );

        assert_eq!(
            cons!(123, abc),
            Object::cons(Object::integer(123), Object::symbol("abc"))
        );
    }

    #[test]
    fn cons_variables() {
        let a = 42;
        let b = Object::nil();
        assert_eq!(
            cons!(@a, @b),
            Object::cons(Object::integer(42), Object::nil())
        )
    }

    #[test]
    fn lists() {
        let x = 3;
        assert_eq!(
            list!(Object::nil(), one, 2, @x),
            Object::cons(
                Object::nil(),
                Object::cons(
                    Object::symbol("one"),
                    Object::cons(
                        Object::integer(2),
                        Object::cons(Object::integer(3), Object::nil())
                    )
                )
            )
        );
    }

    #[test]
    fn nested_lists() {
        let x = 3;
        assert_eq!(
            list!((Object::nil(), one), (2, @x)),
            Object::cons(
                Object::cons(
                    Object::nil(),
                    Object::cons(Object::symbol("one"), Object::nil())
                ),
                Object::cons(
                    Object::cons(
                        Object::integer(2),
                        Object::cons(Object::integer(3), Object::nil())
                    ),
                    Object::nil()
                )
            )
        );
    }

    #[test]
    fn stitched_lists() {
        let a = list!(3, 4);
        assert_eq!(list!(1, 2, . @a), list!(1, 2, 3, 4));

        let a = list!(1, 2);
        let b = list!(3, 4);
        assert_eq!(list!(@a, . @b), list!((1, 2), 3, 4));
    }
}
