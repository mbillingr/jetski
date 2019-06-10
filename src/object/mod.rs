mod constructors;

pub use constructors::ListBuilder;

#[derive(Debug)]
pub struct Object {
    content: TaggedValue,
}

impl Object {
    pub fn new(content: TaggedValue) -> Self {
        Object { content }
    }
}

#[derive(Debug)]
pub enum TaggedValue {
    Integer(i64),
    Float(f64),
    Symbol(String), // todo: use interned symbols
    String(String),
    List(Vec<Object>, Option<Box<Object>>),
}
