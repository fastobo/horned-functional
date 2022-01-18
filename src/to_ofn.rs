use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Error;

use horned_owl::model as owl;
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
        s = &s[i+1..];
    }
    f.write_str(s)?;
    f.write_str("\"")
}

/// A wrapper for displaying an OWL2 element in functional syntax.
#[derive(Debug)]
pub struct Functional<'t, T: ?Sized + AsFunctional>(&'t T, Option<&'t Context<'t>>);

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
                    $(Functional(&self.0.$field, self.1)),*
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
            Functional(&self.0.ann.ap, self.1),
            Functional(&self.0.subject, self.1),
            Functional(&self.0.ann.av, self.1),
        )
    }
}

impl Display for Functional<'_, owl::AnnotationValue> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use owl::AnnotationValue::*;
        match &self.0 {
            Literal(lit) => Functional(lit, self.1).fmt(f),
            IRI(iri) => Functional(iri, self.1).fmt(f),
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
        macro_rules! deref_impl {
            ($($variant:ident,)*) => {
                match self.0 {
                    $(owl::Axiom::$variant(axiom) => {
                        Functional(axiom, self.1).fmt(f)
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
            Class(exp) => Functional(exp, self.1).fmt(f),
            ObjectIntersectionOf(classes) => {
                write!(f, "ObjectIntersectionOf({})", Functional(classes, self.1))
            },
            ObjectUnionOf(classes) => {
                write!(f, "ObjectUnionOf({})", Functional(classes, self.1))
            },
            ObjectComplementOf(class) => {
                write!(f, "ObjectComplementOf({})", Functional(class.as_ref(), self.1))
            },
            ObjectOneOf(individuals) => {
                write!(f, "ObjectOneOf({})", Functional(individuals, self.1))
            },
            ObjectSomeValuesFrom { ope, bce } => {
                write!(f, "ObjectSomeValuesFrom({} {})", Functional(ope, self.1), Functional(bce.as_ref(), self.1))
            },
            ObjectAllValuesFrom { ope, bce } => {
                write!(f, "ObjectAllValuesFrom({} {})", Functional(ope, self.1), Functional(bce.as_ref(), self.1))
            },
            ObjectHasValue { ope, i } => {
                write!(f, "ObjectHasValue({} {})", Functional(ope, self.1), Functional(i, self.1))
            },
            ObjectHasSelf(ope) => {
                write!(f, "ObjectHasSelf({})", Functional(ope, self.1))
            },
            ObjectMinCardinality { n, ope, bce } => {
                write!(f, "ObjectMinCardinality({} {} {})", n, Functional(ope, self.1), Functional(bce.as_ref(), self.1))
            }
            ObjectMaxCardinality { n, ope, bce } => {
                write!(f, "ObjectMaxCardinality({} {} {})", n, Functional(ope, self.1), Functional(bce.as_ref(), self.1))
            }
            ObjectExactCardinality { n, ope, bce } => {
                write!(f, "ObjectExactCardinality({} {} {})", n, Functional(ope, self.1), Functional(bce.as_ref(), self.1))
            }
            DataSomeValuesFrom { dp, dr } => {
                write!(f, "DataSomeValuesFrom({} {})", Functional(dp, self.1), Functional(dr, self.1))
            }
            DataAllValuesFrom { dp, dr } => {
                write!(f, "DataAllValuesFrom({} {})", Functional(dp, self.1), Functional(dr, self.1))
            }
            DataHasValue { dp, l } => {
                write!(f, "DataHasValue({} {})", Functional(dp, self.1), Functional(l, self.1))
            }
            DataMinCardinality { n, dp, dr } => {
                write!(f, "DataMinCardinality({} {} {})", n, Functional(dp, self.1), Functional(dr, self.1))
            }
            DataMaxCardinality { n, dp, dr } => {
                write!(f, "DataMaxCardinality({} {} {})", n, Functional(dp, self.1), Functional(dr, self.1))
            }
            DataExactCardinality { n, dp, dr } => {
                write!(f, "DataMaxCardinality({} {} {})", n, Functional(dp, self.1), Functional(dr, self.1))
            }
        }
    }
}

impl Display for Functional<'_, owl::DataRange> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use owl::DataRange::*;
        match self.0 {
            Datatype(dt) => Functional(dt, self.1).fmt(f),
            DataIntersectionOf(dts) => {
                write!(f, "DataIntersectionOf({})", Functional(dts, self.1))
            },
            DataUnionOf(dts) => {
                write!(f, "DataUnionOf({})", Functional(dts, self.1))
            },
            DataComplementOf(dt) => {
                write!(f, "DataComplementOf({})", Functional(dt.as_ref(), self.1))
            },
            DataOneOf(lits) => {
                write!(f, "DataUnionOf({})", Functional(lits, self.1))
            },
            DatatypeRestriction(dt, frs) => {
                write!(f, "DatatypeRestriction({} {})", Functional(dt, self.1), Functional(frs, self.1))
            },
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
        write!(f, "{} {}", Functional(&self.0.f, self.1), Functional(&self.0.l, self.1))
    }
}

impl Display for Functional<'_, owl::HasKey> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "HasKey({} ", Functional(&self.0.ce, self.1))?;

        f.write_str("(")?;
        let mut n = 0;
        for pe in self.0.vpe.iter() {
            if let owl::PropertyExpression::ObjectPropertyExpression(ope) = pe {
                if n != 0 {
                    f.write_str(" ")?;
                }
                Functional(ope, self.1).fmt(f)?;
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
                Functional(dp, self.1).fmt(f)?;
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
            Named(i) => Functional(i, self.1).fmt(f),
            Anonymous(i) => Functional(i, self.1).fmt(f),
        }
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
                write!(f, "^^{}", Functional(datatype_iri, self.1))
            }
        }
    }
}

impl Display for Functional<'_, owl::ObjectPropertyExpression> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use owl::ObjectPropertyExpression::*;
        match self.0 {
            ObjectProperty(op) => Functional(op, self.1).fmt(f),
            InverseObjectProperty(op) => {
                write!(f, "ObjectInverseOf({})", Functional(op, self.1))
            }
        }
    }
}

impl Display for Functional<'_, owl::SubObjectPropertyExpression> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use owl::SubObjectPropertyExpression::*;
        match self.0 {
            ObjectPropertyExpression(ope) => Functional(ope, self.1).fmt(f),
            ObjectPropertyChain(chain) => {
                write!(f, "ObjectPropertyChain({})", Functional(chain, self.1))
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

impl<'a, T: 'a> Display for Functional<'a, Vec<T>>
where
    Functional<'a, T>: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for (i, x) in self.0.iter().enumerate() {
            if i != 0 {
                f.write_str(" ")?;
            }
            write!(f, "{}", Functional(x, self.1))?;
        }
        Ok(())
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
        Functional(&self, None)
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
        Functional(&self, Some(context))
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

    #[test]
    fn test_ofn_curie() {
        let build = owl::Build::new();
        let mut prefixes = curie::PrefixMapping::default();
        prefixes.add_prefix("obo", "http://purl.obolibrary.org/obo/").ok();
        let context = Context::from(&prefixes);

        let decl = owl::DeclareClass(build.class("http://purl.obolibrary.org/obo/BFO_0000001"));
        let ofn = format!("{}", decl.as_ofn_ctx(&context));
        assert_eq!("Declaration(Class(obo:BFO_0000001))", ofn);

        let decl = owl::DeclareClass(build.class("http://xmlns.com/foaf/0.1/Person"));
        let ofn = format!("{}", decl.as_ofn_ctx(&context));
        assert_eq!("Declaration(Class(<http://xmlns.com/foaf/0.1/Person>))", ofn);
    }
}
