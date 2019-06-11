mod constructors;
mod formatting;
mod primitive_methods;

pub use constructors::ListBuilder;

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

#[derive(Debug)]
pub enum TaggedValue {
    Nil,
    Integer(i64),
    Float(f64),
    Symbol(String), // todo: use interned symbols
    String(String),
    List(Vec<Object>, Option<Box<Object>>),
}
