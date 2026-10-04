//! Exhaustive native three-flag state transitions.
use rustycad_tkbo::bop_algo::SectionAttribute;
use std::fmt::Write;

#[test]
fn all_flag_states_and_single_flag_mutations_match_native() {
    let mut rows = format!("default {}\n", flag_mask(SectionAttribute::new()));
    assert_eq!(SectionAttribute::default(), SectionAttribute::new());
    for mask in 0..8 {
        let flags = SectionAttribute::with_flags(mask & 1 != 0, mask & 2 != 0, mask & 4 != 0);
        writeln!(rows, "new {mask} {}", flag_mask(flags)).unwrap();
        append_mutations(mask, flags, &mut rows);
    }
    let expected = include_str!("../../../tools/oracle/tkbo_section_attribute.expected");
    assert_eq!(rows, expected.split("filler-default").next().unwrap());
}

fn flag_mask(flags: SectionAttribute) -> i32 {
    i32::from(flags.approximation())
        | (i32::from(flags.pcurve_on_s1()) << 1)
        | (i32::from(flags.pcurve_on_s2()) << 2)
}

fn append_mutations(mask: i32, initial: SectionAttribute, rows: &mut String) {
    let setters = [
        SectionAttribute::set_approximation,
        SectionAttribute::set_pcurve_on_s1,
        SectionAttribute::set_pcurve_on_s2,
    ];
    for (axis, setter) in setters.into_iter().enumerate() {
        for enabled in [false, true] {
            let mut flags = initial;
            setter(&mut flags, enabled);
            writeln!(
                rows,
                "set {mask} {axis} {} {}",
                i32::from(enabled),
                flag_mask(flags)
            )
            .unwrap();
        }
    }
}
