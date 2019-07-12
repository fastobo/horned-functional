#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate failure;

extern crate curie;
extern crate horned_owl;
extern crate pest;

pub mod error;
pub mod from_pair;
pub mod parser;

use curie::PrefixMapping;
use horned_owl::model::Ontology;
use horned_owl::model::Build;

use self::parser::OwlFunctionalParser;
use self::parser::Rule;
use self::from_pair::FromPair;
use self::error::Error;

pub fn parse(src: &str) -> Result<(Ontology, PrefixMapping), Error> {
    let mut pair = OwlFunctionalParser::parse(Rule::OntologyDocument, src)?.next().unwrap();
    FromPair::from_pair(pair, &Build::new(), &PrefixMapping::default())
}
