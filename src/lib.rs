#![doc = include_str!("../README.md")]

#[macro_use]
extern crate thiserror;
#[macro_use]
extern crate pest_derive;

extern crate curie;
extern crate horned_owl;
#[cfg(feature = "memmap")]
extern crate memmap;
extern crate pest;

mod as_ofn;
mod error;
mod from_ofn;
mod from_pair;
mod parser;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Write;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use curie::PrefixMapping;
use horned_owl::model::Build;
use horned_owl::model::ForIRI;
use horned_owl::model::Ontology;
use horned_owl::model::IRI;
use horned_owl::ontology::axiom_mapped::AxiomMappedOntology;

pub use self::as_ofn::AsFunctional;
pub use self::as_ofn::Functional;
pub use self::error::Error;
pub use self::error::Result;
pub use self::from_ofn::FromFunctional;

/// A context to pass around while parsing and writing OWL functional documents.
#[derive(Debug)]
pub struct Context<'a, A: ForIRI> {
    build: Option<&'a Build<A>>,
    prefixes: Option<&'a PrefixMapping>,
}

impl<'a, A: ForIRI> Default for Context<'a, A> {
    fn default() -> Self {
        Self {
            build: None,
            prefixes: None,
        }
    }
}

impl<'a, A: ForIRI> Context<'a, A> {
    /// Create a new context with the given IRI builder and prefix mapping.
    pub fn new<B, P>(build: B, prefixes: P) -> Self
    where
        B: Into<Option<&'a Build<A>>>,
        P: Into<Option<&'a PrefixMapping>>,
    {
        Self {
            build: build.into(),
            prefixes: prefixes.into(),
        }
    }

    /// Obtain an IRI for the given string, using the internal builder if any.
    pub fn iri<S>(&self, s: S) -> IRI<A>
    where
        S: Borrow<str>,
    {
        match self.build {
            Some(b) => b.iri(s),
            None => Build::new().iri(s),
        }
    }
}

impl<'a, A: ForIRI> From<&'a Build<A>> for Context<'a, A> {
    fn from(build: &'a Build<A>) -> Self {
        Self {
            build: Some(build),
            prefixes: None,
        }
    }
}

impl<'a, A: ForIRI> From<&'a PrefixMapping> for Context<'a, A> {
    fn from(prefixes: &'a PrefixMapping) -> Self {
        Self {
            build: None,
            prefixes: Some(prefixes),
        }
    }
}

/// Parse an entire OWL document from a string.
#[inline]
pub fn from_str<A, O, S>(src: S) -> Result<(O, PrefixMapping)>
where
    A: ForIRI,
    O: Ontology<A> + FromFunctional<A>,
    S: AsRef<str>,
{
    FromFunctional::from_ofn(src.as_ref())
}

/// Parse an entire OWL document from a `Read` implementor.
#[inline]
pub fn from_reader<A, O, R>(mut r: R) -> Result<(O, PrefixMapping)>
where
    A: ForIRI,
    O: Ontology<A> + FromFunctional<A>,
    R: Read,
{
    let mut s = String::new();
    r.read_to_string(&mut s)?;
    from_str(s)
}

/// Parse an entire OWL document from a file on the local filesystem.
#[inline]
pub fn from_file<A, O, P>(path: P) -> Result<(O, PrefixMapping)>
where
    A: ForIRI,
    O: Ontology<A> + FromFunctional<A>,
    P: AsRef<Path>,
{
    let f = File::open(path)?; // .and_then(from_reader)
    #[cfg(not(feature = "memmap"))]
    return from_reader(f);

    #[cfg(feature = "memmap")]
    unsafe {
        let map = memmap::Mmap::map(&f)?;
        match std::str::from_utf8(&map) {
            Ok(text) => from_str(text),
            Err(error) => Err(Error::IO(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                error,
            ))),
        }
    }
}

// /// Render an entire OWL document to a string.
// #[inline]
// pub fn to_string<'a, P>(ontology: &AxiomMappedOntology, prefixes: P) -> String
// where
//     P: Into<Option<&'a PrefixMapping>>,
// {
//     let mut dest = String::new();
//     // write the prefixes
//     let p = prefixes.into();
//     if let Some(pm) = p {
//         write!(dest, "{}", pm.as_ofn()).expect("infallible");
//     }
//     // write the ontology
//     let ctx = Context::new(None, p);
//     write!(dest, "{}", ontology.as_ofn_ctx(&ctx)).expect("infallible");
//     // return the final string
//     dest
// }
