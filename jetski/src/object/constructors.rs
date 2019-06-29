use super::{Object, TaggedValue};
use crate::runtime::Symbol;
use crate::SchemeExpression;

impl Object {
    pub fn undef() -> Self {
        Object::new(TaggedValue::Undef)
    }
    pub fn nil() -> Self {
        Object::new(TaggedValue::Nil)
    }

    pub fn integer(value: i64) -> Self {
        Object::new(TaggedValue::Integer(value))
    }

    pub fn float(value: f64) -> Self {
        Object::new(TaggedValue::Float(value))
    }

    pub fn symbol<T: AsRef<str> + ToString>(name: T) -> Self {
        Object::new(TaggedValue::Symbol(Symbol::new(name)))
    }

    pub fn string(content: String) -> Self {
        Object::new(TaggedValue::String(content))
    }

    pub fn function(ptr: *const u8) -> Self {
        Object::new(TaggedValue::Function(ptr))
    }

    pub fn cons(car: Object, cdr: Object) -> Self {
        Object::new(TaggedValue::Pair(Box::new(car), Box::new(cdr)))
    }
}

pub struct ListBuilder {
    partial_list: Box<Object>,
    cursor: *mut Object,
}

impl ListBuilder {
    pub fn new() -> Self {
        let mut builder = ListBuilder {
            partial_list: Box::new(Object::nil()),
            cursor: 0 as *mut _,
        };
        builder.cursor = builder.partial_list.as_mut();
        builder
    }

    pub fn append(&mut self, item: Object) {
        unsafe {
            *self.cursor = Object::cons(item, Object::nil());
            self.cursor = (*self.cursor).cdr_mut().unwrap();
        }
    }

    pub fn set_cdr(&mut self, item: Object) {
        unsafe {
            *self.cursor = item;
        }
    }

    pub fn build(self) -> Object {
        *self.partial_list
    }
}
