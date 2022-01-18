#![cfg_attr(feature = "_doc", feature(doc_cfg, external_doc))]
#![cfg_attr(feature = "_doc", doc(include = "../README.md"))]

#[macro_use]
extern crate thiserror;
#[macro_use]
extern crate pest_derive;

extern crate curie;
extern crate horned_owl;
extern crate pest;

mod as_ofn;
mod error;
mod from_ofn;
mod from_pair;
mod parser;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use curie::PrefixMapping;
use horned_owl::model::Build;
use horned_owl::ontology::set::SetOntology;

pub use self::as_ofn::AsFunctional;
pub use self::as_ofn::Functional;
pub use self::error::Error;
pub use self::error::Result;
pub use self::from_ofn::FromFunctional;

/// A context to pass around while parsing and writing OWL functional documents.
#[derive(Debug, Default)]
pub struct Context<'a> {
    build: Option<&'a Build>,
    prefixes: Option<&'a PrefixMapping>,
}

impl<'a> From<&'a Build> for Context<'a> {
    fn from(build: &'a Build) -> Context<'a> {
        Self {
            build: Some(build),
            prefixes: None,
        }
    }
}

impl<'a> From<&'a PrefixMapping> for Context<'a> {
    fn from(prefixes: &'a PrefixMapping) -> Context<'a> {
        Self {
            build: None,
            prefixes: Some(prefixes),
        }
    }
}

/// Parse an entire OWL document from a string.
#[inline]
pub fn from_str<S: AsRef<str>>(src: S) -> Result<(SetOntology, PrefixMapping)> {
    FromFunctional::from_ofn_str(src.as_ref())
}

/// Parse an entire OWL document from a `Read` implementor.
#[inline]
pub fn from_reader<R: Read>(mut r: R) -> Result<(SetOntology, PrefixMapping)> {
    let mut s = String::new();
    r.read_to_string(&mut s)?;
    from_str(s)
}

/// Parse an entire OWL document from a file on the local filesystem..
#[inline]
pub fn from_file<P: AsRef<Path>>(path: P) -> Result<(SetOntology, PrefixMapping)> {
    File::open(path).map_err(Error::from).and_then(from_reader)
}
