use std::collections::BTreeSet;
use std::str::FromStr;

use horned_owl::model::*;
use horned_owl::vocab::OWL;
use horned_owl::vocab::OWL2Datatype;
use horned_owl::vocab::WithIRI;
use enum_meta::Meta;
use curie::Curie;
use curie::PrefixMapping;
use pest::iterators::Pair;

use crate::error::Error;
use crate::error::Result;
use crate::parser::Rule;


// ---------------------------------------------------------------------------

/// A trait for OWL elements that can be obtained from OWL Functional tokens.
///
/// `Pair<Rule>` values can be obtained from the `OwlFunctionalParser` struct
/// after parsing a
pub trait FromPair: Sized {
    /// The valid production rules for the implementor.
    const RULES: &'static [Rule];

    /// Create a new instance from a `Pair`.
    fn from_pair(pair: Pair<Rule>, build: &Build, prefixes: &PrefixMapping) -> Result<Self> {
        if cfg!(debug_assertions) && !Self::RULES.contains(&pair.as_rule()) {
            return Err(Error::from(
                pest::error::Error::new_from_span(
                    pest::error::ErrorVariant::ParsingError {
                        positives: vec![pair.as_rule()],
                        negatives: Self::RULES.to_vec(),
                    },
                    pair.as_span(),
                )
            ));
        }
        Self::from_pair_unchecked(pair, build, prefixes)
    }

    fn from_pair_unchecked(pair: Pair<Rule>, build: &Build, prefixes: &PrefixMapping) -> Result<Self>;
}


// ---------------------------------------------------------------------------

macro_rules! impl_wrapper {
    ($ty:ident, $rule:path) => {
        impl FromPair for $ty {
            const RULES: &'static [Rule] = &[$rule];
            fn from_pair_unchecked(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self> {
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
    const RULES: &'static [Rule] = &[
        Rule::Axiom,
        Rule::ClassAxiom,
        Rule::ObjectPropertyAxiom,
        Rule::DataPropertyAxiom,
        Rule::Assertion,
        Rule::AnnotationAxiom,
        Rule::Declaration,
        Rule::SubClassOf,
        Rule::EquivalentClasses,
        Rule::DisjointClasses,
        Rule::DisjointUnion,
        Rule::SubObjectPropertyOf,
        Rule::EquivalentObjectProperties,
        Rule::DisjointObjectProperties,
        Rule::ObjectPropertyDomain,
        Rule::ObjectPropertyRange,
        Rule::InverseObjectProperties,
        Rule::InverseObjectProperties,
        Rule::InverseFunctionalObjectProperty,
        Rule::ReflexiveObjectProperty,
        Rule::IrreflexiveObjectProperty,
        Rule::SymmetricObjectProperty,
        Rule::AsymmetricObjectProperty,
        Rule::TransitiveObjectProperty,
        Rule::SubDataPropertyOf,
        Rule::EquivalentDataProperties,
        Rule::DisjointDataProperties,
        Rule::DataPropertyDomain,
        Rule::DataPropertyRange,
        Rule::FunctionalDataProperty,
        Rule::DatatypeDefinition,
        Rule::HasKey,
        Rule::SameIndividual,
        Rule::DifferentIndividuals,
        Rule::ClassAssertion,
        Rule::ObjectPropertyAssertion,
        Rule::NegativeObjectPropertyAssertion,
        Rule::DataPropertyAssertion,
        Rule::NegativeDataPropertyAssertion,
        Rule::AnnotationAssertion,
        Rule::SubAnnotationPropertyOf,
        Rule::AnnotationPropertyDomain,
        Rule::AnnotationPropertyRange,
    ];
    fn from_pair_unchecked(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self> {
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
                    rule => unreachable!("unexpected rule in AnnotatedAxiom::Declaration: {:?}", rule),
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
                let ce: Result<_> = inner.map(|pair| FromPair::from_pair(pair, b, p)).collect();
                Ok(Self::new(EquivalentClasses(ce?), annotations))
            }
            Rule::DisjointClasses => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ce: Result<_> = inner.map(|pair| FromPair::from_pair(pair, b, p)).collect();
                Ok(Self::new(DisjointClasses(ce?), annotations))
            }
            Rule::DisjointUnion => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let cls = Class::from_pair(inner.next().unwrap(), b, p)?;
                let ce: Result<_> = inner.map(|pair| FromPair::from_pair(pair, b, p)).collect();
                Ok(Self::new(DisjointUnion(cls, ce?), annotations))
            }

            // ObjectPropertyAxiom
            Rule::SubObjectPropertyOf => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let sub = SubObjectPropertyExpression::from_pair(inner.next().unwrap(), b, p)?;
                let sup = ObjectPropertyExpression::from_pair(
                    inner.next().unwrap().into_inner().next().unwrap(), b, p)?;
                Ok(Self::new(SubObjectPropertyOf { sup, sub }, annotations))
            }
            Rule::EquivalentObjectProperties => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let op: Result<Vec<ObjectPropertyExpression>> = inner.map(|pair| FromPair::from_pair(pair, b, p)).collect();
                Ok(Self::new(EquivalentObjectProperties(op?), annotations))
            }
            Rule::DisjointObjectProperties => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let op: Result<Vec<ObjectPropertyExpression>> = inner.map(|pair| FromPair::from_pair(pair, b, p)).collect();
                Ok(Self::new(DisjointObjectProperties(op?), annotations))
            }
            Rule::ObjectPropertyDomain => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ope = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ce = ClassExpression::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(ObjectPropertyDomain::new(ope, ce), annotations))
            }
            Rule::ObjectPropertyRange => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ope = ObjectPropertyExpression::from_pair(inner.next().unwrap(), b, p)?;
                let ce = ClassExpression::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(ObjectPropertyRange::new(ope, ce), annotations))
            }
            Rule::InverseObjectProperties => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let r1 = ObjectProperty::from_pair(inner.next().unwrap(), b, p)?;
                let r2 = ObjectProperty::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(InverseObjectProperties(r1, r2), annotations))
            }
            Rule::FunctionalObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let r = ObjectPropertyExpression::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(FunctionalObjectProperty(r), annotations))
            }
            Rule::InverseFunctionalObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let r = ObjectPropertyExpression::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(InverseFunctionalObjectProperty(r), annotations))
            }
            Rule::ReflexiveObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let r = ObjectPropertyExpression::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(ReflexiveObjectProperty(r), annotations))
            }
            Rule::IrreflexiveObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let r = ObjectPropertyExpression::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(IrreflexiveObjectProperty(r), annotations))
            }
            Rule::SymmetricObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let r = ObjectPropertyExpression::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(SymmetricObjectProperty(r), annotations))
            }
            Rule::AsymmetricObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let r = ObjectPropertyExpression::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(AsymmetricObjectProperty(r), annotations))
            }
            Rule::TransitiveObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let r = ObjectProperty::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(TransitiveObjectProperty(r), annotations))
            }

            // DataPropertyAxiom
            Rule::SubDataPropertyOf => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let sub = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let sup = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(SubDataPropertyOf { sub, sup }, annotations))
            }
            Rule::EquivalentDataProperties => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let dp: Result<Vec<DataProperty>> = inner.map(|pair| FromPair::from_pair(pair, b, p)).collect();
                Ok(Self::new(EquivalentDataProperties(dp?), annotations))
            }
            Rule::DisjointDataProperties => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let dp: Result<Vec<DataProperty>> = inner.map(|pair| FromPair::from_pair(pair, b, p)).collect();
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
                Err(Error::Unsupported(
                    "horned-owl does not have proper HasKey definition",
                    "https://github.com/phillord/horned-owl/issues/21",
                ))
            }

            // Assertion
            Rule::SameIndividual => {
                // FIXME: support for anonymous individual
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let individuals: Result<_> = inner
                    .map(|pair| NamedIndividual::from_pair(pair, b, p))
                    .collect();
                Ok(Self::new(SameIndividual(individuals?), annotations))
            }
            Rule::DifferentIndividuals => {
                // FIXME: support for anonymous individual
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let individuals: Result<_> = inner
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
                Ok(Self::new(ObjectPropertyAssertion { ope, from, to }, annotations))
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
                let ap = AnnotationProperty::from_pair(inner.next().unwrap(), b, p)?;
                let subject = IRI::from_pair(inner.next().unwrap().into_inner().next().unwrap(), b, p)?;
                let av = AnnotationValue::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(
                    AnnotationAssertion::new(subject, Annotation { ap, av }),
                    annotations,
                ))
            }
            Rule::SubAnnotationPropertyOf => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let sub = FromPair::from_pair(inner.next().unwrap().into_inner().next().unwrap(), b, p)?;
                let sup = FromPair::from_pair(inner.next().unwrap().into_inner().next().unwrap(), b, p)?;
                Ok(Self::new(SubAnnotationPropertyOf { sub, sup }, annotations))
            }

            Rule::AnnotationPropertyDomain => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ap = AnnotationProperty::from_pair(inner.next().unwrap(), b, p)?;
                let iri = IRI::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(AnnotationPropertyDomain::new(ap, iri), annotations))
            }
            Rule::AnnotationPropertyRange => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), b, p)?;
                let ap = AnnotationProperty::from_pair(inner.next().unwrap(), b, p)?;
                let iri = IRI::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Self::new(AnnotationPropertyRange::new(ap, iri), annotations))
            }
            _ => unreachable!("unexpected rule in AnnotatedAxiom::from_pair"),
        }
    }
}

// ---------------------------------------------------------------------------

impl FromPair for Annotation {
    const RULES: &'static [Rule] = &[Rule::Annotation];
    fn from_pair_unchecked(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self> {
        let mut inner = pair.into_inner();
        let _annotations: BTreeSet<Annotation> = FromPair::from_pair(
            inner.next().unwrap(), b, p
        )?;

        Ok(Annotation {
            ap: FromPair::from_pair(inner.next().unwrap(), b, p)?,
            av: FromPair::from_pair(inner.next().unwrap(), b, p)?,
        })
    }
}

// ---------------------------------------------------------------------------

impl FromPair for AnnotationValue {
    const RULES: &'static [Rule] = &[Rule::AnnotationValue];
    fn from_pair_unchecked(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::IRI => IRI::from_pair(inner, b, p).map(AnnotationValue::IRI),
            Rule::Literal => Literal::from_pair(inner, b, p).map(AnnotationValue::Literal),
            Rule::AnonymousIndividual => unimplemented!("AnonymousIndividual"),
            _ => unreachable!(),
        }
    }
}

// ---------------------------------------------------------------------------

impl FromPair for BTreeSet<Annotation> {
    const RULES: &'static [Rule] = &[Rule::AnnotationAnnotations, Rule::AxiomAnnotations];
    fn from_pair_unchecked(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self> {
        pair
            .into_inner()
            .map(|pair| Annotation::from_pair(pair, b, p))
            .collect()
    }
}

// ---------------------------------------------------------------------------

macro_rules! impl_ce_data_cardinality {
    ($b:ident, $p:ident, $inner:ident, $dt:ident) => {{
        let mut pair = $inner.into_inner();
        let n = u32::from_pair(pair.next().unwrap(), $b, $p)?;
        let dp = DataProperty::from_pair(pair.next().unwrap(), $b, $p)?;
        let dr = match pair.next() {
            // No data range is equivalent to `rdfs:Literal` as a data range.
            // see https://www.w3.org/TR/owl2-syntax/#Data_Property_Cardinality_Restrictions
            None => $b.datatype(OWL2Datatype::RDFSLiteral.iri_s()).into(),
            Some(pair) => DataRange::from_pair(pair, $b, $p)?,
        };
        Ok(ClassExpression::$dt { n, dp, dr })
    }}
}

macro_rules! impl_ce_obj_cardinality {
    ($b:ident, $p:ident, $inner:ident, $dt:ident) => {{
        let mut pair = $inner.into_inner();
        let n = u32::from_pair(pair.next().unwrap(), $b, $p)?;
        let ope = ObjectPropertyExpression::from_pair(pair.next().unwrap(), $b, $p)?;
        let bce = match pair.next() {
            // Missing class expression is equivalent to `owl:Thing` as class expression.
            // see https://www.w3.org/TR/owl2-syntax/#Object_Property_Cardinality_Restrictions
            None => Box::new(ClassExpression::Class($b.class(OWL::Thing.iri_s()))),
            Some(x) => Self::from_pair(x, $b, $p).map(Box::new)?,
        };
        Ok(ClassExpression::ObjectMinCardinality { n, ope, bce })
    }}
}

impl FromPair for ClassExpression {
    const RULES: &'static [Rule] = &[Rule::ClassExpression];
    fn from_pair_unchecked(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::Class => {
                Class::from_pair(inner, b, p).map(ClassExpression::Class)
            }
            Rule::ObjectIntersectionOf => {
                inner
                    .into_inner()
                    .map(|pair| Self::from_pair(pair, b, p))
                    .collect::<Result<_>>()
                    .map(ClassExpression::ObjectIntersectionOf)
            }
            Rule::ObjectUnionOf => {
                inner
                    .into_inner()
                    .map(|pair| Self::from_pair(pair, b, p))
                    .collect::<Result<_>>()
                    .map(ClassExpression::ObjectUnionOf)
            }
            Rule::ObjectComplementOf => {
                Self::from_pair(inner.into_inner().next().unwrap(), b, p)
                    .map(Box::new)
                    .map(ClassExpression::ObjectComplementOf)
            }
            Rule::ObjectOneOf => {
                inner
                    .into_inner()
                    .map(|pair| NamedIndividual::from_pair(pair, b, p))
                    .collect::<Result<_>>()
                    .map(ClassExpression::ObjectOneOf)
            }
            Rule::ObjectSomeValuesFrom => {
                let mut pairs = inner.into_inner();
                let ope = ObjectPropertyExpression::from_pair(pairs.next().unwrap(), b, p)?;
                let bce = Self::from_pair(pairs.next().unwrap(), b, p).map(Box::new)?;
                Ok(ClassExpression::ObjectSomeValuesFrom { ope, bce })
            }
            Rule::ObjectAllValuesFrom => {
                let mut pairs = inner.into_inner();
                let ope = ObjectPropertyExpression::from_pair(pairs.next().unwrap(), b, p)?;
                let bce = Self::from_pair(pairs.next().unwrap(), b, p).map(Box::new)?;
                Ok(ClassExpression::ObjectAllValuesFrom { ope, bce })
            }
            Rule::ObjectHasValue => {
                let mut pairs = inner.into_inner();
                let ope = ObjectPropertyExpression::from_pair(pairs.next().unwrap(), b, p)?;
                let i = NamedIndividual::from_pair(pairs.next().unwrap(), b, p)?;
                Ok(ClassExpression::ObjectHasValue { ope, i })
            }
            Rule::ObjectHasSelf => {
                let pair = inner.into_inner().next().unwrap();
                let expr = ObjectPropertyExpression::from_pair(pair, b, p)?;
                Ok(ClassExpression::ObjectHasSelf(expr))
            }
            Rule::ObjectMinCardinality => {
                impl_ce_obj_cardinality!(b, p, inner, ObjectMinCardinality)
            }
            Rule::ObjectMaxCardinality => {
                impl_ce_obj_cardinality!(b, p, inner, ObjectMaxCardinality)
            }
            Rule::ObjectExactCardinality => {
                impl_ce_obj_cardinality!(b, p, inner, ObjectExactCardinality)
            }
            Rule::DataSomeValuesFrom => {
                let mut pair = inner.into_inner();
                let dp = DataProperty::from_pair(pair.next().unwrap(), b, p)?;
                let next = pair.next().unwrap();
                if next.as_rule() == Rule::DataProperty {
                    Err(Error::Unsupported(
                        "data property chaining in DataSomeValuesFrom",
                        "https://github.com/phillord/horned-owl/issues/17"
                    ))
                } else {
                    let dr = DataRange::from_pair(next, b, p)?;
                    Ok(ClassExpression::DataSomeValuesFrom { dp, dr })
                }
            }
            Rule::DataAllValuesFrom => {
                let mut pair = inner.into_inner();
                let dp = DataProperty::from_pair(pair.next().unwrap(), b, p)?;
                let next = pair.next().unwrap();
                if next.as_rule() == Rule::DataProperty {
                    Err(Error::Unsupported(
                        "data property chaining in DataAllValuesFrom",
                        "https://github.com/phillord/horned-owl/issues/17"
                    ))
                } else {
                    let dr = DataRange::from_pair(next, b, p)?;
                    Ok(ClassExpression::DataAllValuesFrom { dp, dr })
                }
            }
            Rule::DataHasValue => {
                let mut pair = inner.into_inner();
                let dp = DataProperty::from_pair(pair.next().unwrap(), b, p)?;
                let l = Literal::from_pair(pair.next().unwrap(), b, p)?;
                Ok(ClassExpression::DataHasValue { dp, l })
            }
            Rule::DataMinCardinality => {
                impl_ce_data_cardinality!(b, p, inner, DataMinCardinality)
            }
            Rule::DataMaxCardinality => {
                impl_ce_data_cardinality!(b, p, inner, DataMaxCardinality)
            }
            Rule::DataExactCardinality => {
                impl_ce_data_cardinality!(b, p, inner, DataExactCardinality)
            }
            rule => unreachable!("unexpected rule in ClassExpression::from_pair: {:?}", rule),
        }
    }
}

// ---------------------------------------------------------------------------

impl FromPair for DataRange {
    const RULES: &'static [Rule] = &[Rule::DataRange];
    fn from_pair_unchecked(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::Datatype => {
                Datatype::from_pair(inner, b, p).map(DataRange::Datatype)
            }
            Rule::DataIntersectionOf => {
                inner
                    .into_inner()
                    .map(|pair| Self::from_pair(pair, b, p))
                    .collect::<Result<_>>()
                    .map(DataRange::DataIntersectionOf)
            }
            Rule::DataUnionOf => {
                inner
                    .into_inner()
                    .map(|pair| Self::from_pair(pair, b, p))
                    .collect::<Result<_>>()
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
                    .collect::<Result<_>>()
                    .map(DataRange::DataOneOf)
            }
            Rule::DatatypeRestriction => {
                let mut pairs = inner.into_inner();
                Ok(DataRange::DatatypeRestriction(
                    Datatype::from_pair(pairs.next().unwrap(), b, p)?,
                    pairs
                        .map(|pair| FacetRestriction::from_pair(pair, b, p))
                        .collect::<Result<_>>()?
                ))
            }
            rule => unreachable!("unexpected rule in DataRange::from_pair: {:?}", rule),
        }
    }
}

// ---------------------------------------------------------------------------

impl FromPair for Facet {
    const RULES: &'static [Rule] = &[Rule::ConstrainingFacet];
    fn from_pair_unchecked(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self> {
        let iri = IRI::from_pair(pair.into_inner().next().unwrap(), b, p)?;
        Facet::all()
            .into_iter()
            .find(|facet| &iri.to_string() == facet.iri_s())
            .ok_or_else(|| Error::InvalidFacet(iri.to_string()))
    }
}

// ---------------------------------------------------------------------------

impl FromPair for FacetRestriction {
    const RULES: &'static [Rule] = &[Rule::FacetRestriction];
    fn from_pair_unchecked(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self> {
        let mut inner = pair.into_inner();
        let f = Facet::from_pair(inner.next().unwrap(), b, p)?;
        let l = Literal::from_pair(inner.next().unwrap(), b, p)?;
        Ok(FacetRestriction { f, l })
    }
}

// ---------------------------------------------------------------------------

impl FromPair for IRI {
    const RULES: &'static [Rule] = &[Rule::IRI, Rule::AbbreviatedIRI, Rule::FullIRI];
    fn from_pair_unchecked(pair: Pair<Rule>, build: &Build, prefixes: &PrefixMapping) -> Result<Self> {
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
                prefixes.expand_curie(&curie).map_err(Error::from).map(|s| build.iri(s))
            }
            Rule::FullIRI => {
                let iri = pair.into_inner().next().unwrap();
                Ok(build.iri(iri.as_str()))
            }
            rule => unreachable!("unexpected rule in IRI::from_pair: {:?}", rule),
        }
    }
}

// ---------------------------------------------------------------------------

impl FromPair for NamedIndividual {
    const RULES: &'static [Rule] = &[Rule::Individual, Rule::SourceIndividual, Rule::TargetIndividual, Rule::AnonymousIndividual, Rule::NamedIndividual];
    fn from_pair_unchecked(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self> {
        match pair.as_rule() {
            Rule::Individual => Self::from_pair(pair.into_inner().next().unwrap(), b, p),
            Rule::SourceIndividual => Self::from_pair(pair.into_inner().next().unwrap(), b, p),
            Rule::TargetIndividual => Self::from_pair(pair.into_inner().next().unwrap(), b, p),
            Rule::NamedIndividual => {
                IRI::from_pair(pair.into_inner().next().unwrap(), b, p)
                    .map(NamedIndividual)
            }
            Rule::AnonymousIndividual => Err(Error::Unsupported(
                "anonymous individual",
                "https://github.com/fastobo/horned-functional/issues/1"
            )),
            rule => unreachable!("unexpected rule in NamedIndividual::from_pair: {:?}", rule)
        }
    }
}

// ---------------------------------------------------------------------------

impl FromPair for Literal {
    const RULES: &'static [Rule] = &[Rule::Literal, Rule::TypedLiteral, Rule::StringLiteralWithLanguage, Rule::StringLiteralNoLanguage];
    fn from_pair_unchecked(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self> {
        match pair.as_rule() {
            Rule::Literal => {
                Self::from_pair(pair.into_inner().next().unwrap(), b, p)
            }
            Rule::TypedLiteral => {
                let mut inner = pair.into_inner();
                let lit = String::from_pair(inner.next().unwrap(), b, p)?;
                let dty = Datatype::from_pair(inner.next().unwrap(), b, p)?;
                Ok(Literal {
                    literal: Some(lit),
                    datatype_iri: Some(dty.0),
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
            rule => unreachable!("unexpected rule in Literal::from_pair: {:?}", rule),
        }
    }
}

// ---------------------------------------------------------------------------

impl FromPair for ObjectPropertyExpression {
    const RULES: &'static [Rule] = &[Rule::ObjectPropertyExpression];
    fn from_pair_unchecked(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::ObjectProperty => ObjectProperty::from_pair(inner, b, p)
                .map(ObjectPropertyExpression::ObjectProperty),
            Rule::InverseObjectProperty => ObjectProperty::from_pair(inner.into_inner().next().unwrap(), b, p)
                .map(ObjectPropertyExpression::InverseObjectProperty),
            rule => unreachable!("unexpected rule in ObjectPropertyExpression::from_pair: {:?}", rule),
        }
    }
}

// ---------------------------------------------------------------------------

impl FromPair for Ontology {
    const RULES: &'static [Rule] = &[Rule::Ontology];
    fn from_pair_unchecked(pair: Pair<Rule>, build: &Build, prefixes: &PrefixMapping) -> Result<Self> {
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

// ---------------------------------------------------------------------------

impl FromPair for OntologyAnnotation {
    const RULES: &'static [Rule] = &[Rule::Annotation];
    fn from_pair_unchecked(pair: Pair<Rule>, build: &Build, prefixes: &PrefixMapping) -> Result<Self> {
        Annotation::from_pair(pair, build, prefixes).map(OntologyAnnotation)
    }
}

// ---------------------------------------------------------------------------

impl FromPair for (Ontology, PrefixMapping) {
    const RULES: &'static [Rule] = &[Rule::OntologyDocument];
    fn from_pair_unchecked(pair: Pair<Rule>, build: &Build, _prefixes: &PrefixMapping) -> Result<Self> {
        let mut pairs = pair.into_inner();

        // Build the prefix mapping and use it to build the ontology
        let mut prefixes = PrefixMapping::default();
        let mut inner = pairs.next().unwrap();
        while inner.as_rule() == Rule::PrefixDeclaration {

            let mut decl = inner.into_inner();
            let mut pname = decl.next().unwrap().into_inner();
            let iri = decl.next().unwrap().into_inner().next().unwrap();

            if let Some(prefix) = pname.next().unwrap().into_inner().next() {
                prefixes.add_prefix(prefix.as_str(), iri.as_str())
                    .expect("grammar does not allow invalid prefixes");
            } else {
                prefixes.set_default(iri.as_str());
            }

            inner = pairs.next().unwrap();
        }

        Ontology::from_pair(inner, build, &prefixes).map(|ont| (ont, prefixes))
    }
}

// ---------------------------------------------------------------------------

impl FromPair for String {
    const RULES: &'static [Rule] = &[Rule::QuotedString];
    fn from_pair_unchecked(pair: Pair<Rule>, _build: &Build, _prefixes: &PrefixMapping) -> Result<Self> {
        let l = pair.as_str().len();
        let s = &pair.as_str()[1..l-1];
        Ok(s.replace(r"\\", r"\").replace(r#"\""#, r#"""#))
    }
}

// ---------------------------------------------------------------------------

impl FromPair for SubObjectPropertyExpression {
    const RULES: &'static [Rule] = &[Rule::SubObjectPropertyExpression];
    fn from_pair_unchecked(pair: Pair<Rule>, b: &Build, p: &PrefixMapping) -> Result<Self> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::ObjectPropertyExpression => {
                ObjectPropertyExpression::from_pair(inner, b, p)
                    .map(SubObjectPropertyExpression::ObjectPropertyExpression)
            }
            Rule::PropertyExpressionChain => {
                let mut objs = Vec::new();
                for pair in inner.into_inner() {
                    objs.push(ObjectPropertyExpression::from_pair(pair, b, p)?);
                }
                Ok(SubObjectPropertyExpression::ObjectPropertyChain(objs))
            }
            rule => unreachable!("unexpected rule in SubObjectProperty::from_pair: {:?}", rule),
        }
    }
}

// ---------------------------------------------------------------------------

impl FromPair for u32 {
    const RULES: &'static [Rule] = &[Rule::NonNegativeInteger];
    fn from_pair_unchecked(pair: Pair<Rule>, _b: &Build, _p: &PrefixMapping) -> Result<Self> {
        Ok(Self::from_str(pair.as_str()).expect("cannot fail with the right rule"))
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
        let prefixes = PrefixMapping::default();

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
