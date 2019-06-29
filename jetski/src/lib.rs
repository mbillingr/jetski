extern crate pest;

#[macro_use]
extern crate pest_derive;
#[macro_use]
pub mod syntax;

mod error;
pub mod jit;
mod object;
pub mod parser;
pub mod runtime;
pub mod transformations;

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
    fn car_mut(&mut self) -> Option<&mut Self> {
        unimplemented!()
    }
    fn cdr_mut(&mut self) -> Option<&mut Self> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
