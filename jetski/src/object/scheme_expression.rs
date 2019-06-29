use super::{Object, TaggedValue};

impl crate::SchemeExpression for Object {
    fn is_nil(&self) -> bool {
        self.is_null()
    }

    fn symbol_name(&self) -> Option<&'static str> {
        match self.content {
            TaggedValue::Symbol(s) => Some(s.name()),
            _ => None,
        }
    }

    fn car(&self) -> Option<&Self> {
        match self.content {
            TaggedValue::Pair(ref a, _) => Some(a),
            _ => None,
        }
    }

    fn cdr(&self) -> Option<&Self> {
        match self.content {
            TaggedValue::Pair(_, ref d) => Some(d),
            _ => None,
        }
    }

    fn car_mut(&mut self) -> Option<&mut Self> {
        match self.content {
            TaggedValue::Pair(ref mut a, _) => Some(a),
            _ => None,
        }
    }

    fn cdr_mut(&mut self) -> Option<&mut Self> {
        match self.content {
            TaggedValue::Pair(_, ref mut d) => Some(d),
            _ => None,
        }
    }
}
