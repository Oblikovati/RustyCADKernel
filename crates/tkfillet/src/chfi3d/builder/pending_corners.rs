// Placeholders for the corner algorithms of ChFi3d_Builder_C1.cxx and ChFi3d_Builder_CnCrn.cxx
// (OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871) whose port is not finished.

//! The corners not yet ported: `PerformMoreSurfdata`, `IntersectMoreCorner` and
//! `PerformMoreThreeCorner` fail explicitly so that a fillet needing them is reported
//! as not done instead of producing a wrong shape.
use super::ChFi3dBuilder;
use rustycad_tkernel::standard::Failure;

/// The failure of a corner whose port is pending.
fn pending(name: &str) -> Failure {
    Failure::with_message(
        "Standard_NotImplemented",
        &format!("ChFi3d_Builder::{name} is not ported yet"),
    )
}

impl ChFi3dBuilder {
    /// `PerformMoreThreeCorner(Index, nconges)`: pending, always fails.
    pub(crate) fn perform_more_three_corner(
        &mut self,
        _index: i32,
        _nconges: i32,
    ) -> Result<(), Failure> {
        Err(pending("PerformMoreThreeCorner"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

    #[test]
    fn the_pending_corners_fail_as_not_implemented() {
        let solid = BRepPrimAPIMakeBox::from_sizes(1., 1., 1.)
            .unwrap()
            .shape()
            .unwrap()
            .clone();
        let mut b = ChFi3dBuilder::new(&solid, 1.0e-2).unwrap();
        let r = b.perform_more_three_corner(1, 3);
        assert_eq!(r.unwrap_err().exception_type(), "Standard_NotImplemented");
    }
}
