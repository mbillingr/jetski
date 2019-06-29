use lazy_static::lazy_static;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::sync::Mutex;

lazy_static! {
    static ref STATIC_NAMES: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());
}

fn static_name<T: AsRef<str> + ToString>(name: T) -> &'static str {
    let mut container = STATIC_NAMES.lock().unwrap();
    let s = match container.get(name.as_ref()) {
        Some(s) => s.as_str(),
        None => {
            container.insert(name.to_string());
            container.get(name.as_ref()).unwrap()
        }
    };

    unsafe {
        // We transmute from &str to &'static str.
        // This should be safe if
        //  1. The string data is never moved in memory
        //  2. The string data is never deallocated: **never** remove a string from STATIC_NAMES
        std::mem::transmute(s)
    }
}

#[derive(Copy, Clone)]
pub struct Symbol {
    name: &'static str,
}

impl Symbol {
    pub fn new<T: AsRef<str> + ToString>(name: T) -> Self {
        Symbol {
            name: static_name(name),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn id(&self) -> usize {
        self.name as *const _ as *const u8 as usize
    }
}

impl From<&str> for Symbol {
    fn from(s: &str) -> Self {
        Symbol::new(s)
    }
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        self.name
    }
}

impl PartialEq<Symbol> for Symbol {
    fn eq(&self, s: &Symbol) -> bool {
        self.id() == s.id()
    }
}

impl Eq for Symbol {}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Since symbols are supposed to be identifiable by pointer
        // we can hash the address rather than the whole string.
        let id = self.name as *const _ as *const u8 as usize;
        id.hash(state);
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::cmp::PartialOrd for Symbol {
    /// We impose an arbitrary order on symbols, based on their static address.
    /// This is in line with the notion that symbols should be comparable by pointer.
    /// It may be necessary to switch to lexical ordering in the future.
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        self.id().partial_cmp(&rhs.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assure_reallocation() {
        for i in 0..100 {
            static_name(format!("{}", i));
        }
    }

    #[test]
    fn static_names_are_unique() {
        let a1 = static_name("A");
        let b1 = static_name("B");

        assure_reallocation();

        let a2 = static_name("A");
        let b2 = static_name("B");

        assert_eq!(a1.as_ptr(), a2.as_ptr());
        assert_eq!(b1.as_ptr(), b2.as_ptr());
        assert_ne!(a1.as_ptr(), b1.as_ptr());
    }

    #[test]
    fn symbols_are_unique() {
        let a1 = Symbol::new("A");
        let b1 = Symbol::new("B");
        let a2 = Symbol::new("A");
        let b2 = Symbol::new("B");

        assert_eq!(a1.id(), a2.id());
        assert_eq!(b1.id(), b2.id());
        assert_ne!(a1.id(), b1.id());
    }

    #[test]
    fn symbol_equivalence() {
        let a1 = Symbol::new("A");
        let b1 = Symbol::new("B");
        let a2 = Symbol::new("A");
        let b2 = Symbol::new("B");

        assert_eq!(a1, a2);
        assert_eq!(b1, b2);
        assert_ne!(a1, b1);
    }

    #[test]
    fn symbol_order_is_consistent() {
        let a1 = Symbol::new("A");
        let b1 = Symbol::new("B");
        let a2 = Symbol::new("A");
        let b2 = Symbol::new("B");

        assert_eq!(a1.partial_cmp(&b1), a2.partial_cmp(&b2));
    }
}
