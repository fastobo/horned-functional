# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).


## [Unreleased]

[Unreleased]: https://github.com/fastobo/horned-functional/compare/v0.3.0...HEAD


## [v0.3.0] - 2022-01-19

[v0.3.0]: https://github.com/fastobo/horned-functional/compare/v0.2.0...v0.3.0

### Added
- `Context` struct to pass optional `horned_owl::model::Build` and `curie::PrefixMapping`
  references to use while parsing and serializing.
- `AsFunctional` trait to render OWL elements in Functional-style syntax
  format.
- `FromFunctional` implementation for `Axiom` based on the `Axiom` implementation
  discarding the annotations.
- Update `FromPair` code to support `AnonymousIndividual` where possible.
- `horned_functional::to_string` function to render an `Ontology`.

### Changed
- Renamed `FromFunctional` methods to `from_ofn` and `from_ofn_ctx`.
- `FromPair` implementors can only be derived from a single `pest` rule.


## [v0.2.0] - 2021-12-12

### Changed
- Bumped `horned-owl` to `v0.10.0` ([#19](https://github.com/fastobo/horned-functional/pull/19) by [@paulalesius](https://github.com/paulalesius)).

[v0.2.0]: https://github.com/fastobo/horned-functional/compare/v0.1.3...v0.2.0


## [v0.1.3] - 2020-11-18

### Changed
- Use `thiserror` instead of `err-derive` to derive error trait.

[v0.1.3]: https://github.com/fastobo/horned-functional/compare/v0.1.2...v0.1.3


## [v0.1.2] - 2020-04-10

### Fixed
- Relaxed `pest` version to avoid compatibility issues with `fastobo`.

[v0.1.1]: https://github.com/fastobo/horned-functional/compare/v0.1.1...v0.1.2


## [v0.1.1] - 2020-04-10

### Changed
- Bumped `err-derive` dependency to `v0.2.0`.
- Bumped `curie` dependency to `v0.1.1`.

[v0.1.1]: https://github.com/fastobo/horned-functional/compare/v0.1.0...v0.1.1


## [v0.1.0] - 2019-07-24

[v0.1.0]: https://github.com/fastobo/horned-functional/compare/0beaa9d...v0.1.0

Initial release.
