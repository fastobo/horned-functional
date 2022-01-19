# `horned-functional` [![Star me](https://img.shields.io/github/stars/fastobo/horned-functional.svg?style=social&label=Star&maxAge=3600)](https://github.com/fastobo/horned-functional/stargazers)

*An [OWL2 Functional-style Syntax](https://www.w3.org/TR/owl2-syntax/) parser for [`horned-owl`](https://github.com/phillord/horned-owl)*

[![Actions](https://img.shields.io/github/workflow/status/fastobo/horned-functional/Test?style=flat-square&maxAge=600)](https://github.com/fastobo/horned-functional/actions)
[![Codecov](https://img.shields.io/codecov/c/gh/fastobo/horned-functional/master.svg?style=flat-square&maxAge=600)](https://codecov.io/gh/fastobo/horned-functional)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square&maxAge=2678400)](https://choosealicense.com/licenses/mit/)
[![Source](https://img.shields.io/badge/source-GitHub-303030.svg?maxAge=2678400&style=flat-square)](https://github.com/fastobo/horned-functional)
[![Crate](https://img.shields.io/crates/v/horned-functional.svg?maxAge=600&style=flat-square)](https://crates.io/crates/horned-functional)
[![Documentation](https://img.shields.io/badge/docs.rs-latest-4d76ae.svg?maxAge=2678400&style=flat-square)](https://docs.rs/horned-functional)
[![Changelog](https://img.shields.io/badge/keep%20a-changelog-8A0707.svg?maxAge=2678400&style=flat-square)](https://github.com/fastobo/horned-functional/blob/master/CHANGELOG.md)
[![GitHub issues](https://img.shields.io/github/issues/fastobo/horned-functional.svg?style=flat-square)](https://github.com/fastobo/horned-functional/issues)
[![DOI](https://img.shields.io/badge/doi-10.7490%2Ff1000research.1117405.1-brightgreen?style=flat-square&maxAge=31536000)](https://f1000research.com/posters/8-1500)


## Overview

This library provides an OWL Functional-style parser implementation for the
`horned-owl` library, which provides the complete OWL2 model as a Rust library.

The parser is implemented as a `pest` parser, using a translation of the BNF
grammar. It provides spanned errors to easily identify the faulty parts of an
invalid OWL2 document.

All OWL2 entities also receive an implementation of `FromFunctional`, which can
be used to deserialize each entity independently from their functional syntax
representation. Since the deserialization is context-dependent when not
considering the entire document, it is possible to provide a custom prefix
mapping to handle compact identifiers in situations where one is needed.


## Usage

Add `horned-owl` and `horned-functional` to the `[dependencies]` sections of
your `Cargo.toml` manifest:
```toml
[dependencies]
horned-functional = "0.2.0"
```

The `from_reader` function is the easiest way to deserialize an OWL Functional
document from a `Read` implementor:
```rust,no_run
extern crate ureq;
extern crate horned_functional;

fn main() {
    let url = "https://raw.githubusercontent.com/ha-mo-we/Racer/master/examples/owl2/owl-primer-mod.ofn";

    let response = ureq::get(url).call().unwrap();
    let mut reader = response.into_reader();

    match horned_functional::from_reader(reader) {
      Ok((ont, _)) => println!("Number of axioms: {}", ont.iter().count()),
      Err(e) => panic!("could not parse document: {}", e)
    };
}
```


## Feedback

Found a bug ? Have an enhancement request ? Head over to the
[GitHub issue tracker](https://github.com/fastobo/horned-functional/issues) of the project if
you need to report or ask something. If you are filling in on a bug, please include as much
information as you can about the issue, and try to recreate the same bug in a simple, easily
reproducible situation.


## About

This project was developed by [Martin Larralde](https://github.com/althonos)
as part of a Master's Degree internship in the [BBOP team](http://berkeleybop.org/) of the
[Lawrence Berkeley National Laboratory](https://www.lbl.gov/), under the supervision of
[Chris Mungall](http://biosciences.lbl.gov/profiles/chris-mungall/). Cite this project as:

*Larralde M.* **Developing Python and Rust libraries to improve the ontology ecosystem**
*\[version 1; not peer reviewed\].* F1000Research 2019, 8(ISCB Comm J):1500 (poster)
([https://doi.org/10.7490/f1000research.1117405.1](https://doi.org/10.7490/f1000research.1117405.1))
