use super::{Object, TaggedValue};
use crate::error::{ErrorKind, Result};
use crate::SchemeExpression;

impl Object {
    pub fn get_ref(&self, idx: usize) -> Option<&Object> {
        if idx == 0 {
            self.car()
        } else {
            self.cdr().and_then(|cdr| cdr.get_ref(idx - 1))
        }
    }

    pub fn list_len(&self) -> Option<usize> {
        let mut len = 0;
        let mut x = self;
        while !x.is_nil() {
            x = x.car()?;
            len += 1;
        }
        Some(len)
    }

    pub fn map<F: FnMut(&Self) -> Result<Self>>(&self, mut op: F) -> Result<Self> {
        if self.is_nil() {
            Ok(Object::nil())
        } else {
            self.car()
                .ok_or_else(|| ErrorKind::NotAPair(self.clone()).into())
                .and_then(|car| op(car))
                .and_then(|new_car| Ok(Object::cons(new_car, self.cdr().unwrap().map(op)?)))
        }
    }
}
