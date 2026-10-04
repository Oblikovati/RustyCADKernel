//! Native ChFiDS concavity discriminants recorded through BRepOffset_Interval.
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
#[test]
fn native_concavity_discriminants_match() {
    let variants = [
        TypeOfConcavity::Concave,
        TypeOfConcavity::Convex,
        TypeOfConcavity::Tangential,
        TypeOfConcavity::FreeBound,
        TypeOfConcavity::Other,
        TypeOfConcavity::Mixed,
    ];
    let expected: Vec<i32> = include_str!("../../../tools/oracle/tkoffset_interval.expected")
        .lines()
        .take(12)
        .step_by(2)
        .map(|line| line.split_whitespace().nth(2).unwrap().parse().unwrap())
        .collect();
    assert_eq!(variants.map(|variant| variant as i32).as_slice(), expected);
}
