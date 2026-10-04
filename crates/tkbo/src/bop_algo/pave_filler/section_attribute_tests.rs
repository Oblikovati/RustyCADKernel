use super::{BOPAlgoPaveFiller, SectionAttribute};
use std::fmt::Write;

#[test]
fn copied_section_attributes_survive_source_mutation_and_clear() {
    let mut filler = BOPAlgoPaveFiller::new();
    let mut rows = format!("filler-default {}\n", mask(&filler));
    let mut flags = SectionAttribute::with_flags(true, false, true);
    filler.set_section_attribute(&flags);
    flags.set_approximation(false);
    writeln!(rows, "filler-copy {}", mask(&filler)).unwrap();
    filler.clear();
    writeln!(rows, "filler-clear {}", mask(&filler)).unwrap();
    let native = include_str!("../../../../../tools/oracle/tkbo_section_attribute.expected");
    assert_eq!(rows, &native[native.find("filler-default").unwrap()..]);
}

fn mask(filler: &BOPAlgoPaveFiller) -> i32 {
    let flags = filler.section_attribute;
    i32::from(flags.approximation())
        | (i32::from(flags.pcurve_on_s1()) << 1)
        | (i32::from(flags.pcurve_on_s2()) << 2)
}
