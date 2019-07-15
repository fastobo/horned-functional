#[macro_use]
extern crate err_derive;
#[macro_use]
extern crate pest_derive;

extern crate curie;
extern crate horned_owl;
extern crate pest;

pub mod from_pair;
pub mod error;
pub mod parser;

use curie::PrefixMapping;
use horned_owl::model::Ontology;
use horned_owl::model::Build;

use self::parser::OwlFunctionalParser;
use self::parser::Rule;

#[doc(inline)]
pub use self::from_pair::FromPair;
#[doc(inline)]
pub use self::error::Error;
#[doc(inline)]
pub use self::error::Result;

/// Parse an entire OWL document from the given string.
#[inline]
pub fn parse(src: &str) -> Result<(Ontology, PrefixMapping)> {
    let pair = OwlFunctionalParser::parse(Rule::OntologyDocument, src)?.next().unwrap();
    FromPair::from_pair(pair, &Build::new(), &PrefixMapping::default())
}
