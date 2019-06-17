use std::collections::BTreeSet;

use horned_owl::model::*;
use horned_owl::vocab::WithIRI;
use enum_meta::Meta;
use curie::Curie;
use curie::PrefixMapping;
use pest::iterators::Pair;
use pest::iterators::Pairs;

use super::error::Error;
use super::parser::Rule;


// ---------------------------------------------------------------------------

pub trait FromPair: Sized {
    /// Create a new instance from a `Pair`.
    fn from_pair(pair: Pair<Rule>, build: &Build, prefixes: &PrefixMapping) -> Result<Self, Error>;
}


// ---------------------------------------------------------------------------

macro_rules! impl_wrapper {
    ($ty:ident, $rule:path) => {
        impl FromPair for $ty {
            fn from_pair(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self, Error> {
                println!("{:?}", pair);
                debug_assert!(pair.as_rule() == $rule);
                FromPair::from_pair(pair.into_inner().next().unwrap(), b, p).map($ty)
            }
        }
    }
}

impl_wrapper!(Class, Rule::Class);
impl_wrapper!(Import, Rule::Import);
impl_wrapper!(Datatype, Rule::DataAllValuesFrom);
impl_wrapper!(ObjectProperty, Rule::ObjectProperty);
impl_wrapper!(DataProperty, Rule::DataProperty);
impl_wrapper!(AnnotationProperty, Rule::AnnotationProperty);
impl_wrapper!(NamedIndividual, Rule::NamedIndividual);

impl_wrapper!(DeclareClass, Rule::ClassDeclaration);
impl_wrapper!(DeclareDatatype, Rule::DatatypeDeclaration);
impl_wrapper!(DeclareObjectProperty, Rule::ObjectPropertyDeclaration);
impl_wrapper!(DeclareDataProperty, Rule::DataPropertyDeclaration);
impl_wrapper!(DeclareAnnotationProperty, Rule::AnnotationPropertyDeclaration);
impl_wrapper!(DeclareNamedIndividual, Rule::NamedIndividualDeclaration);


// ---------------------------------------------------------------------------

impl FromPair for AnnotatedAxiom {
    fn from_pair(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::Axiom => Self::from_pair(pair.into_inner().next().unwrap(), b, p),
            Rule::ClassAxiom => Self::from_pair(pair.into_inner().next().unwrap(), b, p),

            Rule::Declaration => {
                let mut inner = pair.into_inner();

                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let decl = inner.next().unwrap().into_inner().next().unwrap();
                let axiom: Axiom = match decl.as_rule() {
                    Rule::ClassDeclaration => DeclareClass::from_pair(decl, b, p)?.into(),
                    Rule::DatatypeDeclaration => DeclareDatatype::from_pair(decl, b, p)?.into(),
                    Rule::ObjectPropertyDeclaration => DeclareObjectProperty::from_pair(decl, b, p)?.into(),
                    Rule::DataPropertyDeclaration => DeclareDataProperty::from_pair(decl, b, p)?.into(),
                    Rule::AnnotationPropertyDeclaration => DeclareAnnotationProperty::from_pair(decl, b, p)?.into(),
                    Rule::NamedIndividualDeclaration => DeclareNamedIndividual::from_pair(decl, b, p)?.into(),
                    _ => unreachable!(),
                };

                Ok(Self::new(axiom, annotations))
            }

            Rule::SubClassOf => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let subcls = ClassExpression::from_pair(inner.next().unwrap(), b, p)?;
                let supercls = ClassExpression::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(SubClassOf::new(supercls, subcls), annotations))
            }

            Rule::EquivalentClasses => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ce: Result<_, Error> = inner.map(|pair| FromPair::from_pair(pair, b, p)).collect();
                Ok(Self::new(EquivalentClasses(ce?), annotations))
            }

            Rule::DisjointClasses => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ce: Result<_, Error> = inner.map(|pair| FromPair::from_pair(pair, b, p)).collect();
                Ok(Self::new(DisjointClasses(ce?), annotations))
            }

            Rule::DisjointUnion => {
                unimplemented!()
            }

            Rule::SubObjectPropertyOf => {
                let mut inner = pair.into_inner();
                // let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                unimplemented!()
            }

            _ => unimplemented!(),
        }
    }
}

impl FromPair for Annotation {
    fn from_pair(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self, Error> {
        debug_assert!(pair.as_rule() == Rule::Annotation);

        let mut inner = pair.into_inner();
        let _annotations: BTreeSet<Annotation> = FromPair::from_pair(
            inner.next().unwrap(), b, p
        )?;

        Ok(Annotation {
            annotation_property: FromPair::from_pair(inner.next().unwrap(), b, p)?,
            annotation_value: FromPair::from_pair(inner.next().unwrap(), b, p)?,
        })
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

impl FromPair for BTreeSet<Annotation> {
    fn from_pair(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self, Error> {
        debug_assert!(pair.as_rule() == Rule::AxiomAnnotations);
        pair
            .into_inner()
            .map(|pair| Annotation::from_pair(pair, b, p))
            .collect()
    }
}

impl FromPair for ClassExpression {
    fn from_pair(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self, Error> {
        debug_assert!(pair.as_rule() == Rule::ClassExpression);

        let mut inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::Class => {
                Class::from_pair(inner, b, p).map(ClassExpression::Class)
            }
            Rule::ObjectIntersectionOf => {
                let o = inner.into_inner().map(|pair| Self::from_pair(pair, b, p)).collect::<Result<_, Error>>()?;
                Ok(ClassExpression::ObjectIntersectionOf { o })
            }
            Rule::ObjectUnionOf => {
                let o = inner.into_inner().map(|pair| Self::from_pair(pair, b, p)).collect::<Result<_, Error>>()?;
                Ok(ClassExpression::ObjectUnionOf { o })
            }
            Rule::ObjectComplementOf => {
                let ce = Self::from_pair(inner.into_inner().next().unwrap(), b, p).map(Box::new)?;
                Ok(ClassExpression::ObjectComplementOf { ce })
            }
            Rule::ObjectOneOf => {
                unimplemented!()
            }
            Rule::ObjectSomeValuesFrom => {
                unimplemented!()
            }
            Rule::ObjectAllValuesFrom => {
                unimplemented!()
            }
            Rule::ObjectHasValue => {
                unimplemented!()
            }
            Rule::ObjectHasSelf => {
                unimplemented!()
            }
            Rule::ObjectMinCardinality => {
                unimplemented!()
            }
            Rule::ObjectMaxCardinality => {
                unimplemented!()
            }
            Rule::ObjectExactCardinality => {
                unimplemented!()
            }
            Rule::DataSomeValuesFrom => {
                unimplemented!()
            }
            Rule::DataAllValuesFrom => {
                unimplemented!()
            }
            Rule::DataHasValue => {
                unimplemented!()
            }
            Rule::DataMinCardinality => {
                unimplemented!()
            }
            Rule::DataMaxCardinality => {
                unimplemented!()
            }
            Rule::DataExactCardinality => {
                unimplemented!()
            }
            _ => unreachable!("invalid rule in ClassExpression::from_pair"),
        }
    }
}

impl FromPair for DataRange {
    fn from_pair(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self, Error> {
        debug_assert!(pair.as_rule() == Rule::DataRange);
        let mut inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::Datatype => {
                Datatype::from_pair(inner, b, p).map(DataRange::Datatype)
            }
            Rule::DataIntersectionOf => {
                inner
                    .into_inner()
                    .map(|pair| Self::from_pair(pair, b, p))
                    .collect::<Result<_, _>>()
                    .map(DataRange::DataIntersectionOf)
            }
            Rule::DataUnionOf => {
                inner
                    .into_inner()
                    .map(|pair| Self::from_pair(pair, b, p))
                    .collect::<Result<_, _>>()
                    .map(DataRange::DataUnionOf)
            }
            Rule::DataComplementOf => {
                Self::from_pair(inner.into_inner().next().unwrap(), b, p)
                    .map(Box::new)
                    .map(DataRange::DataComplementOf)
            }
            Rule::DataOneOf => {
                inner
                    .into_inner()
                    .map(|pair| Literal::from_pair(pair, b, p))
                    .collect::<Result<_, _>>()
                    .map(DataRange::DataOneOf)
            }
            Rule::DatatypeRestriction => {
                let mut pairs = inner.into_inner();
                Ok(DataRange::DatatypeRestriction(
                    Datatype::from_pair(pairs.next().unwrap(), b, p)?,
                    pairs
                        .map(|pair| FacetRestriction::from_pair(pair, b, p))
                        .collect::<Result<_, _>>()?
                ))
            }
            _ => unreachable!("unexpected rule in DataRange::from_pair"),
        }
    }
}

impl FromPair for Facet {
    fn from_pair(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self, Error> {
        debug_assert!(pair.as_rule() == Rule::ConstrainingFacet);
        let iri = IRI::from_pair(pair.into_inner().next().unwrap(), b, p)?;
        Facet::all()
            .into_iter()
            .find(|facet| &iri.to_string() == facet.iri_s())
            .ok_or( Error::InvalidFacet(iri.to_string()) )
    }
}

impl FromPair for FacetRestriction {
    fn from_pair(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self, Error> {
        debug_assert!(pair.as_rule() == Rule::FacetRestriction);
        let mut inner = pair.into_inner();
        let f = Facet::from_pair(inner.next().unwrap(), b, p)?;
        let l = Literal::from_pair(inner.next().unwrap(), b, p)?;
        Ok(FacetRestriction { f, l })
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
            _ => unreachable!("invalid rule in IRI::from_pair"),
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
            _ => unreachable!("invalid rule in Literal::from_pair"),
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
        for pair in pairs.next().unwrap().into_inner() {
            ontology.insert(OntologyAnnotation::from_pair(pair, build, prefixes)?);
        }

        // Process axioms
        for pair in pairs.next().unwrap().into_inner() {
            ontology.insert(AnnotatedAxiom::from_pair(pair, build, prefixes)?);
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


// ---------------------------------------------------------------------------

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
    fn declare_class() {
        let build = Build::default();
        let mut prefixes = PrefixMapping::default();
        prefixes.add_prefix("owl", "http://www.w3.org/2002/07/owl#").unwrap();

        assert_parse_into!(
            DeclareClass, Rule::ClassDeclaration, build, prefixes,
            "Class( owl:Thing )",
            DeclareClass(build.class("http://www.w3.org/2002/07/owl#Thing"))
        );

        assert_parse_into!(
            AnnotatedAxiom, Rule::Declaration, build, prefixes,
            "Declaration(Class(owl:Thing))",
            AnnotatedAxiom::from(DeclareClass(build.class("http://www.w3.org/2002/07/owl#Thing")))
        );
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
