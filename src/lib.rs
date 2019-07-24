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

use std::fs::File;
use std::path::Path;
use std::io::Read;

use curie::PrefixMapping;
use horned_owl::model::Ontology;

pub use self::error::Error;
pub use self::error::Result;
pub use self::from_fn_str::FromFunctional;

/// Parse an entire OWL document from a string.
#[inline]
pub fn from_str<S: AsRef<str>>(src: S) -> Result<(Ontology, PrefixMapping)> {
    FromFunctional::from_ofn_str(src.as_ref())
}

/// Parse an entire OWL document from a `Read` implementor.
#[inline]
pub fn from_reader<R: Read + 'static>(mut r: R) -> Result<(Ontology, PrefixMapping)> {
    let mut s = String::new();
    r.read_to_string(&mut s)?;
    from_str(s)
}

/// Parse an entire OWL document from a file on the local filesystem..
#[inline]
pub fn from_file<P: AsRef<Path>>(path: P) -> Result<(Ontology, PrefixMapping)> {
    //let f = File::open(path).map_err(Error::from)?;
    from_reader(File::open(path)?)
}
