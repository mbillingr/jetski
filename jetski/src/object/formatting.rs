use super::{Object, TaggedValue};

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use TaggedValue::*;
        match &self.content {
            Nil => write!(f, "'()"),
            Undef => write!(f, "<undefined>"),
            Integer(x) => write!(f, "{}", x),
            Float(x) => write!(f, "{}", x),
            Symbol(s) => write!(f, "{}", s),
            String(s) => write!(f, "{:?}", s),
            Function(_) => write!(f, "<function>"),
            Pair(car, cdr) => {
                let mut cdr = &**cdr;
                write!(f, "(")?;
                write!(f, "{}", car)?;
                while let Pair(a, d) = &cdr.content {
                    write!(f, " {}", a)?;
                    cdr = &**d;
                }
                if !cdr.is_null() {
                    write!(f, " . {}", cdr)?;
                }
                write!(f, ")")
            }
        }
    }
}
