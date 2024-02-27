//! Test parser on OWL Functional files converted from OBO with ROBOT.

extern crate horned_functional;
extern crate horned_owl;

use curie::PrefixMapping;
use horned_owl::ontology::set::SetOntology;

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
                .with_extension("ofn");
            if let Err(e) = horned_functional::from_file::<String, SetOntology<String>, _>(&path) {
                panic!("could not parse {}: {}", stringify!($name), e);
            }
        }
    );
}

foundrytest!(aeo);
foundrytest!(aero);
foundrytest!(amphx);
foundrytest!(apo);
foundrytest!(bco);
foundrytest!(bfo);
foundrytest!(bila);
foundrytest!(bspo);
foundrytest!(cdao);
foundrytest!(ceph);
// foundrytest!(cheminf);  // anonymous annotation target
foundrytest!(chiro);
foundrytest!(cio);
foundrytest!(clao);
foundrytest!(clyh);
foundrytest!(cob);
foundrytest!(core);
foundrytest!(cro);
foundrytest!(cteno);
foundrytest!(cto);
foundrytest!(ddanat);
foundrytest!(ddpheno);
foundrytest!(depictions);
// foundrytest!(dideo);  // anonymous annotation target
foundrytest!(disdriv);
foundrytest!(duo);
foundrytest!(ecao);
foundrytest!(exo);
foundrytest!(fao);
foundrytest!(fbbi);
foundrytest!(fbdv);
foundrytest!(fix);
foundrytest!(gecko);
foundrytest!(genepio);
foundrytest!(geno);
// foundrytest!(geo);  // anonymous annotation target
foundrytest!(hancestro);
foundrytest!(hom);
foundrytest!(hsapdv);
foundrytest!(hso);
foundrytest!(htn);
foundrytest!(iao);
foundrytest!(ido);
foundrytest!(ino);
foundrytest!(kisao);
foundrytest!(labo);
foundrytest!(lepao);
foundrytest!(mamo);
foundrytest!(mcro);
foundrytest!(mf);
foundrytest!(mfmo);
foundrytest!(mfoem);
foundrytest!(mfomd);
foundrytest!(mmo);
foundrytest!(mmusdv);
foundrytest!(mpath);
foundrytest!(ncro);
foundrytest!(nomen);
foundrytest!(oarcs);
foundrytest!(obi_core);
foundrytest!(ogms);
foundrytest!(ogsf);
// foundrytest!(omiabis); // broken language tag
foundrytest!(omo);
// foundrytest!(omrse);  // anonymous annotation target
foundrytest!(one);
foundrytest!(ontoavida);
foundrytest!(ontoneo);
foundrytest!(oostt);
foundrytest!(opl);
foundrytest!(ornaseq);
foundrytest!(pco);
foundrytest!(pdro);
foundrytest!(peco);
foundrytest!(poro);
foundrytest!(ppo);
foundrytest!(proco);
foundrytest!(psdo);
foundrytest!(rex);
foundrytest!(ro);
foundrytest!(rxno);
foundrytest!(sbo);
// foundrytest!(sepio); // anonymous annotation target
foundrytest!(sibo);
foundrytest!(spd);
foundrytest!(symp);
foundrytest!(t4fs);
foundrytest!(tads);
foundrytest!(taxrank);
foundrytest!(trans);
foundrytest!(uo);
foundrytest!(vsao);
foundrytest!(wbls);
foundrytest!(xlmod);
foundrytest!(zeco);
foundrytest!(zfs);
