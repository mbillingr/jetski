use super::{Object, TaggedValue};

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

    pub fn is_integer(&self) -> bool {
        match self.content {
            TaggedValue::Integer(_) => true,
            _ => false,
        }
    }

    pub fn try_as_integer(&self) -> Option<i64> {
        match self.content {
            TaggedValue::Integer(i) => Some(i),
            _ => None,
        }
    }

    pub fn is_float(&self) -> bool {
        match self.content {
            TaggedValue::Float(_) => true,
            _ => false,
        }
    }

    pub fn try_as_float(&self) -> Option<f64> {
        match self.content {
            TaggedValue::Float(f) => Some(f),
            _ => None,
        }
    }

    pub fn is_symbol(&self) -> bool {
        match self.content {
            TaggedValue::Symbol(_) => true,
            _ => false,
        }
    }

    pub fn try_as_symbol_name(&self) -> Option<&str> {
        match self.content {
            TaggedValue::Symbol(ref name) => Some(name),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        match self.content {
            TaggedValue::String(_) => true,
            _ => false,
        }
    }

    pub fn is_list(&self) -> bool {
        match self.content {
            TaggedValue::List(_, _) => true,
            _ => false,
        }
    }

    pub fn try_as_slice(&self) -> Option<&[Object]> {
        match self.content {
            TaggedValue::List(ref list, _) => Some(list),
            _ => None,
        }
    }

    pub fn car(&self) -> Option<&Object> {
        match self.content {
            TaggedValue::List(ref list, _) => Some(&list[0]),
            _ => None,
        }
    }

    pub fn get_ref(&self, idx: usize) -> Option<&Object> {
        match self.content {
            TaggedValue::List(ref list, _) => Some(&list[idx]),
            _ => None,
        }
    }

    pub fn list_len(&self) -> Option<usize> {
        match self.content {
            TaggedValue::Nil => Some(0),
            TaggedValue::List(ref list, _) => Some(list.len()),
            _ => None,
        }
    }
}
