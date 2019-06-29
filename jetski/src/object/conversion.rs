use super::{Object, TaggedValue};
use crate::runtime::Symbol;
use crate::SchemeExpression;

macro_rules! impl_from {
    ($T:ty, $as:ty, $constructor:path) => {
        impl From<$T> for Object {
            fn from(x: $T) -> Object {
                $constructor(x as $as)
            }
        }
    };
}

impl_from!(i64, i64, Object::integer);
impl_from!(i32, i64, Object::integer);
impl_from!(i16, i64, Object::integer);
impl_from!(i8, i64, Object::integer);
impl_from!(u32, i64, Object::integer);
impl_from!(u16, i64, Object::integer);
impl_from!(u8, i64, Object::integer);

impl_from!(f64, f64, Object::float);
impl_from!(f32, f64, Object::float);

impl_from!(Symbol, Symbol, Object::symbol);
