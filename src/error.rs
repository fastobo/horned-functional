use super::parser::Rule;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", 0)]
    PestError(#[cause] pest::error::Error<Rule>),

    #[fail(display = "{}", 0)]
    IOError(#[cause] std::io::Error),

    #[fail(display = "invalid prefix: {:?}", 0)]
    InvalidPrefixError(curie::InvalidPrefixError),

    #[fail(display = "expansion error: {:?}", 0)]
    ExpansionError(curie::ExpansionError),
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
