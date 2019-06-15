pub mod block;
pub mod builder;
pub mod context;
pub mod function;
pub mod jit;
pub mod module;
pub mod types;
pub mod value;

pub use block::Block;
pub use builder::Builder;
pub use context::Context;
pub use function::Function;
pub use module::Module;
pub use types::Type;
pub use value::Value;

use std::ffi::CString;

fn str_to_cstring(s: &str) -> CString {
    CString::new(s).unwrap()
}
