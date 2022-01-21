use std::collections::BTreeSet;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

use horned_owl::model as owl;
use horned_owl::model::Kinded;
use horned_owl::model::Ontology;
use horned_owl::ontology::axiom_mapped::AxiomMappedOntology;
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

/// A wrapper for displaying an OWL2 element in functional syntax.
#[derive(Debug)]
pub struct Functional<'t, T: ?Sized + AsFunctional>(
    // the element to display
    &'t T,
    // an eventual context to use (for IRI prefixes)
    Option<&'t Context<'t>>,
    // an eventual set of annotations (to render inside axioms)
    Option<&'t BTreeSet<owl::Annotation>>,
);

macro_rules! derive_declaration {
    (owl::$ty:ident, $inner:ident) => {
        impl Display for Functional<'_, owl::$ty> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                write!(
                    f,
                    concat!("Declaration(", stringify!($inner), "({}))"),
                    Functional(&self.0 .0, self.1, None)
                )
            }
        }
    };
}

derive_declaration!(owl::DeclareClass, Class);
derive_declaration!(owl::DeclareAnnotationProperty, AnnotationProperty);
derive_declaration!(owl::DeclareObjectProperty, ObjectProperty);
derive_declaration!(owl::DeclareDataProperty, DataProperty);
derive_declaration!(owl::DeclareNamedIndividual, NamedIndividual);
derive_declaration!(owl::DeclareDatatype, Datatype);

macro_rules! derive_wrapper {
    (owl::$ty:ident) => {
        impl Display for Functional<'_, owl::$ty> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                write!(f, "{}", Functional(&self.0 .0, self.1, None))
            }
        }
    };
}

derive_wrapper!(owl::AnnotationProperty);
derive_wrapper!(owl::Class);
derive_wrapper!(owl::DataProperty);
derive_wrapper!(owl::Datatype);
derive_wrapper!(owl::NamedIndividual);
derive_wrapper!(owl::OntologyAnnotation);
derive_wrapper!(owl::ObjectProperty);

macro_rules! derive_axiom {
    (owl::$ty:ident ( $($field:tt),* )) => {
        impl Display for Functional<'_, owl::$ty> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                if let Some(annotations) = self.2 {
                    write!(
                        f,
                        concat!(stringify!($ty), "({} {})"),
                        Functional(annotations, self.1, None),
                        Functional(&($(&self.0.$field,)*), self.1, None)
                    )
                } else {
                    write!(
                        f,
                        concat!(stringify!($ty), "({})"),
                        Functional(&($(&self.0.$field,)*), self.1, None)
                    )
                }
            }
        }
    };
}

derive_axiom!(owl::Annotation(ap, av));
derive_axiom!(owl::AnnotationPropertyRange(ap, iri));
derive_axiom!(owl::AnnotationPropertyDomain(ap, iri));
derive_axiom!(owl::AsymmetricObjectProperty(0));
derive_axiom!(owl::ClassAssertion(ce, i));
derive_axiom!(owl::DataPropertyAssertion(dp, from, to));
derive_axiom!(owl::DataPropertyDomain(dp, ce));
derive_axiom!(owl::DataPropertyRange(dp, dr));
derive_axiom!(owl::DatatypeDefinition(kind, range));
derive_axiom!(owl::DifferentIndividuals(0));
derive_axiom!(owl::DisjointClasses(0));
derive_axiom!(owl::DisjointDataProperties(0));
derive_axiom!(owl::DisjointObjectProperties(0));
derive_axiom!(owl::DisjointUnion(0, 1));
derive_axiom!(owl::EquivalentClasses(0));
derive_axiom!(owl::EquivalentDataProperties(0));
derive_axiom!(owl::EquivalentObjectProperties(0));
derive_axiom!(owl::FunctionalObjectProperty(0));
derive_axiom!(owl::FunctionalDataProperty(0));
derive_axiom!(owl::Import(0));
derive_axiom!(owl::InverseFunctionalObjectProperty(0));
derive_axiom!(owl::InverseObjectProperties(0, 1));
derive_axiom!(owl::IrreflexiveObjectProperty(0));
derive_axiom!(owl::NegativeDataPropertyAssertion(dp, from, to));
derive_axiom!(owl::NegativeObjectPropertyAssertion(ope, from, to));
derive_axiom!(owl::ObjectPropertyAssertion(ope, from, to));
derive_axiom!(owl::ObjectPropertyDomain(ope, ce));
derive_axiom!(owl::ObjectPropertyRange(ope, ce));
derive_axiom!(owl::ReflexiveObjectProperty(0));
derive_axiom!(owl::SameIndividual(0));
derive_axiom!(owl::SubClassOf(sub, sup));
derive_axiom!(owl::SubAnnotationPropertyOf(sub, sup));
derive_axiom!(owl::SubDataPropertyOf(sub, sup));
derive_axiom!(owl::SubObjectPropertyOf(sub, sup));
derive_axiom!(owl::SymmetricObjectProperty(0));
derive_axiom!(owl::TransitiveObjectProperty(0));

impl Display for Functional<'_, owl::AnnotatedAxiom> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        Functional(&self.0.axiom, self.1, Some(&self.0.ann)).fmt(f)
    }
}

impl Display for Functional<'_, owl::AnnotationAssertion> {
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

impl Display for Functional<'_, owl::AnnotationValue> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use owl::AnnotationValue::*;
        match &self.0 {
            Literal(lit) => Functional(lit, self.1, None).fmt(f),
            IRI(iri) => Functional(iri, self.1, None).fmt(f),
        }
    }
}

impl Display for Functional<'_, owl::AnonymousIndividual> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.0.fmt(f)
    }
}

impl Display for Functional<'_, owl::Axiom> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        macro_rules! enum_impl {
            ($($variant:ident,)*) => {
                match self.0 {
                    $(owl::Axiom::$variant(axiom) => {
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

impl Display for Functional<'_, owl::ClassExpression> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use owl::ClassExpression::*;
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

impl Display for Functional<'_, owl::DataRange> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use owl::DataRange::*;
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

impl Display for Functional<'_, owl::Facet> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(self.0.iri_str())
    }
}

impl Display for Functional<'_, owl::FacetRestriction> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{} {}",
            Functional(&self.0.f, self.1, None),
            Functional(&self.0.l, self.1, None)
        )
    }
}

impl Display for Functional<'_, owl::HasKey> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "HasKey({} ", Functional(&self.0.ce, self.1, None))?;

        f.write_str("(")?;
        let mut n = 0;
        for pe in self.0.vpe.iter() {
            if let owl::PropertyExpression::ObjectPropertyExpression(ope) = pe {
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
            if let owl::PropertyExpression::DataProperty(dp) = pe {
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

impl Display for Functional<'_, owl::IRI> {
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

impl Display for Functional<'_, owl::Individual> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use owl::Individual::*;
        match self.0 {
            Named(i) => Functional(i, self.1, None).fmt(f),
            Anonymous(i) => Functional(i, self.1, None).fmt(f),
        }
    }
}

impl Display for Functional<'_, owl::Literal> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self.0 {
            owl::Literal::Simple { literal } => quote(&literal, f),
            owl::Literal::Language { literal, lang } => {
                quote(&literal, f)?;
                write!(f, "@{}", lang)
            }
            owl::Literal::Datatype {
                literal,
                datatype_iri,
            } => {
                quote(&literal, f)?;
                write!(f, "^^{}", Functional(datatype_iri, self.1, None))
            }
        }
    }
}

impl Display for Functional<'_, owl::ObjectPropertyExpression> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use owl::ObjectPropertyExpression::*;
        match self.0 {
            ObjectProperty(op) => Functional(op, self.1, None).fmt(f),
            InverseObjectProperty(op) => {
                write!(f, "ObjectInverseOf({})", Functional(op, self.1, None))
            }
        }
    }
}

impl Display for Functional<'_, owl::SubObjectPropertyExpression> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use owl::SubObjectPropertyExpression::*;
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

impl Display for Functional<'_, curie::PrefixMapping> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for (name, value) in self.0.mappings() {
            writeln!(f, "Prefix({}:={})", name, value)?;
        }
        Ok(())
    }
}

impl<'a, T: 'a> Display for Functional<'a, BTreeSet<T>>
where
    Functional<'a, T>: Display,
{
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

impl<'a, T: 'a> Display for Functional<'a, Vec<T>>
where
    Functional<'a, T>: Display,
{
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

impl<'a, A: 'a> Display for Functional<'a, (&A,)>
where
    Functional<'a, A>: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", Functional(self.0 .0, self.1, None),)
    }
}

impl<'a, A: 'a, B: 'a> Display for Functional<'a, (&A, &B)>
where
    Functional<'a, A>: Display,
    Functional<'a, B>: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{} {}",
            Functional(self.0 .0, self.1, None),
            Functional(self.0 .1, self.1, None),
        )
    }
}

impl<'a, A: 'a, B: 'a, C: 'a> Display for Functional<'a, (&A, &B, &C)>
where
    Functional<'a, A>: Display,
    Functional<'a, B>: Display,
    Functional<'a, C>: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{} {} {}",
            Functional(self.0 .0, self.1, None),
            Functional(self.0 .1, self.1, None),
            Functional(self.0 .2, self.1, None)
        )
    }
}

impl Display for Functional<'_, AxiomMappedOntology> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        // open the Ontology element
        f.write_str("Ontology(")?;
        // write the IRI and Version IRI if any
        let id = self.0.id();
        if let Some(iri) = &id.iri {
            writeln!(f, "{}", Functional(iri, self.1, None))?;
            if let Some(viri) = &id.viri {
                writeln!(f, " {}", Functional(viri, self.1, None))?;
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
            if kind != owl::AxiomKind::OntologyAnnotation && kind != owl::AxiomKind::Import {
                writeln!(f, "{}", Functional(axiom, self.1, None))?;
            }
        }
        // and close the Ontology element
        f.write_str(")")
    }
}

/// A trait for OWL elements that can be serialized to OWL Functional syntax.
pub trait AsFunctional {
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
    /// # let build = horned_owl::model::Build::new();
    /// use horned_functional::AsFunctional;
    ///
    /// let axiom = DeclareClass(build.class("http://xmlns.com/foaf/0.1/Person"));
    /// assert_eq!(
    ///     axiom.as_ofn().to_string(),
    ///     "Declaration(Class(<http://xmlns.com/foaf/0.1/Person>))"
    /// );
    /// ```
    fn as_ofn<'t>(&'t self) -> Functional<'t, Self> {
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
    /// # let build = horned_owl::model::Build::new();
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
    fn as_ofn_ctx<'t>(&'t self, context: &'t Context<'t>) -> Functional<'t, Self> {
        Functional(&self, Some(context), None)
    }
}

impl<'t, T: 't> AsFunctional for T where Functional<'t, T>: Display {}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_ofn_declareclass() {
        let build = owl::Build::new();
        let decl = owl::DeclareClass(build.class("http://purl.obolibrary.org/obo/BFO_0000001"));
        let ofn = format!("{}", decl.as_ofn());
        assert_eq!(
            "Declaration(Class(<http://purl.obolibrary.org/obo/BFO_0000001>))",
            &ofn
        );
    }

    #[test]
    fn test_ofn_literal_simple() {
        let lit = owl::Literal::Simple {
            literal: String::from("test"),
        };
        let ofn = format!("{}", lit.as_ofn());
        assert_eq!(r#""test""#, &ofn);

        let lit = owl::Literal::Simple {
            literal: String::from("test\""),
        };
        let ofn = format!("{}", lit.as_ofn());
        assert_eq!(r#""test\"""#, &ofn);

        let lit = owl::Literal::Simple {
            literal: String::from("test\\"),
        };
        let ofn = format!("{}", lit.as_ofn());
        assert_eq!(r#""test\\""#, &ofn);
    }

    #[test]
    fn test_ofn_literal_language() {
        let lit = owl::Literal::Language {
            literal: String::from("hello"),
            lang: String::from("en"),
        };
        let ofn = format!("{}", lit.as_ofn());
        assert_eq!(r#""hello"@en"#, &ofn);
    }

    #[test]
    fn test_ofn_literal_datatype() {
        let build = owl::Build::new();
        let lit = owl::Literal::Datatype {
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
        let build = owl::Build::new();
        let import = owl::Import(build.iri("http://example.com/"));
        let ofn = format!("{}", import.as_ofn());
        assert_eq!("Import(<http://example.com/>)", ofn);
    }

    #[test]
    fn test_ofn_curie() {
        let build = owl::Build::new();
        let mut prefixes = curie::PrefixMapping::default();
        prefixes
            .add_prefix("obo", "http://purl.obolibrary.org/obo/")
            .ok();
        let context = Context::from(&prefixes);

        let decl = owl::DeclareClass(build.class("http://purl.obolibrary.org/obo/BFO_0000001"));
        let ofn = format!("{}", decl.as_ofn_ctx(&context));
        assert_eq!("Declaration(Class(obo:BFO_0000001))", ofn);

        let decl = owl::DeclareClass(build.class("http://xmlns.com/foaf/0.1/Person"));
        let ofn = format!("{}", decl.as_ofn_ctx(&context));
        assert_eq!(
            "Declaration(Class(<http://xmlns.com/foaf/0.1/Person>))",
            ofn
        );
    }

    #[test]
    fn test_annotated_axiom() {
        let build = owl::Build::new();
        let mut prefixes = curie::PrefixMapping::default();
        prefixes
            .add_prefix("obo", "http://purl.obolibrary.org/obo/")
            .ok();
        prefixes
            .add_prefix("oboInOwl", "http://www.geneontology.org/formats/oboInOwl#")
            .ok();
        let context = Context::from(&prefixes);

        let axiom = owl::EquivalentClasses(vec![
            owl::ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/HAO_0000935")),
            owl::ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/HAO_0000933")),
        ]);
        let annotated = owl::AnnotatedAxiom {
            axiom: owl::Axiom::EquivalentClasses(axiom),
            ann: BTreeSet::from_iter([owl::Annotation {
                ap: build
                    .annotation_property("http://www.geneontology.org/formats/oboInOwl#hasDbXref"),
                av: owl::AnnotationValue::Literal(owl::Literal::Simple {
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
