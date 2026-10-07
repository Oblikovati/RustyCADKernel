// Rust translation of ShapeFix::RemoveSmallEdges (ShapeFix.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeFix::RemoveSmallEdges`: a ShapeFix_Shape run restricted to FixSmall.
use super::ShapeFix;
use crate::shape_build::ShapeBuildReShape;
use crate::shape_fix::{FixRoot, ShapeFixShape};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};

/// The result of `ShapeFix::RemoveSmallEdges`: the healed shape and the reshape context
/// that records the removed edges (the native `context` out argument).
pub struct ShapeFixRemovedSmallEdges {
    /// The shape returned by native `RemoveSmallEdges`.
    pub shape: Shape,
    /// The ShapeFix_Shape context written to the native `context` argument.
    pub context: Handle<ShapeBuildReShape>,
}

impl ShapeFix {
    /// Removes the edges smaller than `tolerance` with a ShapeFix_Shape whose wire tool
    /// only runs FixSmall (topology mode on; connected, edge-curve, degenerated,
    /// self-intersection and lacking fixes off; missing-seam, orientation and small-area
    /// face fixes off). Example: `let fixed = ShapeFix::remove_small_edges(&shape, 0.1)?;`.
    pub fn remove_small_edges(
        shape: &Shape,
        tolerance: f64,
    ) -> Result<ShapeFixRemovedSmallEdges, Failure> {
        let fixer = Handle::new(ShapeFixShape::empty());
        fixer.init(shape.clone());
        FixRoot::set_precision(&*fixer, tolerance);
        restrict_to_small_edges(&fixer);
        fixer.perform()?;
        Ok(ShapeFixRemovedSmallEdges {
            shape: fixer.shape(),
            context: fixer.context(),
        })
    }
}

fn restrict_to_small_edges(fixer: &ShapeFixShape) {
    let face = fixer.fix_face_tool();
    face.set_fix_missing_seam_mode(0);
    face.set_fix_orientation_mode(0);
    face.set_fix_small_area_wire_mode(0);
    let wire = fixer.fix_wire_tool();
    wire.set_topology_mode(true);
    wire.set_fix_connected_mode(0);
    wire.set_fix_edge_curves_mode(0);
    wire.set_fix_degenerated_mode(0);
    wire.set_fix_self_intersection_mode(0);
    wire.set_fix_lacking_mode(0);
    wire.set_fix_small_mode(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

    #[test]
    fn null_shape_returns_null_result_and_a_fresh_context() {
        let fixed = ShapeFix::remove_small_edges(&Shape::default(), 1.0e-3).unwrap();
        assert!(fixed.shape.is_null());
        assert!(!fixed.context.is_null());
    }

    #[test]
    fn box_without_small_edges_keeps_its_twelve_edges() {
        let mut maker = BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 10.0).unwrap();
        let solid = maker.shape().unwrap().clone();
        let fixed = ShapeFix::remove_small_edges(&solid, 1.0e-3).unwrap();
        assert_eq!(count_edges(&fixed.shape), 12);
    }

    fn count_edges(shape: &Shape) -> usize {
        let mut edges = rustycad_tkbrep::topexp::Explorer::new(
            shape,
            rustycad_tkg3d::top_abs::ShapeEnum::Edge,
            rustycad_tkg3d::top_abs::ShapeEnum::Shape,
        )
        .unwrap();
        let mut unique = Vec::<Shape>::new();
        while edges.more() {
            if !unique.iter().any(|edge| edge.is_same(edges.current())) {
                unique.push(edges.current().clone());
            }
            edges.next().unwrap();
        }
        unique.len()
    }
}
