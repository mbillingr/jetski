use crate::object::Object;
use crate::parser::Rule;
use pest::error::Error as PestError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    PestError(PestError<Rule>),
    InvalidNumericConstant(String),
    UnknownExpressionType(Object),

    NotAPair(Object),
    SyntaxError(String),
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error { kind }
    }
}

impl From<PestError<Rule>> for Error {
    fn from(pe: PestError<Rule>) -> Self {
        ErrorKind::PestError(pe).into()
    }
}
