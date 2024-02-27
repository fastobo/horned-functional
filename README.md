# `horned-functional` [![Star me](https://img.shields.io/github/stars/fastobo/horned-functional.svg?style=social&label=Star&maxAge=3600)](https://github.com/fastobo/horned-functional/stargazers)

*An [OWL2 Functional-style Syntax](https://www.w3.org/TR/owl2-syntax/) parser and serializer for [`horned-owl`](https://github.com/phillord/horned-owl).*

[![Actions](https://img.shields.io/github/workflow/status/fastobo/horned-functional/Test?style=flat-square&maxAge=600)](https://github.com/fastobo/horned-functional/actions)
[![Codecov](https://img.shields.io/codecov/c/gh/fastobo/horned-functional/master.svg?style=flat-square&maxAge=600)](https://codecov.io/gh/fastobo/horned-functional)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square&maxAge=2678400)](https://choosealicense.com/licenses/mit/)
[![Source](https://img.shields.io/badge/source-GitHub-303030.svg?maxAge=2678400&style=flat-square)](https://github.com/fastobo/horned-functional)
[![Crate](https://img.shields.io/crates/v/horned-functional.svg?maxAge=600&style=flat-square)](https://crates.io/crates/horned-functional)
[![Documentation](https://img.shields.io/badge/docs.rs-latest-4d76ae.svg?maxAge=2678400&style=flat-square)](https://docs.rs/horned-functional)
[![Changelog](https://img.shields.io/badge/keep%20a-changelog-8A0707.svg?maxAge=2678400&style=flat-square)](https://github.com/fastobo/horned-functional/blob/master/CHANGELOG.md)
[![GitHub issues](https://img.shields.io/github/issues/fastobo/horned-functional.svg?style=flat-square)](https://github.com/fastobo/horned-functional/issues)
[![DOI](https://img.shields.io/badge/doi-10.7490%2Ff1000research.1117405.1-brightgreen?style=flat-square&maxAge=31536000)](https://f1000research.com/posters/8-1500)


## 🗺️ Overview

This library provides extensions to the [`horned-owl`](https://crates.io/crates/horned-owl)
crate to work with the [OWL Functional-Style](https://www.w3.org/TR/owl2-syntax) syntax.
It provides a parser written with [`pest`](https://pest.rs) and a zero-copy serializer.

## 🔌 Usage

Add the latest versions of `horned-owl` and `horned-functional` to the
`[dependencies]` sections of your `Cargo.toml` manifest:
```toml
[dependencies]
horned-owl = "0.11.0"
horned-functional = "0.4.0"
```

### 🔍 Parser

To easily read an entire OWL document, including prefixes, use the
[`horned_functional::to_string`](https://docs.rs/horned-functional/latest/horned_functional/fn.from_str.html) function:

```rust
use horned_owl::ontology::set::SetOntology;

let s = std::fs::read_to_string("tests/data/bfo.ofn")
    .expect("failed to read OWL file");
let (ontology, prefixes) = horned_functional::from_str::<_, SetOntology<String>, _>(&s)
    .expect("failed to parse OWL file");
```

All OWL elements can be parsed from functional syntax as well, using the
`FromFunctional` trait to read a from a serialized string with the `from_ofn`
method:

```rust
use horned_owl::model::Axiom;
use horned_functional::FromFunctional;

let axiom = Axiom::<String>::from_ofn("Declaration(Class(<http://purl.obolibrary.org/obo/MS_1000031>))")
    .expect("failed to parse axiom");
```

If the serialized version contains abbreviated IRIs, you can pass a custom
prefix mapping to the `from_ofn_ctx` method:

```rust
use horned_owl::model::Axiom;
use horned_functional::FromFunctional;

let mut mapping = curie::PrefixMapping::default();
mapping.add_prefix("obo", "http://purl.obolibrary.org/obo/").ok();

let ctx = horned_functional::Context::from(&mapping);
let axiom = Axiom::<String>::from_ofn_ctx("Declaration(Class(obo:MS_1000031))", &ctx)
    .expect("failed to parse axiom");
```


### ✏️ Serializer

To easily serialize an entire OWL document, including prefixes, use the
[`horned_functional::to_string`](https://docs.rs/horned-functional/latest/horned_functional/fn.to_string.html) function:

```rust
use std::rc::Rc;
use horned_owl::ontology::axiom_mapped::AxiomMappedOntology;

let mut file = std::fs::File::open("tests/data/ms.owx")
    .map(std::io::BufReader::new)
    .expect("failed to open OWL file");
let cfg = Default::default();
let (ontology, prefixes) = horned_owl::io::owx::reader::read(&mut file, cfg)
    .expect("failed to read OWL file");

// `horned_functional::to_string` needs an AxiomMappedOntology
let axiom_mapped: AxiomMappedOntology<Rc<str>, Rc<_>> = ontology.into();

// serialize using the same prefixes as the input OWL/XML file
let ofn = horned_functional::to_string(&axiom_mapped, &prefixes);

// serialize without abbreviated IRIs
let ofn = horned_functional::to_string(&axiom_mapped, None);
```

All OWL elements can be displayed in functional syntax as well, using
a custom `Display` implementation, allowing the functional syntax in
[`format!`](https://doc.rust-lang.org/std/macro.format.html),
[`println!`](https://doc.rust-lang.org/std/macro.println.html) or
[`write!`](https://doc.rust-lang.org/std/macro.write.html) macros.
Just add the [`AsFunctional`](https://docs.rs/horned-functional/latest/horned_functional/trait.AsFunctional.html) trait to the scope, and use the `as_ofn` method
to get a displayable type for any supported element:

```rust
use horned_owl::model::*;
use horned_functional::AsFunctional;

let build = Build::new_arc();

// build a Declaration(ObjectProperty(<http://purl.obolibrary.org/obo/RO_0002175>))
let op = build.object_property("http://purl.obolibrary.org/obo/RO_0002175");
let axiom = Axiom::from(DeclareObjectProperty(op));

println!("Axiom: {}", axiom.as_ofn());
```

## 💭 Feedback

### ⚠️ Issue Tracker

Found a bug ? Have an enhancement request ? Head over to the
[GitHub issue tracker](https://github.com/fastobo/horned-functional/issues) of the project if
you need to report or ask something. If you are filling in on a bug, please include as much
information as you can about the issue, and try to recreate the same bug in a simple, easily
reproducible situation.

## 📋 Changelog

This project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html)
and provides a [changelog](https://github.com/fastobo/horned-functional/blob/master/CHANGELOG.md)
in the [Keep a Changelog](http://keepachangelog.com/en/1.0.0/) format.

## 📜 License

This library is provided under the open-source
[MIT license](https://choosealicense.com/licenses/mit/).

## 📰 Citation

This project was developed by [Martin Larralde](https://github.com/althonos)
as part of a Master's Degree internship in the [BBOP team](http://berkeleybop.org/) of the
[Lawrence Berkeley National Laboratory](https://www.lbl.gov/), under the supervision of
[Chris Mungall](http://biosciences.lbl.gov/profiles/chris-mungall/). Cite this project as:

*Larralde M.* **Developing Python and Rust libraries to improve the ontology ecosystem**
*\[version 1; not peer reviewed\].* F1000Research 2019, 8(ISCB Comm J):1500 (poster)
([https://doi.org/10.7490/f1000research.1117405.1](https://doi.org/10.7490/f1000research.1117405.1))
