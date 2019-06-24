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
impl_wrapper!(Datatype, Rule::Datatype);
impl_wrapper!(ObjectProperty, Rule::ObjectProperty);
impl_wrapper!(DataProperty, Rule::DataProperty);
impl_wrapper!(AnnotationProperty, Rule::AnnotationProperty);

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
            // Rule::AnnotatedAxiom
            Rule::Axiom => Self::from_pair(pair.into_inner().next().unwrap(), b, p),
            Rule::ClassAxiom => Self::from_pair(pair.into_inner().next().unwrap(), b, p),
            Rule::ObjectPropertyAxiom => Self::from_pair(pair.into_inner().next().unwrap(), b, p),
            Rule::DataPropertyAxiom => Self::from_pair(pair.into_inner().next().unwrap(), b, p),
            Rule::Assertion => Self::from_pair(pair.into_inner().next().unwrap(), b, p),
            Rule::AnnotationAxiom => Self::from_pair(pair.into_inner().next().unwrap(), b, p),

            // Declaration
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

            // ClassAxiom
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
                // FIXME: missing definition in `horned-owl`
                unimplemented!()
            }

            // ObjectPropertyAxiom
            Rule::SubObjectPropertyOf => {
                unimplemented!()
            }
            Rule::EquivalentObjectProperties => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let op: Result<Vec<ObjectPropertyExpression>, Error> = inner.map(|pair| FromPair::from_pair(pair, b, p)).collect();
                Ok(Self::new(EquivalentObjectProperties(op?), annotations))
            }
            Rule::DisjointObjectProperties => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let op: Result<Vec<ObjectPropertyExpression>, Error> = inner.map(|pair| FromPair::from_pair(pair, b, p)).collect();
                Ok(Self::new(DisjointObjectProperties(op?), annotations))
            }
            Rule::ObjectPropertyDomain => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ope = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ce = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(ObjectPropertyDomain::new(ope, ce), annotations))
            }
            Rule::ObjectPropertyRange => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ope = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ce = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(ObjectPropertyRange::new(ope, ce), annotations))
            }
            Rule::InverseObjectProperties => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let r1 = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let r2 = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(InverseObjectProperties(r1, r2), annotations))
            }
            Rule::FunctionalObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let r = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(FunctionalObjectProperty(r), annotations))
            }
            Rule::InverseFunctionalObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let r = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(InverseFunctionalObjectProperty(r), annotations))
            }
            Rule::ReflexiveObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let r = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(ReflexiveObjectProperty(r), annotations))
            }
            Rule::IrreflexiveObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let r = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(IrreflexiveObjectProperty(r), annotations))
            }
            Rule::SymmetricObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let r = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(SymmetricObjectProperty(r), annotations))
            }
            Rule::AsymmetricObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let r = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(AsymmetricObjectProperty(r), annotations))
            }
            Rule::TransitiveObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let r = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(TransitiveObjectProperty(r), annotations))
            }

            // DataPropertyAxiom
            Rule::SubDataPropertyOf => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let sub_property = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let super_property = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(
                    SubDataPropertyOf { sub_property, super_property },
                    annotations,
                ))
            }
            Rule::EquivalentDataProperties => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let dp: Result<Vec<DataProperty>, Error> = inner.map(|pair| FromPair::from_pair(pair, b, p)).collect();
                Ok(Self::new(EquivalentDataProperties(dp?), annotations))
            }
            Rule::DisjointDataProperties => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let dp: Result<Vec<DataProperty>, Error> = inner.map(|pair| FromPair::from_pair(pair, b, p)).collect();
                Ok(Self::new(DisjointDataProperties(dp?), annotations))
            }
            Rule::DataPropertyDomain => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let dp = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ce = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(ObjectPropertyDomain::new(dp, ce), annotations))
            }
            Rule::DataPropertyRange => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let dp = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ce = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(ObjectPropertyRange::new(dp, ce), annotations))
            }
            Rule::FunctionalDataProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let dp = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(FunctionalDataProperty(dp), annotations))
            }
            Rule::DatatypeDefinition => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let k = Datatype::from_pair(inner.next().unwrap(), b, p)?;
                let r = DataRange::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(DatatypeDefinition::new(k, r), annotations))
            }

            // HasKey
            Rule::HasKey => {
                unimplemented!()
            }

            // Assertion
            Rule::SameIndividual => {
                // FIXME: support for anonymous individual
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let individuals: Result<_, _> = inner
                    .map(|pair| NamedIndividual::from_pair(pair, b, p))
                    .collect();
                Ok(Self::new(SameIndividual(individuals?), annotations))
            }
            Rule::DifferentIndividuals => {
                // FIXME: support for anonymous individual
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let individuals: Result<_, _> = inner
                    .map(|pair| NamedIndividual::from_pair(pair, b, p))
                    .collect();
                Ok(Self::new(DifferentIndividuals(individuals?), annotations))
            }
            Rule::ClassAssertion => {
                // FIXME: support for anonymous individual
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ce = ClassExpression::from_pair(inner.next().unwrap(), b, p)?;
                let i = NamedIndividual::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(ClassAssertion::new(ce, i), annotations))
            }
            Rule::ObjectPropertyAssertion => {
                // FIXME: support for anonymous individual
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ope = ObjectPropertyExpression::from_pair(inner.next().unwrap(), b, p)?;
                let from = NamedIndividual::from_pair(inner.next().unwrap(), b, p)?;
                let to = NamedIndividual::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(ObjectPropertyAssertion::new(ope, from, to), annotations))
            }
            Rule::NegativeObjectPropertyAssertion => {
                // FIXME: support for anonymous individual
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ope = ObjectPropertyExpression::from_pair(inner.next().unwrap(), b, p)?;
                let from = NamedIndividual::from_pair(inner.next().unwrap(), b, p)?;
                let to = NamedIndividual::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(NegativeObjectPropertyAssertion::new(ope, from, to), annotations))
            }
            Rule::DataPropertyAssertion => {
                // FIXME: support for anonymous individual
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ope = DataProperty::from_pair(inner.next().unwrap(), b, p)?;
                let from = NamedIndividual::from_pair(inner.next().unwrap(), b, p)?;
                let to = Literal::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(DataPropertyAssertion::new(ope, from, to), annotations))
            }
            Rule::NegativeDataPropertyAssertion => {
                // FIXME: support for anonymous individual
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ope = DataProperty::from_pair(inner.next().unwrap(), b, p)?;
                let from = NamedIndividual::from_pair(inner.next().unwrap(), b, p)?;
                let to = Literal::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(NegativeDataPropertyAssertion::new(ope, from, to), annotations))
            }

            // AnnotationAxiom
            Rule::AnnotationAssertion => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let property = AnnotationProperty::from_pair(inner.next().unwrap(), b, p)?;
                let subject = IRI::from_pair(inner.next().unwrap().into_inner().next().unwrap(), b, p)?;
                let value = AnnotationValue::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(
                    AnnotationAssertion::new(subject, Annotation {
                        annotation_property: property,
                        annotation_value: value,
                    }),
                    annotations,
                ))
            }
            Rule::SubAnnotationPropertyOf => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let sub_property = FromPair::from_pair(inner.next().unwrap().into_inner().next().unwrap(), b, p)?;
                let super_property = FromPair::from_pair(inner.next().unwrap().into_inner().next().unwrap(), b, p)?;
                Ok(Self::new(SubAnnotationPropertyOf { sub_property, super_property }, annotations))
            }
            Rule::AnnotationPropertyDomain => unimplemented!(),
            Rule::AnnotationPropertyRange => unimplemented!(),

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

impl FromPair for NamedIndividual {
    fn from_pair(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::Individual => Self::from_pair(pair.into_inner().next().unwrap(), b, p),
            Rule::SourceIndividual => Self::from_pair(pair.into_inner().next().unwrap(), b, p),
            Rule::TargetIndividual => Self::from_pair(pair.into_inner().next().unwrap(), b, p),
            Rule::AnonymousIndividual => unimplemented!("AnonymousIndividual are unsupported"),
            Rule::NamedIndividual => {
                IRI::from_pair(pair.into_inner().next().unwrap(), b, p)
                    .map(NamedIndividual)
            }
            _ => unreachable!("invalid rule in NamedIndividual::from_pair")
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

impl FromPair for ObjectPropertyExpression {
    fn from_pair(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self, Error> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::ObjectProperty => ObjectProperty::from_pair(inner, b, p)
                .map(ObjectPropertyExpression::ObjectProperty),
            Rule::InverseObjectProperty => ObjectProperty::from_pair(inner.into_inner().next().unwrap(), b, p)
                .map(ObjectPropertyExpression::InverseObjectProperty),
            _ => unreachable!("invalid rule in ObjectPropertyExpression.from_pair"),
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
