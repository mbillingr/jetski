mod algorithms;
mod constructors;
mod conversion;
mod formatting;
mod primitive_methods;
mod scheme_expression;

use crate::runtime::Symbol;
pub use constructors::ListBuilder;

#[derive(Clone, PartialEq)]
pub struct Object {
    content: TaggedValue,
}

impl Object {
    pub fn new(content: TaggedValue) -> Self {
        Object { content }
    }

    pub fn as_value(&self) -> &TaggedValue {
        &self.content
    }

    pub fn into_value(&self) -> &TaggedValue {
        &self.content
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaggedValue {
    Undef,
    Nil,
    Integer(i64),
    Float(f64),
    Symbol(Symbol),
    String(String),
    Pair(Box<Object>, Box<Object>),
    Function(*const u8),
}
