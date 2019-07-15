use curie::PrefixMapping;
use horned_owl::model::Build;

use crate::error::Result;
use crate::from_pair::FromPair;
use crate::parser::OwlFunctionalParser;

/// A trait for OWL elements that can be deserialized from OWL strings.
///
/// This i
pub trait FromFunctional: Sized + FromPair {
    /// Deserialize a string containing an OWL element in functional syntax.
    fn from_ofn(s: &str, build: &Build, prefixes: &PrefixMapping) -> Result<Self>;

    #[inline]
    fn from_ofn_with_build(s: &str, build: &Build) -> Result<Self> {
        Self::from_ofn(s, build, &PrefixMapping::default())
    }

    #[inline]
    fn from_ofn_with_prefixes(s: &str, prefixes: &PrefixMapping) -> Result<Self> {
        Self::from_ofn(s, &Build::default(), prefixes)
    }

    #[inline]
    fn from_ofn_str(s: &str) -> Result<Self> {
        Self::from_ofn(s, &Build::default(), &PrefixMapping::default())
    }
}


impl<T: Sized + FromPair> FromFunctional for T {
    fn from_ofn(s: &str, build: &Build, prefixes: &PrefixMapping) -> Result<Self> {
        for rule in T::RULES {
            if let Ok(mut pairs) = OwlFunctionalParser::parse(*rule, s) {
                return T::from_pair(pairs.next().unwrap(), build, prefixes);
            }
        }

        panic!()
    }
}
