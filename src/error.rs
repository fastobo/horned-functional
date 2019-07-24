use super::parser::Rule;

/// The result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// The error type for this crate.
#[derive(Debug, Error)]
pub enum Error {
    /// An error that occurred at the `pest` parser level.
    ///
    /// This is returned from any parsing methods when the input is written
    /// with invalid syntax, or when attempting to parse an incomplete input.
    ///
    /// # Example:
    /// ```rust
    /// # #[macro_use] extern crate matches;
    /// # extern crate horned_owl;
    /// # use horned_owl::model::Ontology;
    /// use horned_functional::FromFunctional;
    ///
    /// let res = Ontology::from_ofn_str("Ontology(");
    /// assert_matches!(res, Err(horned_functional::Error::PestError(_)));
    /// ```
    #[error(display = "{}", 0)]
    PestError(#[error(cause)] pest::error::Error<Rule>),

    /// An error that happened at the I/O level.
    ///
    /// # Example:
    /// ```rust
    /// # #[macro_use] extern crate matches;
    /// let res = horned_functional::from_file("/some/missing/file").map(|x| x.0);
    /// assert_matches!(res, Err(horned_functional::Error::IOError(_)));
    /// ```
    #[error(display = "{}", 0)]
    IOError(#[error(cause)] std::io::Error),

    /// A CURIE expansion went wrong.
    ///
    /// This error can be encountered in documents where a CURIE used an
    /// undefined prefix.
    ///
    /// # Example
    /// ```rust
    /// # #[macro_use] extern crate matches;
    /// # extern crate horned_owl;
    /// # use horned_owl::model::IRI;
    /// use horned_functional::FromFunctional;
    ///
    /// let res = IRI::from_ofn_str("example:Entity");
    /// assert_matches!(res, Err(horned_functional::Error::ExpansionError(_)));
    /// ```
    #[error(display = "expansion error: {:?}", 0)]
    ExpansionError(curie::ExpansionError),

    /// An unknown IRI was used as a facet.
    ///
    /// # Example
    /// ```rust
    /// # #[macro_use] extern crate matches;
    /// # use horned_owl::model::Facet;
    /// use horned_functional::FromFunctional;
    ///
    /// let res = Facet::from_ofn_str("<http://example.com/thing>");
    /// assert_matches!(res, Err(horned_functional::Error::InvalidFacet(_)));
    /// ```
    #[error(display = "invalid facet: {}", 0)]
    InvalidFacet(String),

    /// An unsupported construct was used.
    ///
    /// See the relevant issue for each unsupported syntax construct; if the
    /// issue has been resolved, please open an issue on the
    /// [`fastobo/horned-functional`](https://github.com/fastobo/horned-functional)
    /// repository so that it can be fixed.
    #[error(display = "unsupported: {} (see {})", 0, 1)]
    Unsupported(&'static str, &'static str),
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

impl From<curie::ExpansionError> for Error {
    fn from(e: curie::ExpansionError) -> Self {
        Error::ExpansionError(e)
    }
}
