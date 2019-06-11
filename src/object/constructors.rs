use super::{Object, TaggedValue};

impl Object {
    pub fn integer(value: i64) -> Self {
        Object::new(TaggedValue::Integer(value))
    }

    pub fn float(value: f64) -> Self {
        Object::new(TaggedValue::Float(value))
    }

    pub fn symbol(name: &str) -> Self {
        Object::new(TaggedValue::Symbol(name.to_owned()))
    }

    pub fn string(content: String) -> Self {
        Object::new(TaggedValue::String(content))
    }
}

pub struct ListBuilder {
    partial_list: Vec<Object>,
    last_cdr: Option<Box<Object>>,
}

impl ListBuilder {
    pub fn new() -> Self {
        ListBuilder {
            partial_list: vec![],
            last_cdr: None,
        }
    }

    pub fn append(&mut self, item: Object) {
        self.partial_list.push(item);
    }

    pub fn set_cdr(&mut self, item: Object) {
        match item.content {
            TaggedValue::List(list, cdr) => {
                self.partial_list.extend(list);
                self.last_cdr = cdr;
            }
            _ => self.last_cdr = Some(Box::new(item)),
        }
    }

    pub fn build(self) -> Object {
        if self.partial_list.is_empty() {
            Object::new(TaggedValue::Nil)
        } else {
            Object::new(TaggedValue::List(self.partial_list, self.last_cdr))
        }
    }
}
