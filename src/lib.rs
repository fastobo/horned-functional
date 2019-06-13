#[macro_use]
extern crate pest_derive;
extern crate pest;
extern crate horned_owl;

use pest::error::Error;

/// The OWL2 Functional-style Syntax parser.
#[derive(Debug, Parser)]
#[grammar = "owl.pest"]
pub struct OwlFunctionalParser;

#[cfg(test)]
mod tests {

    use pest::Parser;
    use super::*;

    macro_rules! assert_parse {
        ($rule:path, $doc:expr) => {
            let doc = $doc.trim();
            match OwlFunctionalParser::parse($rule, doc) {
                Ok(mut p) => assert_eq!(p.next().unwrap().as_span().end(), doc.len()),
                Err(e) => panic!("parsing using {:?}:\n{}\nfailed with: {}", $rule, doc.trim(), e),
            }
        }
    }

    #[test]
    fn ontology_document() {
        assert_parse!(
            Rule::OntologyDocument,
            r#"Ontology( <http://www.my.example.com/example>
                ClassAssertion( a:Person a:Peter )
            )"#
        );
    }

    #[test]
    fn ontology_document_empty() {
        assert_parse!(Rule::OntologyDocument, r#"Ontology ()"#);
    }


    #[test]
    fn literal() {
        assert_parse!(Rule::Literal, r#""gene_ontology"^^xsd:string"#);
    }

    #[test]
    fn typed_literal() {
        assert_parse!(Rule::TypedLiteral, r#""gene_ontology"^^xsd:string"#);
    }

    #[test]
    fn quoted_string() {
        assert_parse!(Rule::QuotedString, r#""gene_ontology""#);
    }

}
