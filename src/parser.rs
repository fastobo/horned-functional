use pest::iterators::Pairs;
use super::error::Result;

/// The OWL2 Functional-style Syntax parser.
///
/// You shouldn't have to use this type directly: instead, use the top level
/// `parse` function to parse an ontology document.
#[derive(Debug, Parser)]
#[grammar = "owl.pest"]
pub struct OwlFunctionalParser;

impl OwlFunctionalParser {
    /// Parse an input string using the given production rule.
    ///
    /// This is basically a specialized version of [`pest::Parser::parse`]
    /// that only accepts [`Rule`], and does not need the `Parser` trait to
    /// be in scope.
    ///
    /// [`Rule`]: ./enum.Rule.html
    /// [`pest::Parser::parse`]: https://docs.rs/pest/latest/pest/trait.Parser.html
    pub fn parse(rule: Rule, input: &str) -> Result<Pairs<Rule>> {
        <Self as pest::Parser<Rule>>::parse(rule, input).map_err(From::from)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    macro_rules! assert_parse {
        ($rule:path, $doc:expr) => {
            let doc = $doc.trim();
            match OwlFunctionalParser::parse($rule, doc) {
                Ok(mut p) => assert_eq!(p.next().unwrap().as_span().end(), doc.len()),
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
        assert_parse!(Rule::Literal, r#""Alan Ruttenberg"@en"#);
    }

    #[test]
    fn typed_literal() {
        assert_parse!(Rule::TypedLiteral, r#""gene_ontology"^^xsd:string"#);
    }

    #[test]
    fn quoted_string() {
        assert_parse!(Rule::QuotedString, r#""gene_ontology""#);
    }

    #[test]
    fn object_intersection_of() {
        assert_parse!(
            Rule::ObjectIntersectionOf,
            r#"ObjectIntersectionOf(<http://purl.obolibrary.org/obo/GO_0008150> ObjectSomeValuesFrom(<http://purl.obolibrary.org/obo/RO_0002211> <http://purl.obolibrary.org/obo/GO_0006310>))"#
        );
    }

    #[test]
    fn object_some_values_from() {
        assert_parse!(
            Rule::ObjectSomeValuesFrom,
            r#"ObjectSomeValuesFrom(<http://purl.obolibrary.org/obo/RO_0002211> <http://purl.obolibrary.org/obo/GO_0006310>)"#
        );
    }

    #[test]
    fn string_literal_with_language() {
        assert_parse!(Rule::StringLiteralWithLanguage, r#""Alan Ruttenberg"@en"#);
    }

}
