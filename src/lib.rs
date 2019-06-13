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
            let doc = $doc;
            if let Err(e) = OwlFunctionalParser::parse($rule, doc.trim()) {
                panic!("parsing using {:?}:\n{}\nfailed with: {}", $rule, doc.trim(), e);
            };
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
        assert_parse!(
            Rule::OntologyDocument,
            r#"Ontology ()"#
        );
    }
}
