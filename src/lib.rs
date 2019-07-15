#[macro_use]
extern crate err_derive;
#[macro_use]
extern crate pest_derive;

extern crate curie;
extern crate horned_owl;
extern crate pest;

mod from_pair;
mod from_fn_str;
mod error;
mod parser;

use curie::PrefixMapping;
use horned_owl::model::Ontology;

pub use self::error::Error;
pub use self::error::Result;
pub use self::from_fn_str::FromFunctional;

/// Parse an entire OWL document from the given string.
#[inline]
pub fn parse(src: &str) -> Result<(Ontology, PrefixMapping)> {
    FromFunctional::from_ofn_str(src)
}
