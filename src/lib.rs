extern crate pest;

#[macro_use]
extern crate pest_derive;

mod error;
mod object;
pub mod parser;

pub use error::*;
pub use object::Object;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}