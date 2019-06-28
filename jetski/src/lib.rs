extern crate pest;

#[macro_use]
extern crate pest_derive;

mod error;
#[macro_use]
pub mod expression_matcher;
#[macro_use]
pub mod scheme_matcher;
pub mod jit;
mod object;
pub mod parser;

pub use error::*;
pub use object::Object;

// TODO: I'm not yet sure where this trait should live...
pub trait SchemeExpression {
    fn is_nil(&self) -> bool;

    fn symbol_name(&self) -> Option<&'static str>;

    fn car(&self) -> Option<&Self>;
    fn cdr(&self) -> Option<&Self>;
    fn decons(&self) -> Option<(&Self, &Self)> {
        self.car().map(|a| (a, self.cdr().unwrap()))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
