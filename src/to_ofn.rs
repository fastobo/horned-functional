use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Error;

use horned_owl::model as owl;

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
        s = &s[i+1..];
    }
    f.write_str(s)?;
    f.write_str("\"")
}

/// A wrapper for displaying an OWL2 element in functional syntax.
#[derive(Debug)]
pub struct Functional<'t, T: ?Sized>(&'t T);

macro_rules! derive_display {
    ($ty:ty) => {
        derive_display!($ty, "{}", 0);
    };
    ($ty:ty, $template:literal, $($field:tt),*) => {
        impl Display for Functional<'_, $ty> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                write!(
                    f,
                    $template,
                    $(self.0.$field.as_ofn()),*
                )
            }
        }
    };
}

derive_display!(owl::Annotation, "Annotation({} {})", ap, av);
derive_display!(owl::AnnotationProperty);
derive_display!(owl::AnnotationPropertyRange, "AnnotationPropertyRange({} {})", ap, iri);
derive_display!(owl::AnnotationPropertyDomain, "AnnotationPropertyDomain({} {})", ap, iri);
derive_display!(owl::AsymmetricObjectProperty, "AsymmetricObjectProperty({})", 0);
derive_display!(owl::Class);
derive_display!(owl::ClassAssertion, "ClassAssertion({} {})", ce, i);
derive_display!(owl::DataProperty);
derive_display!(owl::DataPropertyAssertion, "DataPropertyAssertion({} {} {})", dp, from, to);
derive_display!(owl::DataPropertyDomain, "DataPropertyDomain({} {})", dp, ce);
derive_display!(owl::DataPropertyRange, "DataPropertyRange({} {})", dp, dr);
derive_display!(owl::Datatype);
derive_display!(owl::DatatypeDefinition, "DatatypeDefinition({} {})", kind, range);
derive_display!(owl::DeclareClass, "Declaration(Class({}))", 0);
derive_display!(owl::DeclareAnnotationProperty, "Declaration(AnnotationProperty({}))", 0);
derive_display!(owl::DeclareObjectProperty, "Declaration(ObjectProperty({}))", 0);
derive_display!(owl::DeclareDataProperty, "Declaration(DataProperty({}))", 0);
derive_display!(owl::DeclareNamedIndividual, "Declaration(NamedIndividual({}))", 0);
derive_display!(owl::DeclareDatatype, "Declaration(Datatype({}))", 0);
derive_display!(owl::DifferentIndividuals, "DifferentIndividuals({})", 0);
derive_display!(owl::DisjointClasses, "DisjointClasses({})", 0);
derive_display!(owl::DisjointDataProperties, "DisjointDataProperties({})", 0);
derive_display!(owl::DisjointObjectProperties, "DisjointObjectProperties({})", 0);
derive_display!(owl::DisjointUnion, "DisjointUnion({} {})", 0, 1);
derive_display!(owl::EquivalentClasses, "EquivalentClasses({})", 0);
derive_display!(owl::EquivalentDataProperties, "EquivalentDataProperties({})", 0);
derive_display!(owl::EquivalentObjectProperties, "EquivalentObjectProperties({})", 0);
derive_display!(owl::FunctionalObjectProperty, "FunctionalObjectProperty({})", 0);
derive_display!(owl::FunctionalDataProperty, "FunctionalDataProperty({})", 0);
derive_display!(owl::Import, "Import({})", 0);
derive_display!(owl::InverseFunctionalObjectProperty, "InverseFunctionalObjectProperty({})", 0);
derive_display!(owl::InverseObjectProperties, "InverseObjectProperties({} {})", 0, 1);
derive_display!(owl::IrreflexiveObjectProperty, "IrreflexiveObjectProperty({})", 0);
derive_display!(owl::NamedIndividual);
derive_display!(owl::NegativeDataPropertyAssertion, "NegativeDataPropertyAssertion({} {} {})", dp, from, to);
derive_display!(owl::NegativeObjectPropertyAssertion, "NegativeObjectPropertyAssertion({} {} {})", ope, from, to);
derive_display!(owl::OntologyAnnotation);
derive_display!(owl::ObjectProperty);
derive_display!(owl::ObjectPropertyAssertion, "ObjectPropertyAssertion({} {} {})", ope, from, to);
derive_display!(owl::ObjectPropertyDomain, "ObjectPropertyDomain({} {})", ope, ce);
derive_display!(owl::ObjectPropertyRange, "ObjectPropertyRange({} {})", ope, ce);
derive_display!(owl::ReflexiveObjectProperty, "ReflexiveObjectProperty({})", 0);
derive_display!(owl::SameIndividual, "SameIndividual({})", 0);
derive_display!(owl::SubClassOf, "SubClassOf({} {})", sub, sup);
derive_display!(owl::SubAnnotationPropertyOf, "SubAnnotationPropertyOf({} {})", sub, sup);
derive_display!(owl::SubDataPropertyOf, "SubDataPropertyOf({} {})", sub, sup);
derive_display!(owl::SubObjectPropertyOf, "SubObjectPropertyOf({} {})", sub, sup);
derive_display!(owl::SymmetricObjectProperty, "SymmetricObjectProperty({})", 0);
derive_display!(owl::TransitiveObjectProperty, "TransitiveObjectProperty({})", 0);

impl Display for Functional<'_, owl::AnnotationAssertion> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "AnnotationAssertion({} {} {})",
            self.0.ann.ap.as_ofn(),
            self.0.subject.as_ofn(),
            self.0.ann.av.as_ofn(),
        )
    }
}

impl Display for Functional<'_, owl::AnnotationValue> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self.0 {
            owl::AnnotationValue::Literal(lit) => lit.as_ofn().fmt(f),
            owl::AnnotationValue::IRI(iri) => iri.as_ofn().fmt(f),
        }
    }
}

impl Display for Functional<'_, owl::Axiom> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        macro_rules! deref_impl {
            ($($variant:ident,)*) => {
                match self.0 {
                    $(owl::Axiom::$variant(axiom) => {
                        axiom.as_ofn().fmt(f)
                    }),*
                }
            }
        }
        deref_impl!(
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
            Class(exp) => exp.as_ofn().fmt(f),
            ObjectIntersectionOf(classes) => {
                write!(f, "ObjectIntersectionOf({})", classes.as_ofn())
            },
            ObjectUnionOf(classes) => {
                write!(f, "ObjectUnionOf({})", classes.as_ofn())
            },
            ObjectComplementOf(class) => {
                write!(f, "ObjectComplementOf({})", class.as_ofn())
            },
            ObjectOneOf(individuals) => {
                write!(f, "ObjectOneOf({})", individuals.as_ofn())
            },
            ObjectSomeValuesFrom { ope, bce } => {
                write!(f, "ObjectSomeValuesFrom({} {})", ope.as_ofn(), bce.as_ofn())
            },
            ObjectAllValuesFrom { ope, bce } => {
                write!(f, "ObjectAllValuesFrom({} {})", ope.as_ofn(), bce.as_ofn())
            },
            ObjectHasValue { ope, i } => {
                write!(f, "ObjectHasValue({} {})", ope.as_ofn(), i.as_ofn())
            },
            ObjectHasSelf(ope) => {
                write!(f, "ObjectHasSelf({})", ope.as_ofn())
            },
            ObjectMinCardinality { n, ope, bce } => {
                write!(f, "ObjectMinCardinality({} {} {})", n, ope.as_ofn(), bce.as_ofn())
            }
            ObjectMaxCardinality { n, ope, bce } => {
                write!(f, "ObjectMaxCardinality({} {} {})", n, ope.as_ofn(), bce.as_ofn())
            }
            ObjectExactCardinality { n, ope, bce } => {
                write!(f, "ObjectExactCardinality({} {} {})", n, ope.as_ofn(), bce.as_ofn())
            }
            DataSomeValuesFrom { dp, dr } => {
                write!(f, "DataSomeValuesFrom({} {})", dp.as_ofn(), dr.as_ofn())
            }
            DataAllValuesFrom { dp, dr } => {
                write!(f, "DataAllValuesFrom({} {})", dp.as_ofn(), dr.as_ofn())
            }
            DataHasValue { dp, l } => {
                write!(f, "DataHasValue({} {})", dp.as_ofn(), l.as_ofn())
            }
            DataMinCardinality { n, dp, dr } => {
                write!(f, "DataMinCardinality({} {} {})", n, dp.as_ofn(), dr.as_ofn())
            }
            DataMaxCardinality { n, dp, dr } => {
                write!(f, "DataMaxCardinality({} {} {})", n, dp.as_ofn(), dr.as_ofn())
            }
            DataExactCardinality { n, dp, dr } => {
                write!(f, "DataMaxCardinality({} {} {})", n, dp.as_ofn(), dr.as_ofn())
            }
        }
    }
}

impl Display for Functional<'_, owl::DataRange> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use owl::DataRange::*;
        match self.0 {
            Datatype(dt) => dt.as_ofn().fmt(f),
            DataIntersectionOf(dts) => {
                f.write_str("DataIntersectionOf(")?;
                for (i, dt) in dts.iter().enumerate() {
                    if i != 0 {
                        f.write_str(" ")?;
                    }
                    write!(f, "{}", dt.as_ofn())?;
                }
                f.write_str(")")
            },
            DataUnionOf(dts) => {
                f.write_str("DataUnionOf(")?;
                for (i, dt) in dts.iter().enumerate() {
                    if i != 0 {
                        f.write_str(" ")?;
                    }
                    write!(f, "{}", dt.as_ofn())?;
                }
                f.write_str(")")
            },
            DataComplementOf(dt) => {
                write!(f, "DataComplementOf({})", dt.as_ofn())
            },
            DataOneOf(lits) => {
                f.write_str("DataOneOf(")?;
                for (i, lit) in lits.iter().enumerate() {
                    if i != 0 {
                        f.write_str(" ")?;
                    }
                    write!(f, "{}", lit.as_ofn())?;
                }
                f.write_str(")")
            },
            DatatypeRestriction(dt, frs) => {
                write!(f, "DatatypeRestriction({} {})", dt.as_ofn(), frs.as_ofn())
            },
        }
    }
}

impl Display for Functional<'_, owl::FacetRestriction> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        unimplemented!()
    }
}

impl Display for Functional<'_, owl::HasKey> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "HasKey({}", self.0.ce.as_ofn())?;

        f.write_str("(")?;
        let mut n = 0;
        for pe in self.0.vpe.iter() {
            if let owl::PropertyExpression::ObjectPropertyExpression(ope) = pe {
                if n != 0 {
                    f.write_str(" ")?;
                }
                ope.as_ofn().fmt(f)?;
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
                dp.as_ofn().fmt(f)?;
                n += 1
            }
        }
        f.write_str(") ")?;

        f.write_str(")")
    }
}

impl Display for Functional<'_, owl::IRI> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "<{}>", self.0)
    }
}

impl Display for Functional<'_, owl::Individual> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        unimplemented!()
    }
}

impl Display for Functional<'_, owl::Literal> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self.0 {
            owl::Literal::Simple { literal } => {
                quote(&literal, f)
            }
            owl::Literal::Language { literal, lang } => {
                quote(&literal, f)?;
                write!(f, "@{}", lang)
            }
            owl::Literal::Datatype { literal, datatype_iri } => {
                quote(&literal, f)?;
                write!(f, "^^{}", datatype_iri.as_ofn())
            }
        }
    }
}

impl Display for Functional<'_, owl::ObjectPropertyExpression> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use owl::ObjectPropertyExpression::*;
        match self.0 {
            ObjectProperty(op) => op.as_ofn().fmt(f),
            InverseObjectProperty(op) => {
                write!(f, "ObjectInverseOf({})", op.as_ofn())
            }
        }
    }
}

impl Display for Functional<'_, owl::SubObjectPropertyExpression> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use owl::SubObjectPropertyExpression::*;
        match self.0 {
            ObjectPropertyExpression(ope) => ope.as_ofn().fmt(f),
            ObjectPropertyChain(chain) => {
                write!(f, "ObjectPropertyChain({})", chain.as_ofn())
            }
        }
    }
}

impl<'a, T: 'a> Display for Functional<'a, Vec<T>>
where
    Functional<'a, T>: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for (i, x) in self.0.iter().enumerate() {
            if i != 0 {
                f.write_str(" ")?;
            }
            write!(f, "{}", x.as_ofn())?;
        }
        Ok(())
    }
}

/// A trait for OWL elements that can be serialized to OWL Functional syntax.
pub trait AsFunctional {
    /// Get a handle for displaying the object in functional syntax.
    ///
    /// Instead of returning a `String`, this method returns an opaque struct
    /// that implements `Display`, allowing to write it to a file without
    /// building a fully-serialized string first, or to just get a string
    /// with the `ToString` implementation.
    ///
    /// # Example
    /// ```
    /// # use horned_owl::model::DeclareClass;
    /// # let build = horned_owl::model::Build::new();
    /// use horned_functional::AsFunctional;
    ///
    /// let axiom = DeclareClass(build.class("https://example.com/a"));
    /// assert_eq!(
    ///     axiom.as_ofn().to_string(),
    ///     "Declaration(Class(<https://example.com/a>))"
    /// );
    /// ```
    fn as_ofn<'t>(&'t self) -> Functional<'t, Self> {
        Functional(&self)
    }
}

impl<'t, T: 't> AsFunctional for T
where
    Functional<'t, T>: Display
{}

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
        let lit = owl::Literal::Simple { literal: String::from("test") };
        let ofn = format!("{}", lit.as_ofn());
        assert_eq!(r#""test""#, &ofn);

        let lit = owl::Literal::Simple { literal: String::from("test\"") };
        let ofn = format!("{}", lit.as_ofn());
        assert_eq!(r#""test\"""#, &ofn);

        let lit = owl::Literal::Simple { literal: String::from("test\\") };
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
        assert_eq!(r#""hello"^^<http://www.w3.org/2001/XMLSchema#string>"#, &ofn);
    }

    #[test]
    fn test_ofn_import() {
        let build = owl::Build::new();
        let import = owl::Import(build.iri("http://example.com/"));
        let ofn = format!("{}", import.as_ofn());
        assert_eq!("Import(<http://example.com/>)", ofn);
    }
}
