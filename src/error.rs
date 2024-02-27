use pest::Span;

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
    /// # use horned_owl::ontology::set::SetOntology;
    /// use horned_functional::FromFunctional;
    ///
    /// let res = SetOntology::<String>::from_ofn("Ontology(");
    /// assert_matches!(res, Err(horned_functional::Error::Pest(_)));
    /// ```
    #[error(transparent)]
    Pest(Box<pest::error::Error<Rule>>),

    /// An error that happened at the I/O level.
    ///
    /// # Example:
    /// ```rust
    /// # #[macro_use] extern crate matches;
    /// # use horned_owl::ontology::set::SetOntology;
    /// let res = horned_functional::from_file::<_, SetOntology::<String>, _>("/some/missing/file")
    ///     .map(|x| x.0);
    /// assert_matches!(res, Err(horned_functional::Error::IO(_)));
    /// ```
    #[error(transparent)]
    IO(#[from] std::io::Error),

    /// A CURIE expansion went wrong.
    ///
    /// This error can be encountered in documents where a CURIE used an
    /// undefined prefix, or when attempting to parse an abbreviated IRI
    /// without providing a prefix mapping.
    ///
    /// # Example
    /// ```rust
    /// # #[macro_use] extern crate matches;
    /// # extern crate horned_owl;
    /// # use horned_owl::model::IRI;
    /// use horned_functional::FromFunctional;
    ///
    /// let res = IRI::<String>::from_ofn("example:Entity");
    /// assert_matches!(res, Err(horned_functional::Error::Expansion(_)));
    /// ```
    #[error("expansion error: {0:?}")]
    Expansion(curie::ExpansionError),

    /// An unknown IRI was used as a facet.
    ///
    /// # Example
    /// ```rust
    /// # #[macro_use] extern crate matches;
    /// # use horned_owl::model::Facet;
    /// use horned_functional::FromFunctional;
    ///
    /// let res = <Facet as FromFunctional<String>>::from_ofn("<http://example.com/thing>");
    /// assert_matches!(res, Err(horned_functional::Error::InvalidFacet(_)));
    /// ```
    #[error("invalid facet: {0}")]
    InvalidFacet(String),
}

impl Error {
    /// Create a custom `pest` error located at the given span.
    pub fn custom<S: Into<String>>(message: S, span: Span) -> Self {
        Self::from(pest::error::Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: message.into(),
            },
            span,
        ))
    }
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(e: pest::error::Error<Rule>) -> Self {
        Error::Pest(Box::new(e))
    }
}

impl From<curie::ExpansionError> for Error {
    fn from(e: curie::ExpansionError) -> Self {
        Error::Expansion(e)
    }
}
