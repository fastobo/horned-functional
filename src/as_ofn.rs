use std::collections::BTreeSet;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

use horned_owl::model::*;
use horned_owl::ontology::axiom_mapped::AxiomMappedOntology;
use horned_owl::ontology::indexed::ForIndex;
use horned_owl::vocab::WithIRI;

use super::Context;

/// Write a string literal while escaping `"` and `\` characters.
fn quote(mut s: &str, f: &mut Formatter<'_>) -> Result<(), Error> {
    f.write_str("\"")?;
    while let Some((i, c)) = s.chars().enumerate().find(|(_, c)| *c == '\\' || *c == '"') {
        f.write_str(&s[..i])?;
        match c {
            '\\' => f.write_str("\\\\")?,
            '"' => f.write_str("\\\"")?,
            _ => unreachable!(),
        }
        s = &s[i + 1..];
    }
    f.write_str(s)?;
    f.write_str("\"")
}

/// A trait for OWL elements that can be serialized to OWL Functional syntax.
pub trait AsFunctional<A: ForIRI> {
    /// Get a handle for displaying the element in functional syntax.
    ///
    /// Instead of returning a `String`, this method returns an opaque struct
    /// that implements `Display`, which can be used to write to a file without
    /// having to build a fully-serialized string first, or to just get a string
    /// with the `ToString` implementation.
    ///
    /// # Example
    /// ```
    /// # use horned_owl::model::DeclareClass;
    /// # let build = horned_owl::model::Build::new_arc();
    /// use horned_functional::AsFunctional;
    ///
    /// let axiom = DeclareClass(build.class("http://xmlns.com/foaf/0.1/Person"));
    /// assert_eq!(
    ///     axiom.as_ofn().to_string(),
    ///     "Declaration(Class(<http://xmlns.com/foaf/0.1/Person>))"
    /// );
    /// ```
    fn as_ofn<'t>(&'t self) -> Functional<'t, Self, A> {
        Functional(&self, None, None)
    }

    /// Get a handle for displaying the element, using the given context.
    ///
    /// Use the context to pass around a `PrefixMapping`, allowing the
    /// functional representation to be written using abbreviated IRIs
    /// when possible.
    ///
    /// # Example
    /// ```
    /// # use horned_owl::model::DeclareClass;
    /// # let build = horned_owl::model::Build::new_arc();
    /// use horned_functional::AsFunctional;
    /// use horned_functional::Context;
    ///
    /// let mut prefixes = curie::PrefixMapping::default();
    /// prefixes.add_prefix("foaf", "http://xmlns.com/foaf/0.1/");
    ///
    /// let axiom = DeclareClass(build.class("http://xmlns.com/foaf/0.1/Person"));
    /// assert_eq!(
    ///     axiom.as_ofn_ctx(&Context::from(&prefixes)).to_string(),
    ///     "Declaration(Class(foaf:Person))"
    /// );
    /// ```
    fn as_ofn_ctx<'t>(&'t self, context: &'t Context<'t, A>) -> Functional<'t, Self, A> {
        Functional(&self, Some(context), None)
    }
}

/// A wrapper for displaying an OWL2 element in functional syntax.
#[derive(Debug)]
pub struct Functional<'t, T: ?Sized, A: ForIRI>(
    // the element to display
    &'t T,
    // an eventual context to use (for IRI prefixes)
    Option<&'t Context<'t, A>>,
    // an eventual set of annotations (to render inside axioms)
    Option<&'t BTreeSet<Annotation<A>>>,
);

// ---------------------------------------------------------------------------

macro_rules! derive_vec {
    ($A:ident, $t:ty) => {
        impl<'a, $A: ForIRI> Display for Functional<'a, Vec<$t>, $A> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                for (i, x) in self.0.iter().enumerate() {
                    if i != 0 {
                        f.write_str(" ")?;
                    }
                    write!(f, "{}", Functional(x, self.1, None))?;
                }
                Ok(())
            }
        }
    };
}

derive_vec!(A, ClassExpression<A>);
derive_vec!(A, DataRange<A>);
derive_vec!(A, Individual<A>);
derive_vec!(A, ObjectPropertyExpression<A>);
derive_vec!(A, FacetRestriction<A>);
derive_vec!(A, Literal<A>);
derive_vec!(A, DataProperty<A>);

// ---------------------------------------------------------------------------

macro_rules! derive_tuple1 {
    ($A:ident, $t:ty) => {
        impl<'a, $A: ForIRI> Display for Functional<'a, (&$t,), $A> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                write!(f, "{}", Functional(self.0 .0, self.1, None),)
            }
        }
    };
}

derive_tuple1!(A, IRI<A>);
derive_tuple1!(A, DataProperty<A>);
derive_tuple1!(A, ObjectPropertyExpression<A>);
derive_tuple1!(A, Vec<Individual<A>>);
derive_tuple1!(A, Vec<ClassExpression<A>>);
derive_tuple1!(A, Vec<DataProperty<A>>);
derive_tuple1!(A, Vec<ObjectPropertyExpression<A>>);

macro_rules! derive_tuple2 {
    ($A:ident, $t1:ty, $t2:ty) => {
        impl<'a, $A: ForIRI> Display for Functional<'a, (&$t1, &$t2), $A> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                write!(
                    f,
                    "{} {}",
                    Functional(self.0 .0, self.1, None),
                    Functional(self.0 .1, self.1, None),
                )
            }
        }
    };
}

derive_tuple2!(A, Class<A>, Vec<ClassExpression<A>>);
derive_tuple2!(A, Datatype<A>, DataRange<A>);
derive_tuple2!(A, ClassExpression<A>, Individual<A>);
derive_tuple2!(A, ObjectProperty<A>, ObjectProperty<A>);
derive_tuple2!(A, ObjectPropertyExpression<A>, ClassExpression<A>);
derive_tuple2!(A, AnnotationProperty<A>, AnnotationValue<A>);
derive_tuple2!(A, AnnotationProperty<A>, IRI<A>);
derive_tuple2!(A, ClassExpression<A>, ClassExpression<A>);
derive_tuple2!(A, AnnotationProperty<A>, AnnotationProperty<A>);
derive_tuple2!(A, DataProperty<A>, DataProperty<A>);
derive_tuple2!(A, DataProperty<A>, DataRange<A>);
derive_tuple2!(A, DataProperty<A>, ClassExpression<A>);
derive_tuple2!(
    A,
    SubObjectPropertyExpression<A>,
    ObjectPropertyExpression<A>
);

macro_rules! derive_tuple3 {
    ($A:ident, $t1:ty, $t2:ty, $t3:ty) => {
        impl<'a, $A: ForIRI> Display for Functional<'a, (&$t1, &$t2, &$t3), $A> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                write!(
                    f,
                    "{} {} {}",
                    Functional(self.0 .0, self.1, None),
                    Functional(self.0 .1, self.1, None),
                    Functional(self.0 .2, self.1, None),
                )
            }
        }
    };
}

derive_tuple3!(A, DataProperty<A>, Individual<A>, Literal<A>);
derive_tuple3!(A, ObjectPropertyExpression<A>, Individual<A>, Individual<A>);

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI> Display for Functional<'a, BTreeSet<Annotation<A>>, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for (i, x) in self.0.iter().enumerate() {
            if i != 0 {
                f.write_str(" ")?;
            }
            write!(f, "{}", Functional(x, self.1, None))?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

macro_rules! derive_declaration {
    ($A:ident, $ty:ty, $inner:ty, $name:ident) => {
        impl<'a, $A: ForIRI> Display for Functional<'a, $ty, $A> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                write!(
                    f,
                    concat!("Declaration(", stringify!($name), "({}))"),
                    Functional(&self.0 .0, self.1, None)
                )
            }
        }

        impl<$A: ForIRI> AsFunctional<$A> for $ty {}
    };
}

derive_declaration!(A, DeclareClass<A>, Class<A>, Class);
derive_declaration!(
    A,
    DeclareAnnotationProperty<A>,
    AnnotationProperty<A>,
    AnnotationProperty
);
derive_declaration!(
    A,
    DeclareObjectProperty<A>,
    ObjectProperty<A>,
    ObjectProperty
);
derive_declaration!(A, DeclareDataProperty<A>, DataProperty<A>, DataProperty);
derive_declaration!(
    A,
    DeclareNamedIndividual<A>,
    NamedIndividual<A>,
    NamedIndividual
);
derive_declaration!(A, DeclareDatatype<A>, Datatype<A>, Datatype);

// ---------------------------------------------------------------------------

macro_rules! derive_wrapper {
    ($A:ident, $ty:ty) => {
        impl<'a, $A: ForIRI> Display for Functional<'a, $ty, $A> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                write!(f, "{}", Functional(&self.0 .0, self.1, None))
            }
        }

        impl<$A: ForIRI> AsFunctional<$A> for $ty {}
    };
}

derive_wrapper!(A, AnnotationProperty<A>);
derive_wrapper!(A, Class<A>);
derive_wrapper!(A, DataProperty<A>);
derive_wrapper!(A, Datatype<A>);
derive_wrapper!(A, NamedIndividual<A>);
derive_wrapper!(A, OntologyAnnotation<A>);
derive_wrapper!(A, ObjectProperty<A>);

// ---------------------------------------------------------------------------

macro_rules! derive_axiom {
    ($A:ident, $ty:ty, $name:ident ( $($field:tt),* )) => {
        impl<'a, $A: ForIRI> Display for Functional<'a, $ty, $A> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                if let Some(annotations) = self.2 {
                    write!(
                        f,
                        concat!(stringify!($name), "({} {})"),
                        Functional(annotations, self.1, None),
                        Functional(&($(&self.0.$field,)*), self.1, None)
                    )
                } else {
                    write!(
                        f,
                        concat!(stringify!($name), "({})"),
                        Functional(&($(&self.0.$field,)*), self.1, None)
                    )
                }
            }
        }

        impl<$A: ForIRI> AsFunctional<$A> for $ty {}
    };
}

derive_axiom!(A, Annotation<A>, Annotation(ap, av));
derive_axiom!(
    A,
    AnnotationPropertyRange<A>,
    AnnotationPropertyRange(ap, iri)
);
derive_axiom!(
    A,
    AnnotationPropertyDomain<A>,
    AnnotationPropertyDomain(ap, iri)
);
derive_axiom!(A, AsymmetricObjectProperty<A>, AsymmetricObjectProperty(0));
derive_axiom!(A, ClassAssertion<A>, ClassAssertion(ce, i));
derive_axiom!(
    A,
    DataPropertyAssertion<A>,
    DataPropertyAssertion(dp, from, to)
);
derive_axiom!(A, DataPropertyDomain<A>, DataPropertyDomain(dp, ce));
derive_axiom!(A, DataPropertyRange<A>, DataPropertyRange(dp, dr));
derive_axiom!(A, DatatypeDefinition<A>, DatatypeDefinition(kind, range));
derive_axiom!(A, DifferentIndividuals<A>, DifferentIndividuals(0));
derive_axiom!(A, DisjointClasses<A>, DisjointClasses(0));
derive_axiom!(A, DisjointDataProperties<A>, DisjointDataProperties(0));
derive_axiom!(A, DisjointObjectProperties<A>, DisjointObjectProperties(0));
derive_axiom!(A, DisjointUnion<A>, DisjointUnion(0, 1));
derive_axiom!(A, EquivalentClasses<A>, EquivalentClasses(0));
derive_axiom!(A, EquivalentDataProperties<A>, EquivalentDataProperties(0));
derive_axiom!(
    A,
    EquivalentObjectProperties<A>,
    EquivalentObjectProperties(0)
);
derive_axiom!(A, FunctionalObjectProperty<A>, FunctionalObjectProperty(0));
derive_axiom!(A, FunctionalDataProperty<A>, FunctionalDataProperty(0));
derive_axiom!(A, Import<A>, Import(0));
derive_axiom!(
    A,
    InverseFunctionalObjectProperty<A>,
    InverseFunctionalObjectProperty(0)
);
derive_axiom!(A, InverseObjectProperties<A>, InverseObjectProperties(0, 1));
derive_axiom!(
    A,
    IrreflexiveObjectProperty<A>,
    IrreflexiveObjectProperty(0)
);
derive_axiom!(
    A,
    NegativeDataPropertyAssertion<A>,
    NegativeDataPropertyAssertion(dp, from, to)
);
derive_axiom!(
    A,
    NegativeObjectPropertyAssertion<A>,
    NegativeObjectPropertyAssertion(ope, from, to)
);
derive_axiom!(
    A,
    ObjectPropertyAssertion<A>,
    ObjectPropertyAssertion(ope, from, to)
);
derive_axiom!(A, ObjectPropertyDomain<A>, ObjectPropertyDomain(ope, ce));
derive_axiom!(A, ObjectPropertyRange<A>, ObjectPropertyRange(ope, ce));
derive_axiom!(A, ReflexiveObjectProperty<A>, ReflexiveObjectProperty(0));
derive_axiom!(A, SameIndividual<A>, SameIndividual(0));
derive_axiom!(A, SubClassOf<A>, SubClassOf(sub, sup));
derive_axiom!(
    A,
    SubAnnotationPropertyOf<A>,
    SubAnnotationPropertyOf(sub, sup)
);
derive_axiom!(A, SubDataPropertyOf<A>, SubDataPropertyOf(sub, sup));
derive_axiom!(A, SubObjectPropertyOf<A>, SubObjectPropertyOf(sub, sup));
derive_axiom!(A, SymmetricObjectProperty<A>, SymmetricObjectProperty(0));
derive_axiom!(A, TransitiveObjectProperty<A>, TransitiveObjectProperty(0));

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI> Display for Functional<'a, AnnotatedAxiom<A>, A> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        Functional(&self.0.axiom, self.1, Some(&self.0.ann)).fmt(f)
    }
}

impl<A: ForIRI> AsFunctional<A> for AnnotatedAxiom<A> {}

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI> Display for Functional<'a, AnnotationAssertion<A>, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "AnnotationAssertion({} {} {})",
            Functional(&self.0.ann.ap, self.1, None),
            Functional(&self.0.subject, self.1, None),
            Functional(&self.0.ann.av, self.1, None),
        )
    }
}

impl<A: ForIRI> AsFunctional<A> for AnnotationAssertion<A> {}

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI> Display for Functional<'a, AnnotationSubject<A>, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use AnnotationSubject::*;
        match &self.0 {
            IRI(iri) => Functional(iri, self.1, None).fmt(f),
            AnonymousIndividual(anon) => Functional(anon, self.1, None).fmt(f),
        }
    }
}

impl<A: ForIRI> AsFunctional<A> for AnnotationSubject<A> {}

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI> Display for Functional<'a, AnnotationValue<A>, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use AnnotationValue::*;
        match &self.0 {
            Literal(lit) => Functional(lit, self.1, None).fmt(f),
            IRI(iri) => Functional(iri, self.1, None).fmt(f),
        }
    }
}

impl<A: ForIRI> AsFunctional<A> for AnnotationValue<A> {}

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI> Display for Functional<'a, AnonymousIndividual<A>, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.0.fmt(f)
    }
}

impl<A: ForIRI> AsFunctional<A> for AnonymousIndividual<A> {}

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI> Display for Functional<'a, Axiom<A>, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        macro_rules! enum_impl {
            ($($variant:ident,)*) => {
                match self.0 {
                    $(Axiom::$variant(axiom) => {
                        Functional(axiom, self.1, self.2).fmt(f)
                    }),*
                }
            }
        }
        enum_impl!(
            OntologyAnnotation,
            Import,
            DeclareClass,
            DeclareObjectProperty,
            DeclareAnnotationProperty,
            DeclareDataProperty,
            DeclareNamedIndividual,
            DeclareDatatype,
            SubClassOf,
            EquivalentClasses,
            DisjointClasses,
            DisjointUnion,
            SubObjectPropertyOf,
            EquivalentObjectProperties,
            DisjointObjectProperties,
            InverseObjectProperties,
            ObjectPropertyDomain,
            ObjectPropertyRange,
            FunctionalObjectProperty,
            InverseFunctionalObjectProperty,
            ReflexiveObjectProperty,
            IrreflexiveObjectProperty,
            SymmetricObjectProperty,
            AsymmetricObjectProperty,
            TransitiveObjectProperty,
            SubDataPropertyOf,
            EquivalentDataProperties,
            DisjointDataProperties,
            DataPropertyDomain,
            DataPropertyRange,
            FunctionalDataProperty,
            DatatypeDefinition,
            HasKey,
            SameIndividual,
            DifferentIndividuals,
            ClassAssertion,
            ObjectPropertyAssertion,
            NegativeObjectPropertyAssertion,
            DataPropertyAssertion,
            NegativeDataPropertyAssertion,
            AnnotationAssertion,
            SubAnnotationPropertyOf,
            AnnotationPropertyDomain,
            AnnotationPropertyRange,
        )
    }
}

impl<A: ForIRI> AsFunctional<A> for Axiom<A> {}

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI> Display for Functional<'a, ClassExpression<A>, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use ClassExpression::*;
        match self.0 {
            Class(exp) => Functional(exp, self.1, None).fmt(f),
            ObjectIntersectionOf(classes) => {
                write!(
                    f,
                    "ObjectIntersectionOf({})",
                    Functional(classes, self.1, None)
                )
            }
            ObjectUnionOf(classes) => {
                write!(f, "ObjectUnionOf({})", Functional(classes, self.1, None))
            }
            ObjectComplementOf(class) => {
                write!(
                    f,
                    "ObjectComplementOf({})",
                    Functional(class.as_ref(), self.1, None)
                )
            }
            ObjectOneOf(individuals) => {
                write!(f, "ObjectOneOf({})", Functional(individuals, self.1, None))
            }
            ObjectSomeValuesFrom { ope, bce } => {
                write!(
                    f,
                    "ObjectSomeValuesFrom({} {})",
                    Functional(ope, self.1, None),
                    Functional(bce.as_ref(), self.1, None)
                )
            }
            ObjectAllValuesFrom { ope, bce } => {
                write!(
                    f,
                    "ObjectAllValuesFrom({} {})",
                    Functional(ope, self.1, None),
                    Functional(bce.as_ref(), self.1, None)
                )
            }
            ObjectHasValue { ope, i } => {
                write!(
                    f,
                    "ObjectHasValue({} {})",
                    Functional(ope, self.1, None),
                    Functional(i, self.1, None)
                )
            }
            ObjectHasSelf(ope) => {
                write!(f, "ObjectHasSelf({})", Functional(ope, self.1, None))
            }
            ObjectMinCardinality { n, ope, bce } => {
                write!(
                    f,
                    "ObjectMinCardinality({} {} {})",
                    n,
                    Functional(ope, self.1, None),
                    Functional(bce.as_ref(), self.1, None)
                )
            }
            ObjectMaxCardinality { n, ope, bce } => {
                write!(
                    f,
                    "ObjectMaxCardinality({} {} {})",
                    n,
                    Functional(ope, self.1, None),
                    Functional(bce.as_ref(), self.1, None)
                )
            }
            ObjectExactCardinality { n, ope, bce } => {
                write!(
                    f,
                    "ObjectExactCardinality({} {} {})",
                    n,
                    Functional(ope, self.1, None),
                    Functional(bce.as_ref(), self.1, None)
                )
            }
            DataSomeValuesFrom { dp, dr } => {
                write!(
                    f,
                    "DataSomeValuesFrom({} {})",
                    Functional(dp, self.1, None),
                    Functional(dr, self.1, None)
                )
            }
            DataAllValuesFrom { dp, dr } => {
                write!(
                    f,
                    "DataAllValuesFrom({} {})",
                    Functional(dp, self.1, None),
                    Functional(dr, self.1, None)
                )
            }
            DataHasValue { dp, l } => {
                write!(
                    f,
                    "DataHasValue({} {})",
                    Functional(dp, self.1, None),
                    Functional(l, self.1, None)
                )
            }
            DataMinCardinality { n, dp, dr } => {
                write!(
                    f,
                    "DataMinCardinality({} {} {})",
                    n,
                    Functional(dp, self.1, None),
                    Functional(dr, self.1, None)
                )
            }
            DataMaxCardinality { n, dp, dr } => {
                write!(
                    f,
                    "DataMaxCardinality({} {} {})",
                    n,
                    Functional(dp, self.1, None),
                    Functional(dr, self.1, None)
                )
            }
            DataExactCardinality { n, dp, dr } => {
                write!(
                    f,
                    "DataMaxCardinality({} {} {})",
                    n,
                    Functional(dp, self.1, None),
                    Functional(dr, self.1, None)
                )
            }
        }
    }
}

impl<A: ForIRI> AsFunctional<A> for ClassExpression<A> {}

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI> Display for Functional<'a, DataRange<A>, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use DataRange::*;
        match self.0 {
            Datatype(dt) => Functional(dt, self.1, None).fmt(f),
            DataIntersectionOf(dts) => {
                write!(f, "DataIntersectionOf({})", Functional(dts, self.1, None))
            }
            DataUnionOf(dts) => {
                write!(f, "DataUnionOf({})", Functional(dts, self.1, None))
            }
            DataComplementOf(dt) => {
                write!(
                    f,
                    "DataComplementOf({})",
                    Functional(dt.as_ref(), self.1, None)
                )
            }
            DataOneOf(lits) => {
                write!(f, "DataUnionOf({})", Functional(lits, self.1, None))
            }
            DatatypeRestriction(dt, frs) => {
                write!(
                    f,
                    "DatatypeRestriction({} {})",
                    Functional(dt, self.1, None),
                    Functional(frs, self.1, None)
                )
            }
        }
    }
}

impl<A: ForIRI> AsFunctional<A> for DataRange<A> {}

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI> Display for Functional<'a, Facet, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(self.0.iri_str())
    }
}

impl<A: ForIRI> AsFunctional<A> for Facet {}

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI> Display for Functional<'a, FacetRestriction<A>, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{} {}",
            Functional(&self.0.f, self.1, None),
            Functional(&self.0.l, self.1, None)
        )
    }
}

impl<A: ForIRI> AsFunctional<A> for FacetRestriction<A> {}

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI> Display for Functional<'a, HasKey<A>, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "HasKey({} ", Functional(&self.0.ce, self.1, None))?;

        f.write_str("(")?;
        let mut n = 0;
        for pe in self.0.vpe.iter() {
            if let PropertyExpression::ObjectPropertyExpression(ope) = pe {
                if n != 0 {
                    f.write_str(" ")?;
                }
                Functional(ope, self.1, None).fmt(f)?;
                n += 1
            }
        }
        f.write_str(") ")?;

        f.write_str("(")?;
        let mut n = 0;
        for pe in self.0.vpe.iter() {
            if let PropertyExpression::DataProperty(dp) = pe {
                if n != 0 {
                    f.write_str(" ")?;
                }
                Functional(dp, self.1, None).fmt(f)?;
                n += 1
            }
        }
        f.write_str(") ")?;

        f.write_str(")")
    }
}

impl<A: ForIRI> AsFunctional<A> for HasKey<A> {}

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI> Display for Functional<'a, IRI<A>, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if let Some(prefixes) = self.1.as_ref().and_then(|ctx| ctx.prefixes) {
            match prefixes.shrink_iri(self.0) {
                Ok(curie) => write!(f, "{}", curie),
                Err(_) => write!(f, "<{}>", self.0),
            }
        } else {
            write!(f, "<{}>", self.0)
        }
    }
}

impl<A: ForIRI> AsFunctional<A> for IRI<A> {}

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI> Display for Functional<'a, Individual<A>, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use Individual::*;
        match self.0 {
            Named(i) => Functional(i, self.1, None).fmt(f),
            Anonymous(i) => Functional(i, self.1, None).fmt(f),
        }
    }
}

impl<A: ForIRI> AsFunctional<A> for Individual<A> {}

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI> Display for Functional<'a, Literal<A>, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self.0 {
            Literal::Simple { literal } => quote(&literal, f),
            Literal::Language { literal, lang } => {
                quote(&literal, f)?;
                write!(f, "@{}", lang)
            }
            Literal::Datatype {
                literal,
                datatype_iri,
            } => {
                quote(&literal, f)?;
                write!(f, "^^{}", Functional(datatype_iri, self.1, None))
            }
        }
    }
}

impl<A: ForIRI> AsFunctional<A> for Literal<A> {}

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI> Display for Functional<'a, ObjectPropertyExpression<A>, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use ObjectPropertyExpression::*;
        match self.0 {
            ObjectProperty(op) => Functional(op, self.1, None).fmt(f),
            InverseObjectProperty(op) => {
                write!(f, "ObjectInverseOf({})", Functional(op, self.1, None))
            }
        }
    }
}

impl<A: ForIRI> AsFunctional<A> for ObjectPropertyExpression<A> {}

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI> Display for Functional<'a, SubObjectPropertyExpression<A>, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use SubObjectPropertyExpression::*;
        match self.0 {
            ObjectPropertyExpression(ope) => Functional(ope, self.1, None).fmt(f),
            ObjectPropertyChain(chain) => {
                write!(
                    f,
                    "ObjectPropertyChain({})",
                    Functional(chain, self.1, None)
                )
            }
        }
    }
}

impl<A: ForIRI> AsFunctional<A> for SubObjectPropertyExpression<A> {}

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI> Display for Functional<'a, curie::PrefixMapping, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for (name, value) in self.0.mappings() {
            writeln!(f, "Prefix({}:=<{}>)", name, value)?;
        }
        Ok(())
    }
}

impl<A: ForIRI> AsFunctional<A> for curie::PrefixMapping {}

// ---------------------------------------------------------------------------

impl<'a, A: ForIRI, AA: ForIndex<A>> Display for Functional<'a, AxiomMappedOntology<A, AA>, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        // open the Ontology element
        f.write_str("Ontology(")?;
        // write the IRI and Version IRI if any
        let id = self.0.id();
        if let Some(iri) = &id.iri {
            write!(f, "{}", Functional(iri, self.1, None))?;
            if let Some(viri) = &id.viri {
                writeln!(f, " {}", Functional(viri, self.1, None))?;
            } else {
                writeln!(f)?;
            }
        }
        // write imports first
        for axiom in self.0.i().import() {
            writeln!(f, "{}", Functional(axiom, self.1, None))?;
        }
        // then write ontology annotations
        for axiom in self.0.i().ontology_annotation() {
            writeln!(f, "{}", Functional(axiom, self.1, None))?;
        }
        // then write the rest
        for axiom in self.0.i().iter() {
            let kind = axiom.axiom.kind();
            if kind != AxiomKind::OntologyAnnotation && kind != AxiomKind::Import {
                writeln!(f, "{}", Functional(axiom, self.1, None))?;
            }
        }
        // and close the Ontology element
        f.write_str(")")
    }
}

impl<A: ForIRI, AA: ForIndex<A>> AsFunctional<A> for AxiomMappedOntology<A, AA> {}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_ofn_declareclass() {
        let build = Build::new_arc();
        let decl = DeclareClass(build.class("http://purl.obolibrary.org/obo/BFO_0000001"));
        let ofn = format!("{}", decl.as_ofn());
        assert_eq!(
            "Declaration(Class(<http://purl.obolibrary.org/obo/BFO_0000001>))",
            &ofn
        );
    }

    #[test]
    fn test_ofn_literal_simple() {
        let lit = Literal::<String>::Simple {
            literal: String::from("test"),
        };
        let ofn = format!("{}", lit.as_ofn());
        assert_eq!(r#""test""#, &ofn);

        let lit = Literal::<String>::Simple {
            literal: String::from("test\""),
        };
        let ofn = format!("{}", lit.as_ofn());
        assert_eq!(r#""test\"""#, &ofn);

        let lit = Literal::<String>::Simple {
            literal: String::from("test\\"),
        };
        let ofn = format!("{}", lit.as_ofn());
        assert_eq!(r#""test\\""#, &ofn);
    }

    #[test]
    fn test_ofn_literal_language() {
        let lit = Literal::<String>::Language {
            literal: String::from("hello"),
            lang: String::from("en"),
        };
        let ofn = format!("{}", lit.as_ofn());
        assert_eq!(r#""hello"@en"#, &ofn);
    }

    #[test]
    fn test_ofn_literal_datatype() {
        let build = Build::new_arc();
        let lit = Literal::Datatype {
            literal: String::from("hello"),
            datatype_iri: build.iri("http://www.w3.org/2001/XMLSchema#string"),
        };
        let ofn = format!("{}", lit.as_ofn());
        assert_eq!(
            r#""hello"^^<http://www.w3.org/2001/XMLSchema#string>"#,
            &ofn
        );
    }

    #[test]
    fn test_ofn_import() {
        let build = Build::new_arc();
        let import = Import(build.iri("http://example.com/"));
        let ofn = format!("{}", import.as_ofn());
        assert_eq!("Import(<http://example.com/>)", ofn);
    }

    #[test]
    fn test_ofn_curie() {
        let build = Build::new_arc();
        let mut prefixes = curie::PrefixMapping::default();
        prefixes
            .add_prefix("obo", "http://purl.obolibrary.org/obo/")
            .ok();
        let context = Context::from(&prefixes);

        let decl = DeclareClass(build.class("http://purl.obolibrary.org/obo/BFO_0000001"));
        let ofn = format!("{}", decl.as_ofn_ctx(&context));
        assert_eq!("Declaration(Class(obo:BFO_0000001))", ofn);

        let decl = DeclareClass(build.class("http://xmlns.com/foaf/0.1/Person"));
        let ofn = format!("{}", decl.as_ofn_ctx(&context));
        assert_eq!(
            "Declaration(Class(<http://xmlns.com/foaf/0.1/Person>))",
            ofn
        );
    }

    #[test]
    fn test_annotated_axiom() {
        let build = Build::new_arc();
        let mut prefixes = curie::PrefixMapping::default();
        prefixes
            .add_prefix("obo", "http://purl.obolibrary.org/obo/")
            .ok();
        prefixes
            .add_prefix("oboInOwl", "http://www.geneontology.org/formats/oboInOwl#")
            .ok();
        let context = Context::from(&prefixes);

        let axiom = EquivalentClasses(vec![
            ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/HAO_0000935")),
            ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/HAO_0000933")),
        ]);
        let annotated = AnnotatedAxiom {
            axiom: Axiom::EquivalentClasses(axiom),
            ann: BTreeSet::from_iter([Annotation {
                ap: build
                    .annotation_property("http://www.geneontology.org/formats/oboInOwl#hasDbXref"),
                av: AnnotationValue::Literal(Literal::Simple {
                    literal: "http://api.hymao.org/api/ref/67791".into(),
                }),
            }]),
        };

        let ofn = annotated.as_ofn_ctx(&context).to_string();
        assert_eq!(
            ofn,
            r#"EquivalentClasses(Annotation(oboInOwl:hasDbXref "http://api.hymao.org/api/ref/67791") obo:HAO_0000935 obo:HAO_0000933)"#
        )
    }
}
