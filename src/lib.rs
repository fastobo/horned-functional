#![doc = include_str!("../README.md")]

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

use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Write;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use curie::PrefixMapping;
use horned_owl::model::Build;
use horned_owl::model::Ontology;
use horned_owl::ontology::axiom_mapped::AxiomMappedOntology;
use horned_owl::ontology::set::SetOntology;

pub use self::as_ofn::AsFunctional;
pub use self::as_ofn::Functional;
pub use self::error::Error;
pub use self::error::Result;
pub use self::from_ofn::FromFunctional;

/// A context to pass around while parsing and writing OWL functional documents.
#[derive(Default)]
pub struct Context<'a> {
    build: Option<&'a Build>,
    prefixes: Option<&'a PrefixMapping>,
}

// Before `v0.1.1`, `curie::PrefixMapping` doesn't implement `Debug`.
impl<'a> Debug for Context<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("Context")
            .field("build", &self.build)
            .field(
                "prefixes",
                &match &self.prefixes {
                    None => format!("{}", "None"),
                    Some(p) => {
                        format!("{:?}", p.mappings().collect::<HashMap<_, _>>())
                    }
                },
            )
            .finish()
    }
}

impl<'a> Context<'a> {
    /// Create a new context with the given IRI builder and prefix mapping.
    pub fn new<B, P>(build: B, prefixes: P) -> Self
    where
        B: Into<Option<&'a Build>>,
        P: Into<Option<&'a PrefixMapping>>,
    {
        Self {
            build: build.into(),
            prefixes: prefixes.into(),
        }
    }

    /// Obtain an IRI for the given string, using the internal builder if any.
    pub fn iri<S: Into<String>>(&self, s: S) -> horned_owl::model::IRI
    where
        S: Into<String>,
    {
        match self.build {
            Some(b) => b.iri(s),
            None => Build::default().iri(s),
        }
    }
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
pub fn from_str<O, S>(src: S) -> Result<(O, PrefixMapping)>
where
    O: Ontology + FromFunctional,
    S: AsRef<str>,
{
    FromFunctional::from_ofn(src.as_ref())
}

/// Parse an entire OWL document from a `Read` implementor.
#[inline]
pub fn from_reader<O, R>(mut r: R) -> Result<(O, PrefixMapping)>
where
    O: Ontology + FromFunctional,
    R: Read,
{
    let mut s = String::new();
    r.read_to_string(&mut s)?;
    from_str(s)
}

/// Parse an entire OWL document from a file on the local filesystem.
#[inline]
pub fn from_file<O, P>(path: P) -> Result<(O, PrefixMapping)>
where
    O: Ontology + FromFunctional,
    P: AsRef<Path>,
{
    File::open(path).map_err(Error::from).and_then(from_reader)
}

/// Render an entire OWL document to a string.
#[inline]
pub fn to_string<'a, P>(ontology: &AxiomMappedOntology, prefixes: P) -> String
where
    P: Into<Option<&'a PrefixMapping>>,
{
    let mut dest = String::new();
    // write the prefixes
    let p = prefixes.into();
    if let Some(pm) = p {
        write!(dest, "{}", pm.as_ofn()).expect("infallible");
    }
    // write the ontology
    let ctx = Context::new(None, p);
    write!(dest, "{}", ontology.as_ofn_ctx(&ctx)).expect("infallible");
    // return the final string
    dest
}
