use std::collections::BTreeSet;

use curie::PrefixMapping;
use horned_owl::model::*;
use horned_owl::ontology::set::SetOntology;

use crate::error::Error;
use crate::error::Result;
use crate::from_pair::FromPair;
use crate::parser::OwlFunctionalParser;
use crate::Context;

/// A trait for OWL elements that can be deserialized from OWL Functional syntax.
///
/// The deserialization will fail if the entirety of the input string cannot
/// be deserialized into the declared type.
pub trait FromFunctional: Sized + FromPair {
    /// Deserialize a string containing an OWL element in functional syntax.
    #[inline]
    fn from_ofn(s: &str) -> Result<Self> {
        Self::from_ofn_ctx(s, &Context::default())
    }

    fn from_ofn_ctx(s: &str, context: &Context<'_>) -> Result<Self>;
}

// We use a macro instead of a blanket impl to have all types displayed in
// the documentation.
macro_rules! implement {
    ($($ty:ty),+) => {
        $(impl FromFunctional for $ty {
            fn from_ofn_ctx(s: &str, context: &Context<'_>) -> Result<Self> {
                let mut pairs = OwlFunctionalParser::parse(Self::RULE, s)?;
                if pairs.as_str().len() == s.len() {
                     Self::from_pair(pairs.next().unwrap(), context)
                } else {
                    Err(
                        Error::from(
                            pest::error::Error::new_from_span(
                                pest::error::ErrorVariant::CustomError {
                                    message: "remaining input".to_string(),
                                },
                                pest::Span::new(s, pairs.as_str().len(), s.len()).unwrap()
                            )
                        )
                    )
                }
            }
        })*
    }
}

implement!(
    AnnotationProperty,
    AnnotatedAxiom,
    Annotation,
    AnnotationValue,
    AnonymousIndividual,
    Axiom,
    BTreeSet<Annotation>,
    Class,
    ClassExpression,
    DataProperty,
    DataRange,
    Datatype,
    DeclareClass,
    DeclareDatatype,
    DeclareObjectProperty,
    DeclareDataProperty,
    DeclareAnnotationProperty,
    DeclareNamedIndividual,
    Facet,
    FacetRestriction,
    Import,
    Individual,
    IRI,
    NamedIndividual,
    Literal,
    ObjectPropertyExpression,
    ObjectProperty,
    SetOntology,
    OntologyAnnotation,
    (SetOntology, PrefixMapping),
    String,
    SubObjectPropertyExpression,
    u32
);

#[cfg(test)]
mod tests {

    use super::*;
    use horned_owl::model::DeclareClass;

    #[test]
    fn test_remaining_input() {
        match DeclareClass::from_ofn("Class(<http://example.com/a>) Class(<http://example.com/b>)")
        {
            Ok(ok) => panic!("unexpected success: {:?}", ok),
            Err(Error::PestError(e)) => {
                assert_eq!(
                    e.variant,
                    pest::error::ErrorVariant::CustomError {
                        message: "remaining input".to_string(),
                    }
                )
            }
            Err(other) => panic!("unexpected error: {:?}", other),
        }
    }
}
