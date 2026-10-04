//! Pinned native infinite-point classification cases for the Boolean helper.
use rustycad_tkbo::bop_tools::is_inverted_solid;
use rustycad_tkbrep::{brep::BRepBuilder, topods::Solid};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeOneAxis;
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeSphere,
};
fn solid_fixture(kind: usize) -> Solid {
    match kind {
        0 => BRepPrimAPIMakeBox::from_sizes(1., 2., 3.)
            .unwrap()
            .solid()
            .unwrap(),
        1 => BRepPrimAPIMakeCylinder::new(2., 3.).solid().unwrap(),
        2 => BRepPrimAPIMakeSphere::new(2.).unwrap().solid().unwrap(),
        _ => {
            let mut solid = Solid::default();
            BRepBuilder::default().make_solid(&mut solid);
            solid
        }
    }
}
#[test]
fn native_infinite_point_solid_orientation_matches() {
    for line in include_str!("../../../tools/oracle/tkbo_solid_orientation.expected").lines() {
        let row: Vec<usize> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let mut solid = solid_fixture(row[0]);
        solid.set_orientation(
            [
                Orientation::Forward,
                Orientation::Reversed,
                Orientation::Internal,
                Orientation::External,
            ][row[1]],
        );
        assert_eq!(is_inverted_solid(&solid).unwrap(), row[2] != 0, "{line}");
    }
}

#[path = "solid_orientation_cases/state.rs"]
mod state;

#[path = "solid_orientation_cases/shell.rs"]
mod shell;

#[path = "solid_orientation_cases/connexity.rs"]
mod connexity;
