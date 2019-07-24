//! Test parser on OWL Functional files converted from OBO with ROBOT.

extern crate horned_functional;

macro_rules! foundrytest {
    ( $(#[$attr:meta])* $name:ident) => (
        #[test]
        $(#[$attr])*
        fn $name() {
            let path = std::path::PathBuf::from(file!())
                .parent()
                .unwrap()
                .join("data")
                .join(stringify!($name))
                .with_extension("obo.ofn");
            let txt = std::fs::read_to_string(&path).unwrap();
            if let Err(e) = horned_functional::from_str(&txt) {
                panic!("could not parse {}: {}", stringify!($name), e);
            }
        }
    );
}

// Small test files.
foundrytest!(aero);
foundrytest!(apo);
foundrytest!(cio);
foundrytest!(hao);
foundrytest!(ms);
foundrytest!(peco);
foundrytest!(plana);
foundrytest!(symp);
foundrytest!(to);

// Failing because of invalid IRIs created by `owltools` conversion.
foundrytest!(#[ignore] ecocore);
foundrytest!(#[ignore] cl);
foundrytest!(#[ignore] ro);

// Too large to load in memory
// foundrytest!(oba);
// foundrytest!(tto);
// foundrytest!(uberon);
// foundrytest!(vto);
// foundrytest!(mondo);
// foundrytest!(gaz);
// foundrytest!(go);
// foundrytest!(chebi);
