use super::parser::Rule;

/// The result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// The error type for this crate.
#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "{}", 0)]
    PestError(#[error(cause)] pest::error::Error<Rule>),

    #[error(display = "{}", 0)]
    IOError(#[error(cause)] std::io::Error),

    #[error(display = "invalid prefix: {:?}", 0)]
    InvalidPrefixError(curie::InvalidPrefixError),

    #[error(display = "expansion error: {:?}", 0)]
    ExpansionError(curie::ExpansionError),

    #[error(display = "invalid facet: {}", 0)]
    InvalidFacet(String),
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(e: pest::error::Error<Rule>) -> Self {
        Error::PestError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(e)
    }
}

impl From<curie::InvalidPrefixError> for Error {
    fn from(e: curie::InvalidPrefixError) -> Self {
        Error::InvalidPrefixError(e)
    }
}

impl From<curie::ExpansionError> for Error {
    fn from(e: curie::ExpansionError) -> Self {
        Error::ExpansionError(e)
    }
}
