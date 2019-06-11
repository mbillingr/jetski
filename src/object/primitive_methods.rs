use super::{Object, TaggedValue};
use crate::error::Result;

impl Object {
    pub fn is_null(&self) -> bool {
        match self.content {
            TaggedValue::Nil => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self.content {
            TaggedValue::Integer(_) | TaggedValue::Float(_) => true,
            _ => false,
        }
    }

    pub fn is_symbol(&self) -> bool {
        match self.content {
            TaggedValue::Symbol(_) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self.content {
            TaggedValue::String(_) => true,
            _ => false,
        }
    }

    pub fn car(&self) -> Result<&Object> {
        match self.content {
            TaggedValue::List(ref list, _) => Ok(&list[0]),
            _ => unimplemented!()
        }
    }
}
