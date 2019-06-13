use std::collections::BTreeSet;

use horned_owl::model::*;

use curie::Curie;
use curie::PrefixMapping;

use pest::iterators::Pair;
use pest::iterators::Pairs;

use super::error::Error;
use super::parser::Rule;

pub trait FromPair: Sized {
    /// Create a new instance from a `Pair`.
    fn from_pair(pair: Pair<Rule>, build: &Build, prefixes: &PrefixMapping) -> Result<Self, Error>;
}

impl FromPair for Annotation {
    fn from_pair(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self, Error> {
        debug_assert!(pair.as_rule() == Rule::Annotation);

        let mut inner = pair.into_inner();
        let _annotations: BTreeSet<Annotation> = inner
            .next()
            .unwrap()
            .into_inner()
            .map(|pair| Self::from_pair(pair, b, p))
            .collect::<Result<BTreeSet<Annotation>, Error>>()?;

        Ok(Annotation {
            annotation_property: FromPair::from_pair(inner.next().unwrap(), b, p)?,
            annotation_value: FromPair::from_pair(inner.next().unwrap(), b, p)?,
        })
    }
}

impl FromPair for AnnotationProperty {
    fn from_pair(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self, Error> {
        debug_assert!(pair.as_rule() == Rule::AnnotationProperty);
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::IRI => IRI::from_pair(inner, b, p).map(AnnotationProperty),
            _ => unreachable!(),
        }
    }
}

impl FromPair for AnnotationValue {
    fn from_pair(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self, Error> {
        debug_assert!(pair.as_rule() == Rule::AnnotationValue);
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::IRI => IRI::from_pair(inner, b, p).map(AnnotationValue::IRI),
            Rule::Literal => Literal::from_pair(inner, b, p).map(AnnotationValue::Literal),
            Rule::AnonymousIndividual => unimplemented!("AnonymousIndividual"),
            _ => unreachable!(),
        }
    }
}

impl FromPair for Import {
    fn from_pair(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self, Error> {
        debug_assert!(pair.as_rule() == Rule::Import);
        IRI::from_pair(pair.into_inner().next().unwrap(), b, p).map(Import)
    }
}

impl FromPair for IRI {
    fn from_pair(pair: Pair<Rule>, build: &Build, prefixes: &PrefixMapping) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::IRI => {
                let inner = pair.into_inner().next().unwrap();
                Self::from_pair(inner, build, prefixes)
            }
            Rule::AbbreviatedIRI => {
                let mut pname = pair.into_inner().next().unwrap().into_inner();
                let prefix = pname.next().unwrap().into_inner().next();
                let local = pname.next().unwrap();
                let curie = Curie::new(prefix.map(|p| p.as_str()), local.as_str());
                match prefixes.expand_curie(&curie) {
                    Ok(iri) => Ok(build.iri(iri)),
                    Err(e) => Err(e.into()),
                }
            }
            Rule::FullIRI => {
                let iri = pair.into_inner().next().unwrap();
                Ok(build.iri(iri.as_str()))
            }
            _ => panic!("wrong rule"),
        }
    }
}

impl FromPair for Literal {
    fn from_pair(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::Literal => {
                Self::from_pair(pair.into_inner().next().unwrap(), b, p)
            }
            Rule::TypedLiteral => {
                let mut inner = pair.into_inner();
                let lit = String::from_pair(inner.next().unwrap(), b, p)?;
                let dty = IRI::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Literal {
                    literal: Some(lit),
                    datatype_iri: Some(dty),
                    lang: None,
                })
            }
            Rule::StringLiteralWithLanguage => {
                let mut inner = pair.into_inner();
                let lit = String::from_pair(inner.next().unwrap(), b, p)?;
                let lang = inner.next().unwrap().as_str()[1..].trim().to_string();
                Ok(Literal {
                    literal: Some(lit),
                    datatype_iri: None,
                    lang: Some(lang),
                })
            }
            Rule::StringLiteralNoLanguage => {
                let mut inner = pair.into_inner();
                let lit = String::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Literal {
                    literal: Some(lit),
                    datatype_iri: None,
                    lang: None,
                })
            }
            _ => unreachable!(),
        }
    }
}

impl FromPair for Ontology {
    fn from_pair(pair: Pair<Rule>, build: &Build, prefixes: &PrefixMapping) -> Result<Self, Error> {
        debug_assert!(pair.as_rule() == Rule::Ontology);
        let mut pairs = pair.into_inner();
        let mut pair = pairs.next().unwrap();

        let mut ontology = Ontology::new();

        // Parse ontology IRI and Version IRI if any
        if pair.as_rule() == Rule::OntologyIRI {
            let inner = pair.into_inner().next().unwrap();
            ontology.id.iri = Some(IRI::from_pair(inner, &build, &prefixes)?);
            pair = pairs.next().unwrap();

            if pair.as_rule() == Rule::VersionIRI {
                let inner = pair.into_inner().next().unwrap();
                ontology.id.viri = Some(IRI::from_pair(inner, &build, &prefixes)?);
                pair = pairs.next().unwrap();
            }
        }

        // Process imports
        for p in pair.into_inner() {
            ontology.insert(Import::from_pair(p, build, prefixes)?);
        }

        // Process ontology annotations
        for p in pairs.next().unwrap().into_inner() {
            ontology.insert(OntologyAnnotation::from_pair(p, build, prefixes)?);
        }

        // Process axioms
        for axiom_pair in pairs.next().unwrap().into_inner() {

        }

        Ok(ontology)
    }
}

impl FromPair for OntologyAnnotation {
    fn from_pair(pair: Pair<Rule>, build: &Build, prefixes: &PrefixMapping) -> Result<Self, Error> {
        debug_assert!(pair.as_rule() == Rule::OntologyDocument);
        Annotation::from_pair(pair, build, prefixes).map(OntologyAnnotation)
    }
}

impl FromPair for (Ontology, PrefixMapping) {
    fn from_pair(pair: Pair<Rule>, build: &Build, prefixes: &PrefixMapping) -> Result<Self, Error> {

        debug_assert!(pair.as_rule() == Rule::OntologyDocument);
        let mut pairs = pair.into_inner();

        // Build the prefix mapping and use it to build the ontology
        let mut prefixes = PrefixMapping::default();
        let mut inner = pairs.next().unwrap();
        while inner.as_rule() == Rule::PrefixDeclaration {

            let mut decl = inner.into_inner();
            let mut pname = decl.next().unwrap().into_inner();
            let iri = decl.next().unwrap().into_inner().next().unwrap();

            if let Some(prefix) = pname.next().unwrap().into_inner().next() {
                prefixes.add_prefix(prefix.as_str(), iri.as_str())?;
            } else {
                prefixes.set_default(iri.as_str());
            }

            inner = pairs.next().unwrap();
        }

        Ontology::from_pair(inner, build, &prefixes).map(|ont| (ont, prefixes))
    }
}

impl FromPair for String {
    fn from_pair(pair: Pair<Rule>, build: &Build, prefixes: &PrefixMapping) -> Result<Self, Error> {
        debug_assert!(pair.as_rule() == Rule::QuotedString);
        let l = pair.as_str().len();
        let s = &pair.as_str()[1..l-1];
        Ok(s.replace(r"\\", r"\").replace(r#"\""#, r#"""#))
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use super::*;
    use crate::parser::OwlFunctionalParser;

    macro_rules! assert_parse_into {
        ($ty:ty, $rule:path, $build:ident, $prefixes:ident, $doc:expr, $expected:expr) => {
            let doc = $doc.trim();
            match OwlFunctionalParser::parse($rule, doc) {
                Ok(mut pairs) => {
                    let res = <$ty as FromPair>::from_pair(pairs.next().unwrap(), &$build, &$prefixes);
                    assert_eq!(res.unwrap(), $expected);
                }
                Err(e) => panic!(
                    "parsing using {:?}:\n{}\nfailed with: {}",
                    $rule,
                    doc.trim(),
                    e
                ),
            }
        };
    }

    #[test]
    fn import() {
        let build = Build::default();
        let mut prefixes = PrefixMapping::default();

        assert_parse_into!(
            Import, Rule::Import, build, prefixes,
            "Import(<http://example.com/path#ref>)",
            Import(build.iri("http://example.com/path#ref"))
        );
    }

    #[test]
    fn iri() {
        let build = Build::default();
        let mut prefixes = PrefixMapping::default();
        prefixes.add_prefix("ex", "http://example.com/path#").unwrap();

        assert_parse_into!(
            IRI, Rule::IRI, build, prefixes,
            "<http://example.com/path#ref>",
            build.iri("http://example.com/path#ref")
        );

        assert_parse_into!(
            IRI, Rule::IRI, build, prefixes,
            "ex:ref",
            build.iri("http://example.com/path#ref")
        );
    }

    #[test]
    fn ontology_document() {
        let build = Build::default();
        let prefixes = PrefixMapping::default();
        let txt = "Prefix(ex:=<http://example.com/>) Prefix(:=<http://default.com/>) Ontology()";

        let mut expected = PrefixMapping::default();
        expected.set_default("http://default.com/");
        expected.add_prefix("ex", "http://example.com/").unwrap();

        let pair = OwlFunctionalParser::parse(Rule::OntologyDocument, txt)
            .unwrap()
            .next()
            .unwrap();

        let doc: (Ontology, PrefixMapping) = FromPair::from_pair(pair, &build, &prefixes).unwrap();
        assert_eq!(
            doc.1.mappings().collect::<HashSet<_>>(),
            expected.mappings().collect::<HashSet<_>>()
        );
    }

}
