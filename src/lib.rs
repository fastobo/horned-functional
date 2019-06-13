#[macro_use]
extern crate pest_derive;
extern crate pest;
extern crate horned_owl;

use pest::error::Error;
use pest::iterators::Pairs;

/// The OWL2 Functional-style Syntax parser.
#[derive(Debug, Parser)]
#[grammar = "owl.pest"]
pub struct OwlFunctionalParser;




#[cfg(test)]
mod tests {

    use pest::Parser;
    use super::*;

    #[test]
    fn example() {
        let doc = r#"
            Ontology( <http://www.my.example.com/example>
                ClassAssertion( a:Person a:Peter )
            )
        "#;

        // assert!(OwlFunctionalParser::parse(Rule::OntologyDocument, doc).is_ok());
        match OwlFunctionalParser::parse(Rule::OntologyDocument, doc.trim()) {
            Ok(doc) => panic!("{:#?}", doc),
            Err(e) => panic!("{}", e),
        };
    }
}
