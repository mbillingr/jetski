pub mod alphatize;

use crate::error::Result;
use crate::Object;

pub trait SourceTransformer {
    fn transform(&mut self, source: &Object) -> Result<Object>;
}
