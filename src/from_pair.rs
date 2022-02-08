use std::collections::BTreeSet;
use std::str::FromStr;

use curie::Curie;
use curie::PrefixMapping;
use enum_meta::Meta;
use horned_owl::model::*;
use horned_owl::ontology::axiom_mapped::AxiomMappedOntology;
use horned_owl::ontology::set::SetOntology;
use horned_owl::vocab::OWL2Datatype;
use horned_owl::vocab::WithIRI;
use horned_owl::vocab::OWL;
use pest::iterators::Pair;

use crate::error::Error;
use crate::error::Result;
use crate::parser::Rule;
use crate::Context;

// ---------------------------------------------------------------------------

/// A trait for OWL elements that can be obtained from OWL Functional tokens.
///
/// `Pair<Rule>` values can be obtained from the `OwlFunctionalParser` struct
/// after parsing a document.
pub trait FromPair: Sized {
    /// The valid production rule for the implementor.
    const RULE: Rule;

    /// Create a new instance from a `Pair`.
    #[inline]
    fn from_pair(pair: Pair<Rule>, context: &Context<'_>) -> Result<Self> {
        if cfg!(debug_assertions) && &pair.as_rule() != &Self::RULE {
            return Err(Error::from(pest::error::Error::new_from_span(
                pest::error::ErrorVariant::ParsingError {
                    positives: vec![pair.as_rule()],
                    negatives: vec![Self::RULE],
                },
                pair.as_span(),
            )));
        }
        Self::from_pair_unchecked(pair, context)
    }

    /// Create a new instance from a `Pair` without checking the PEG rule.
    fn from_pair_unchecked(pair: Pair<Rule>, context: &Context<'_>) -> Result<Self>;
}

// ---------------------------------------------------------------------------

macro_rules! impl_wrapper {
    ($ty:ident, $rule:path) => {
        impl FromPair for $ty {
            const RULE: Rule = $rule;
            fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context<'_>) -> Result<Self> {
                FromPair::from_pair(pair.into_inner().next().unwrap(), ctx).map($ty)
            }
        }
    };
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
impl_wrapper!(
    DeclareAnnotationProperty,
    Rule::AnnotationPropertyDeclaration
);
impl_wrapper!(DeclareNamedIndividual, Rule::NamedIndividualDeclaration);

// ---------------------------------------------------------------------------

impl FromPair for AnnotatedAxiom {
    const RULE: Rule = Rule::Axiom;
    fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context<'_>) -> Result<Self> {
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            // Declaration
            Rule::Declaration => {
                let mut inner = pair.into_inner();

                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let decl = inner.next().unwrap().into_inner().next().unwrap();
                let axiom: Axiom = match decl.as_rule() {
                    Rule::ClassDeclaration => DeclareClass::from_pair(decl, ctx)?.into(),
                    Rule::DatatypeDeclaration => DeclareDatatype::from_pair(decl, ctx)?.into(),
                    Rule::ObjectPropertyDeclaration => {
                        DeclareObjectProperty::from_pair(decl, ctx)?.into()
                    }
                    Rule::DataPropertyDeclaration => {
                        DeclareDataProperty::from_pair(decl, ctx)?.into()
                    }
                    Rule::AnnotationPropertyDeclaration => {
                        DeclareAnnotationProperty::from_pair(decl, ctx)?.into()
                    }
                    Rule::NamedIndividualDeclaration => {
                        DeclareNamedIndividual::from_pair(decl, ctx)?.into()
                    }
                    rule => {
                        unreachable!("unexpected rule in AnnotatedAxiom::Declaration: {:?}", rule)
                    }
                };

                Ok(Self::new(axiom, annotations))
            }

            // ClassAxiom
            Rule::SubClassOf => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let subcls = ClassExpression::from_pair(inner.next().unwrap(), ctx)?;
                let supercls = ClassExpression::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(SubClassOf::new(supercls, subcls), annotations))
            }
            Rule::EquivalentClasses => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let ce: Result<_> = inner.map(|pair| FromPair::from_pair(pair, ctx)).collect();
                Ok(Self::new(EquivalentClasses(ce?), annotations))
            }
            Rule::DisjointClasses => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let ce: Result<_> = inner.map(|pair| FromPair::from_pair(pair, ctx)).collect();
                Ok(Self::new(DisjointClasses(ce?), annotations))
            }
            Rule::DisjointUnion => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let cls = Class::from_pair(inner.next().unwrap(), ctx)?;
                let ce: Result<_> = inner.map(|pair| FromPair::from_pair(pair, ctx)).collect();
                Ok(Self::new(DisjointUnion(cls, ce?), annotations))
            }

            // ObjectPropertyAxiom
            Rule::SubObjectPropertyOf => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let sub = SubObjectPropertyExpression::from_pair(inner.next().unwrap(), ctx)?;
                let sup = ObjectPropertyExpression::from_pair(
                    inner.next().unwrap().into_inner().next().unwrap(),
                    ctx,
                )?;
                Ok(Self::new(SubObjectPropertyOf { sup, sub }, annotations))
            }
            Rule::EquivalentObjectProperties => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let op: Result<Vec<ObjectPropertyExpression>> =
                    inner.map(|pair| FromPair::from_pair(pair, ctx)).collect();
                Ok(Self::new(EquivalentObjectProperties(op?), annotations))
            }
            Rule::DisjointObjectProperties => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let op: Result<Vec<ObjectPropertyExpression>> =
                    inner.map(|pair| FromPair::from_pair(pair, ctx)).collect();
                Ok(Self::new(DisjointObjectProperties(op?), annotations))
            }
            Rule::ObjectPropertyDomain => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let ope = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let ce = ClassExpression::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(ObjectPropertyDomain::new(ope, ce), annotations))
            }
            Rule::ObjectPropertyRange => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let ope = ObjectPropertyExpression::from_pair(inner.next().unwrap(), ctx)?;
                let ce = ClassExpression::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(ObjectPropertyRange::new(ope, ce), annotations))
            }
            Rule::InverseObjectProperties => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let r1 = ObjectProperty::from_pair(inner.next().unwrap(), ctx)?;
                let r2 = ObjectProperty::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(InverseObjectProperties(r1, r2), annotations))
            }
            Rule::FunctionalObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let r = ObjectPropertyExpression::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(FunctionalObjectProperty(r), annotations))
            }
            Rule::InverseFunctionalObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let r = ObjectPropertyExpression::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(InverseFunctionalObjectProperty(r), annotations))
            }
            Rule::ReflexiveObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let r = ObjectPropertyExpression::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(ReflexiveObjectProperty(r), annotations))
            }
            Rule::IrreflexiveObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let r = ObjectPropertyExpression::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(IrreflexiveObjectProperty(r), annotations))
            }
            Rule::SymmetricObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let r = ObjectPropertyExpression::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(SymmetricObjectProperty(r), annotations))
            }
            Rule::AsymmetricObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let r = ObjectPropertyExpression::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(AsymmetricObjectProperty(r), annotations))
            }
            Rule::TransitiveObjectProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let r = ObjectProperty::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(TransitiveObjectProperty(r.into()), annotations))
            }

            // DataPropertyAxiom
            Rule::SubDataPropertyOf => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let sub = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let sup = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(SubDataPropertyOf { sub, sup }, annotations))
            }
            Rule::EquivalentDataProperties => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let dp: Result<Vec<DataProperty>> =
                    inner.map(|pair| FromPair::from_pair(pair, ctx)).collect();
                Ok(Self::new(EquivalentDataProperties(dp?), annotations))
            }
            Rule::DisjointDataProperties => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let dp: Result<Vec<DataProperty>> =
                    inner.map(|pair| FromPair::from_pair(pair, ctx)).collect();
                Ok(Self::new(DisjointDataProperties(dp?), annotations))
            }
            Rule::DataPropertyDomain => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let dp = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let ce = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(ObjectPropertyDomain::new(dp, ce), annotations))
            }
            Rule::DataPropertyRange => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let dp = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let ce = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(ObjectPropertyRange::new(dp, ce), annotations))
            }
            Rule::FunctionalDataProperty => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let dp = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(FunctionalDataProperty(dp), annotations))
            }
            Rule::DatatypeDefinition => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let k = Datatype::from_pair(inner.next().unwrap(), ctx)?;
                let r = DataRange::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(DatatypeDefinition::new(k, r), annotations))
            }

            // HasKey
            Rule::HasKey => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let ce = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let vpe: Result<Vec<PropertyExpression>> = inner
                    .map(|pair| match pair.as_rule() {
                        Rule::ObjectPropertyExpression => FromPair::from_pair(pair, ctx)
                            .map(PropertyExpression::ObjectPropertyExpression),
                        Rule::DataProperty => {
                            FromPair::from_pair(pair, ctx).map(PropertyExpression::DataProperty)
                        }
                        _ => unreachable!(),
                    })
                    .collect();
                Ok(Self::new(HasKey::new(ce, vpe?), annotations))
            }

            // Assertion
            Rule::SameIndividual => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let individuals: Result<Vec<Individual>> =
                    inner.map(|pair| Individual::from_pair(pair, ctx)).collect();
                Ok(Self::new(SameIndividual(individuals?), annotations))
            }
            Rule::DifferentIndividuals => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let individuals: Result<Vec<Individual>> =
                    inner.map(|pair| Individual::from_pair(pair, ctx)).collect();
                Ok(Self::new(DifferentIndividuals(individuals?), annotations))
            }
            Rule::ClassAssertion => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let ce = ClassExpression::from_pair(inner.next().unwrap(), ctx)?;
                let i = Individual::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(ClassAssertion::new(ce, i), annotations))
            }
            Rule::ObjectPropertyAssertion => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let ope = ObjectPropertyExpression::from_pair(inner.next().unwrap(), ctx)?;
                let from = Individual::from_pair(inner.next().unwrap(), ctx)?;
                let to = Individual::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(
                    ObjectPropertyAssertion { ope, from, to },
                    annotations,
                ))
            }
            Rule::NegativeObjectPropertyAssertion => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let ope = ObjectPropertyExpression::from_pair(inner.next().unwrap(), ctx)?;
                let from = Individual::from_pair(inner.next().unwrap(), ctx)?.into();
                let to = Individual::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(
                    NegativeObjectPropertyAssertion::new(ope, from, to),
                    annotations,
                ))
            }
            Rule::DataPropertyAssertion => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let ope = DataProperty::from_pair(inner.next().unwrap(), ctx)?;
                let from = Individual::from_pair(inner.next().unwrap(), ctx)?;
                let to = Literal::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(
                    DataPropertyAssertion::new(ope, from, to),
                    annotations,
                ))
            }
            Rule::NegativeDataPropertyAssertion => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let ope = DataProperty::from_pair(inner.next().unwrap(), ctx)?;
                let from = Individual::from_pair(inner.next().unwrap(), ctx)?;
                let to = Literal::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(
                    NegativeDataPropertyAssertion::new(ope, from, to),
                    annotations,
                ))
            }

            // AnnotationAxiom
            Rule::AnnotationAssertion => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let ap = AnnotationProperty::from_pair(inner.next().unwrap(), ctx)?;
                let subject = {
                    let inner2 = inner.next().unwrap().into_inner().next().unwrap();
                    match inner2.as_rule() {
                        // FIXME: likely to change after discussion in
                        //        https://github.com/phillord/horned-owl/pull/32
                        Rule::IRI => IRI::from_pair(inner2, ctx).map(AnnotationSubject::from)?,
                        // .map(Individual::Named)?,
                        Rule::AnonymousIndividual => AnonymousIndividual::from_pair(inner2, ctx)
                            .map(AnnotationSubject::from)?,
                        rule => {
                            unreachable!("unexpected rule in Individual::from_pair: {:?}", rule)
                        }
                    }
                };
                let av = AnnotationValue::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(
                    AnnotationAssertion::new(subject, Annotation { ap, av }),
                    annotations,
                ))
            }
            Rule::SubAnnotationPropertyOf => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let sub =
                    FromPair::from_pair(inner.next().unwrap().into_inner().next().unwrap(), ctx)?;
                let sup =
                    FromPair::from_pair(inner.next().unwrap().into_inner().next().unwrap(), ctx)?;
                Ok(Self::new(SubAnnotationPropertyOf { sub, sup }, annotations))
            }
            Rule::AnnotationPropertyDomain => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let ap = AnnotationProperty::from_pair(inner.next().unwrap(), ctx)?;
                let iri = IRI::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(
                    AnnotationPropertyDomain::new(ap, iri),
                    annotations,
                ))
            }
            Rule::AnnotationPropertyRange => {
                let mut inner = pair.into_inner();
                let annotations = FromPair::from_pair(inner.next().unwrap(), ctx)?;
                let ap = AnnotationProperty::from_pair(inner.next().unwrap(), ctx)?;
                let iri = IRI::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Self::new(
                    AnnotationPropertyRange::new(ap, iri),
                    annotations,
                ))
            }

            _ => unreachable!("unexpected rule in AnnotatedAxiom::from_pair"),
        }
    }
}

// ---------------------------------------------------------------------------

impl FromPair for Annotation {
    const RULE: Rule = Rule::Annotation;
    fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context<'_>) -> Result<Self> {
        let mut inner = pair.into_inner();
        let _annotations: BTreeSet<Annotation> = FromPair::from_pair(inner.next().unwrap(), ctx)?;

        Ok(Annotation {
            ap: FromPair::from_pair(inner.next().unwrap(), ctx)?,
            av: FromPair::from_pair(inner.next().unwrap(), ctx)?,
        })
    }
}

// ---------------------------------------------------------------------------

impl FromPair for AnnotationValue {
    const RULE: Rule = Rule::AnnotationValue;
    fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context<'_>) -> Result<Self> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::IRI => IRI::from_pair(inner, ctx).map(AnnotationValue::IRI),
            Rule::Literal => Literal::from_pair(inner, ctx).map(AnnotationValue::Literal),
            // FIXME: when horned-owl is updated, replace with
            //        AnonymousIndividual::from_pair(inner, ctx).map(AnnotationValue::Anonymous)
            Rule::AnonymousIndividual => unimplemented!(
                "horned-owl does not support AnonymousIndividual as annotation values"
            ),
            _ => unreachable!(),
        }
    }
}

// ---------------------------------------------------------------------------

impl FromPair for AnonymousIndividual {
    const RULE: Rule = Rule::AnonymousIndividual;
    fn from_pair_unchecked(pair: Pair<Rule>, _ctx: &Context<'_>) -> Result<Self> {
        // FIXME: use a builder here when possible to reuse the string.
        let nodeid = pair.into_inner().next().unwrap();
        let inner = nodeid.into_inner().next().unwrap();
        Ok(AnonymousIndividual(From::from(inner.as_str())))
    }
}

// ---------------------------------------------------------------------------

impl FromPair for Axiom {
    const RULE: Rule = Rule::Axiom;
    fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context<'_>) -> Result<Self> {
        AnnotatedAxiom::from_pair_unchecked(pair, ctx).map(|aa| aa.axiom)
    }
}

// ---------------------------------------------------------------------------

impl FromPair for BTreeSet<Annotation> {
    const RULE: Rule = Rule::Annotations;
    fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context<'_>) -> Result<Self> {
        pair.into_inner()
            .map(|pair| Annotation::from_pair(pair, ctx))
            .collect()
    }
}

// ---------------------------------------------------------------------------

macro_rules! impl_ce_data_cardinality {
    ($ctx:ident, $inner:ident, $dt:ident) => {{
        let mut pair = $inner.into_inner();
        let n = u32::from_pair(pair.next().unwrap(), $ctx)?;
        let dp = DataProperty::from_pair(pair.next().unwrap(), $ctx)?;
        let dr = match pair.next() {
            Some(pair) => DataRange::from_pair(pair, $ctx)?,
            // No data range is equivalent to `rdfs:Literal` as a data range.
            // see https://www.w3.org/TR/owl2-syntax/#Data_Property_Cardinality_Restrictions
            None => Datatype($ctx.iri(OWL2Datatype::RDFSLiteral.iri_s())).into(),
        };
        Ok(ClassExpression::$dt { n, dp, dr })
    }};
}

macro_rules! impl_ce_obj_cardinality {
    ($ctx:ident, $inner:ident, $dt:ident) => {{
        let mut pair = $inner.into_inner();
        let n = u32::from_pair(pair.next().unwrap(), $ctx)?;
        let ope = ObjectPropertyExpression::from_pair(pair.next().unwrap(), $ctx)?;
        let bce = match pair.next() {
            Some(x) => Self::from_pair(x, $ctx).map(Box::new)?,
            // Missing class expression is equivalent to `owl:Thing` as class expression.
            // see https://www.w3.org/TR/owl2-syntax/#Object_Property_Cardinality_Restrictions
            None => Box::new(ClassExpression::Class(Class($ctx.iri(OWL::Thing.iri_s())))),
        };
        Ok(ClassExpression::ObjectMinCardinality { n, ope, bce })
    }};
}

impl FromPair for ClassExpression {
    const RULE: Rule = Rule::ClassExpression;
    fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context<'_>) -> Result<Self> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::Class => Class::from_pair(inner, ctx).map(ClassExpression::Class),
            Rule::ObjectIntersectionOf => inner
                .into_inner()
                .map(|pair| Self::from_pair(pair, ctx))
                .collect::<Result<_>>()
                .map(ClassExpression::ObjectIntersectionOf),
            Rule::ObjectUnionOf => inner
                .into_inner()
                .map(|pair| Self::from_pair(pair, ctx))
                .collect::<Result<_>>()
                .map(ClassExpression::ObjectUnionOf),
            Rule::ObjectComplementOf => Self::from_pair(inner.into_inner().next().unwrap(), ctx)
                .map(Box::new)
                .map(ClassExpression::ObjectComplementOf),
            Rule::ObjectOneOf => inner
                .into_inner()
                .map(|pair| Individual::from_pair(pair, ctx))
                .collect::<Result<Vec<Individual>>>()
                .map(ClassExpression::ObjectOneOf),
            Rule::ObjectSomeValuesFrom => {
                let mut pairs = inner.into_inner();
                let ope = ObjectPropertyExpression::from_pair(pairs.next().unwrap(), ctx)?;
                let bce = Self::from_pair(pairs.next().unwrap(), ctx).map(Box::new)?;
                Ok(ClassExpression::ObjectSomeValuesFrom { ope, bce })
            }
            Rule::ObjectAllValuesFrom => {
                let mut pairs = inner.into_inner();
                let ope = ObjectPropertyExpression::from_pair(pairs.next().unwrap(), ctx)?;
                let bce = Self::from_pair(pairs.next().unwrap(), ctx).map(Box::new)?;
                Ok(ClassExpression::ObjectAllValuesFrom { ope, bce })
            }
            Rule::ObjectHasValue => {
                let mut pairs = inner.into_inner();
                let ope = ObjectPropertyExpression::from_pair(pairs.next().unwrap(), ctx)?;
                let i = Individual::from_pair(pairs.next().unwrap(), ctx)?;
                Ok(ClassExpression::ObjectHasValue { ope, i })
            }
            Rule::ObjectHasSelf => {
                let pair = inner.into_inner().next().unwrap();
                let expr = ObjectPropertyExpression::from_pair(pair, ctx)?;
                Ok(ClassExpression::ObjectHasSelf(expr))
            }
            Rule::ObjectMinCardinality => {
                impl_ce_obj_cardinality!(ctx, inner, ObjectMinCardinality)
            }
            Rule::ObjectMaxCardinality => {
                impl_ce_obj_cardinality!(ctx, inner, ObjectMaxCardinality)
            }
            Rule::ObjectExactCardinality => {
                impl_ce_obj_cardinality!(ctx, inner, ObjectExactCardinality)
            }
            Rule::DataSomeValuesFrom => {
                let mut pair = inner.into_inner();
                let dp = DataProperty::from_pair(pair.next().unwrap(), ctx)?;
                let next = pair.next().unwrap();
                if next.as_rule() == Rule::DataProperty {
                    Err(Error::Unsupported(
                        "data property chaining in DataSomeValuesFrom",
                        "https://github.com/phillord/horned-owl/issues/17",
                    ))
                } else {
                    let dr = DataRange::from_pair(next, ctx)?;
                    Ok(ClassExpression::DataSomeValuesFrom { dp, dr })
                }
            }
            Rule::DataAllValuesFrom => {
                let mut pair = inner.into_inner();
                let dp = DataProperty::from_pair(pair.next().unwrap(), ctx)?;
                let next = pair.next().unwrap();
                if next.as_rule() == Rule::DataProperty {
                    Err(Error::Unsupported(
                        "data property chaining in DataAllValuesFrom",
                        "https://github.com/phillord/horned-owl/issues/17",
                    ))
                } else {
                    let dr = DataRange::from_pair(next, ctx)?;
                    Ok(ClassExpression::DataAllValuesFrom { dp, dr })
                }
            }
            Rule::DataHasValue => {
                let mut pair = inner.into_inner();
                let dp = DataProperty::from_pair(pair.next().unwrap(), ctx)?;
                let l = Literal::from_pair(pair.next().unwrap(), ctx)?;
                Ok(ClassExpression::DataHasValue { dp, l })
            }
            Rule::DataMinCardinality => {
                impl_ce_data_cardinality!(ctx, inner, DataMinCardinality)
            }
            Rule::DataMaxCardinality => {
                impl_ce_data_cardinality!(ctx, inner, DataMaxCardinality)
            }
            Rule::DataExactCardinality => {
                impl_ce_data_cardinality!(ctx, inner, DataExactCardinality)
            }
            rule => unreachable!("unexpected rule in ClassExpression::from_pair: {:?}", rule),
        }
    }
}

// ---------------------------------------------------------------------------

impl FromPair for DataRange {
    const RULE: Rule = Rule::DataRange;
    fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context<'_>) -> Result<Self> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::Datatype => Datatype::from_pair(inner, ctx).map(DataRange::Datatype),
            Rule::DataIntersectionOf => inner
                .into_inner()
                .map(|pair| Self::from_pair(pair, ctx))
                .collect::<Result<_>>()
                .map(DataRange::DataIntersectionOf),
            Rule::DataUnionOf => inner
                .into_inner()
                .map(|pair| Self::from_pair(pair, ctx))
                .collect::<Result<_>>()
                .map(DataRange::DataUnionOf),
            Rule::DataComplementOf => Self::from_pair(inner.into_inner().next().unwrap(), ctx)
                .map(Box::new)
                .map(DataRange::DataComplementOf),
            Rule::DataOneOf => inner
                .into_inner()
                .map(|pair| Literal::from_pair(pair, ctx))
                .collect::<Result<_>>()
                .map(DataRange::DataOneOf),
            Rule::DatatypeRestriction => {
                let mut pairs = inner.into_inner();
                Ok(DataRange::DatatypeRestriction(
                    Datatype::from_pair(pairs.next().unwrap(), ctx)?,
                    pairs
                        .map(|pair| FacetRestriction::from_pair(pair, ctx))
                        .collect::<Result<_>>()?,
                ))
            }
            rule => unreachable!("unexpected rule in DataRange::from_pair: {:?}", rule),
        }
    }
}

// ---------------------------------------------------------------------------

impl FromPair for Facet {
    const RULE: Rule = Rule::ConstrainingFacet;
    fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context<'_>) -> Result<Self> {
        let iri = IRI::from_pair(pair.into_inner().next().unwrap(), ctx)?;
        Facet::all()
            .into_iter()
            .find(|facet| &iri.to_string() == facet.iri_s())
            .ok_or_else(|| Error::InvalidFacet(iri.to_string()))
    }
}

// ---------------------------------------------------------------------------

impl FromPair for FacetRestriction {
    const RULE: Rule = Rule::FacetRestriction;
    fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context<'_>) -> Result<Self> {
        let mut inner = pair.into_inner();
        let f = Facet::from_pair(inner.next().unwrap(), ctx)?;
        let l = Literal::from_pair(inner.next().unwrap(), ctx)?;
        Ok(FacetRestriction { f, l })
    }
}

// ---------------------------------------------------------------------------

impl FromPair for Individual {
    const RULE: Rule = Rule::Individual;
    fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context<'_>) -> Result<Self> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::NamedIndividual => NamedIndividual::from_pair(inner, ctx).map(Individual::Named),
            Rule::AnonymousIndividual => {
                AnonymousIndividual::from_pair(inner, ctx).map(Individual::Anonymous)
            }
            rule => unreachable!("unexpected rule in Individual::from_pair: {:?}", rule),
        }
    }
}

// ---------------------------------------------------------------------------

impl FromPair for IRI {
    const RULE: Rule = Rule::IRI;
    fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context<'_>) -> Result<Self> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::AbbreviatedIRI => {
                let mut pname = inner.into_inner().next().unwrap().into_inner();
                let prefix = pname.next().unwrap().into_inner().next();
                let local = pname.next().unwrap();
                let curie = Curie::new(prefix.map(|p| p.as_str()), local.as_str());
                if let Some(prefixes) = ctx.prefixes {
                    prefixes
                        .expand_curie(&curie)
                        .map_err(Error::from)
                        .map(|s| ctx.iri(s))
                } else {
                    Err(Error::from(curie::ExpansionError::Invalid))
                }
            }
            Rule::FullIRI => {
                let iri = inner.into_inner().next().unwrap();
                Ok(ctx.iri(iri.as_str()))
            }
            rule => unreachable!("unexpected rule in IRI::from_pair: {:?}", rule),
        }
    }
}

// ---------------------------------------------------------------------------

impl FromPair for NamedIndividual {
    const RULE: Rule = Rule::NamedIndividual;
    fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context) -> Result<Self> {
        IRI::from_pair(pair.into_inner().next().unwrap(), ctx).map(NamedIndividual)
    }
}

// ---------------------------------------------------------------------------

impl FromPair for Literal {
    const RULE: Rule = Rule::Literal;
    fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context) -> Result<Self> {
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::Literal => Self::from_pair(pair.into_inner().next().unwrap(), ctx),
            Rule::TypedLiteral => {
                let mut inner = pair.into_inner();
                let literal = String::from_pair(inner.next().unwrap(), ctx)?;
                let dty = Datatype::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Literal::Datatype {
                    literal,
                    datatype_iri: dty.0,
                })
            }
            Rule::StringLiteralWithLanguage => {
                let mut inner = pair.into_inner();
                let literal = String::from_pair(inner.next().unwrap(), ctx)?;
                let lang = inner.next().unwrap().as_str()[1..].trim().to_string();
                Ok(Literal::Language { literal, lang })
            }
            Rule::StringLiteralNoLanguage => {
                let mut inner = pair.into_inner();
                let literal = String::from_pair(inner.next().unwrap(), ctx)?;
                Ok(Literal::Simple { literal })
            }
            rule => unreachable!("unexpected rule in Literal::from_pair: {:?}", rule),
        }
    }
}

// ---------------------------------------------------------------------------

impl FromPair for ObjectPropertyExpression {
    const RULE: Rule = Rule::ObjectPropertyExpression;
    fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context<'_>) -> Result<Self> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::ObjectProperty => {
                ObjectProperty::from_pair(inner, ctx).map(ObjectPropertyExpression::ObjectProperty)
            }
            Rule::InverseObjectProperty => {
                ObjectProperty::from_pair(inner.into_inner().next().unwrap(), ctx)
                    .map(ObjectPropertyExpression::InverseObjectProperty)
            }
            rule => unreachable!(
                "unexpected rule in ObjectPropertyExpression::from_pair: {:?}",
                rule
            ),
        }
    }
}

// ---------------------------------------------------------------------------

macro_rules! impl_ontology {
    ($ty:ident) => {
        impl FromPair for $ty {
            const RULE: Rule = Rule::Ontology;
            fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context<'_>) -> Result<Self> {
                debug_assert!(pair.as_rule() == Rule::Ontology);
                let mut pairs = pair.into_inner();
                let mut pair = pairs.next().unwrap();

                let mut ontology = $ty::default();
                let mut ontology_id = ontology.mut_id();

                // Parse ontology IRI and Version IRI if any
                if pair.as_rule() == Rule::OntologyIRI {
                    let inner = pair.into_inner().next().unwrap();
                    ontology_id.iri = Some(IRI::from_pair(inner, ctx)?);
                    pair = pairs.next().unwrap();
                    if pair.as_rule() == Rule::VersionIRI {
                        let inner = pair.into_inner().next().unwrap();
                        ontology_id.viri = Some(IRI::from_pair(inner, ctx)?);
                        pair = pairs.next().unwrap();
                    }
                }

                // Process imports
                for p in pair.into_inner() {
                    ontology.insert(Import::from_pair(p, ctx)?);
                }

                // Process ontology annotations
                for pair in pairs.next().unwrap().into_inner() {
                    ontology.insert(OntologyAnnotation::from_pair(pair, ctx)?);
                }

                // Process axioms
                for pair in pairs.next().unwrap().into_inner() {
                    ontology.insert(AnnotatedAxiom::from_pair(pair, ctx)?);
                }

                Ok(ontology)
            }
        }
    };
}

impl_ontology!(SetOntology);
impl_ontology!(AxiomMappedOntology);

// ---------------------------------------------------------------------------

impl FromPair for OntologyAnnotation {
    const RULE: Rule = Rule::Annotation;
    fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context<'_>) -> Result<Self> {
        Annotation::from_pair(pair, ctx).map(OntologyAnnotation)
    }
}

// ---------------------------------------------------------------------------

impl<O> FromPair for (O, PrefixMapping)
where
    O: Ontology + FromPair,
{
    const RULE: Rule = Rule::OntologyDocument;
    fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context<'_>) -> Result<Self> {
        let mut pairs = pair.into_inner();

        // Build the prefix mapping and use it to build the ontology
        let mut prefixes = PrefixMapping::default();
        let mut inner = pairs.next().unwrap();
        while inner.as_rule() == Rule::PrefixDeclaration {
            let mut decl = inner.into_inner();
            let mut pname = decl.next().unwrap().into_inner();
            let iri = decl.next().unwrap().into_inner().next().unwrap();

            if let Some(prefix) = pname.next().unwrap().into_inner().next() {
                prefixes
                    .add_prefix(prefix.as_str(), iri.as_str())
                    .expect("grammar does not allow invalid prefixes");
            } else {
                prefixes.set_default(iri.as_str());
            }

            inner = pairs.next().unwrap();
        }

        let context = Context::new(ctx.build, &prefixes);
        O::from_pair(inner, &context).map(|ont| (ont, prefixes))
    }
}

// ---------------------------------------------------------------------------

impl FromPair for String {
    const RULE: Rule = Rule::QuotedString;
    fn from_pair_unchecked(pair: Pair<Rule>, _ctx: &Context<'_>) -> Result<Self> {
        let l = pair.as_str().len();
        let s = &pair.as_str()[1..l - 1];
        Ok(s.replace(r"\\", r"\").replace(r#"\""#, r#"""#))
    }
}

// ---------------------------------------------------------------------------

impl FromPair for SubObjectPropertyExpression {
    const RULE: Rule = Rule::SubObjectPropertyExpression;
    fn from_pair_unchecked(pair: Pair<Rule>, ctx: &Context<'_>) -> Result<Self> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::ObjectPropertyExpression => ObjectPropertyExpression::from_pair(inner, ctx)
                .map(SubObjectPropertyExpression::ObjectPropertyExpression),
            Rule::PropertyExpressionChain => {
                let mut objs = Vec::new();
                for pair in inner.into_inner() {
                    objs.push(ObjectPropertyExpression::from_pair(pair, ctx)?);
                }
                Ok(SubObjectPropertyExpression::ObjectPropertyChain(objs))
            }
            rule => unreachable!(
                "unexpected rule in SubObjectProperty::from_pair: {:?}",
                rule
            ),
        }
    }
}

// ---------------------------------------------------------------------------

impl FromPair for u32 {
    const RULE: Rule = Rule::NonNegativeInteger;
    fn from_pair_unchecked(pair: Pair<Rule>, _ctx: &Context<'_>) -> Result<Self> {
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
            let ctx = Context::new(&$build, &$prefixes);
            match OwlFunctionalParser::parse($rule, doc) {
                Ok(mut pairs) => {
                    let res = <$ty as FromPair>::from_pair(pairs.next().unwrap(), &ctx);
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
    fn anonymous_individual() {
        let build = Build::default();
        let prefixes = PrefixMapping::default();

        assert_parse_into!(
            AnonymousIndividual,
            Rule::AnonymousIndividual,
            build,
            prefixes,
            "_:anon",
            AnonymousIndividual(From::from("anon"))
        );
    }

    #[test]
    fn has_key() {
        let build = Build::default();
        let mut prefixes = PrefixMapping::default();
        prefixes
            .add_prefix("owl", "http://www.w3.org/2002/07/owl#")
            .unwrap();

        assert_parse_into!(
            AnnotatedAxiom,
            Rule::Axiom,
            build,
            prefixes,
            "HasKey( owl:Thing () (<http://www.example.com/issn>) )",
            AnnotatedAxiom::from(HasKey::new(
                ClassExpression::Class(build.class("http://www.w3.org/2002/07/owl#Thing")),
                vec![PropertyExpression::DataProperty(
                    build.data_property("http://www.example.com/issn")
                )],
            ))
        );
    }

    #[test]
    fn declare_class() {
        let build = Build::default();
        let mut prefixes = PrefixMapping::default();
        prefixes
            .add_prefix("owl", "http://www.w3.org/2002/07/owl#")
            .unwrap();

        assert_parse_into!(
            DeclareClass,
            Rule::ClassDeclaration,
            build,
            prefixes,
            "Class( owl:Thing )",
            DeclareClass(build.class("http://www.w3.org/2002/07/owl#Thing"))
        );

        assert_parse_into!(
            Axiom,
            Rule::Axiom,
            build,
            prefixes,
            "Declaration(Class(owl:Thing))",
            Axiom::DeclareClass(DeclareClass(
                build.class("http://www.w3.org/2002/07/owl#Thing")
            ))
        );

        assert_parse_into!(
            AnnotatedAxiom,
            Rule::Axiom,
            build,
            prefixes,
            "Declaration(Class(owl:Thing))",
            AnnotatedAxiom::from(DeclareClass(
                build.class("http://www.w3.org/2002/07/owl#Thing")
            ))
        );
    }

    #[test]
    fn import() {
        let build = Build::default();
        let prefixes = PrefixMapping::default();

        assert_parse_into!(
            Import,
            Rule::Import,
            build,
            prefixes,
            "Import(<http://example.com/path#ref>)",
            Import(build.iri("http://example.com/path#ref"))
        );
    }

    #[test]
    fn iri() {
        let build = Build::default();
        let mut prefixes = PrefixMapping::default();
        prefixes
            .add_prefix("ex", "http://example.com/path#")
            .unwrap();

        assert_parse_into!(
            IRI,
            Rule::IRI,
            build,
            prefixes,
            "<http://example.com/path#ref>",
            build.iri("http://example.com/path#ref")
        );

        assert_parse_into!(
            IRI,
            Rule::IRI,
            build,
            prefixes,
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

        let doc: (SetOntology, PrefixMapping) =
            FromPair::from_pair(pair, &Context::new(&build, &prefixes)).unwrap();
        assert_eq!(
            doc.1.mappings().collect::<HashSet<_>>(),
            expected.mappings().collect::<HashSet<_>>()
        );
    }

    #[test]
    fn same_individual() {
        let build = Build::default();
        let mut prefixes = PrefixMapping::default();
        prefixes
            .add_prefix("owl", "http://www.w3.org/2002/07/owl#")
            .unwrap();

        assert_parse_into!(
            AnnotatedAxiom,
            Rule::Axiom,
            build,
            prefixes,
            "SameIndividual( owl:Thing _:thing )",
            AnnotatedAxiom::from(SameIndividual(vec![
                Individual::Named(build.named_individual("http://www.w3.org/2002/07/owl#Thing")),
                Individual::Anonymous(AnonymousIndividual(From::from("thing"))),
            ]))
        );
    }
}
