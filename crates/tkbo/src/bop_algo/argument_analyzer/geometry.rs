//! The geometric tests of `BOPAlgo_ArgumentAnalyzer`: C0 curves and surfaces, and edges
//! farther from their faces than their tolerance.
use super::BOPAlgoArgumentAnalyzer;
use super::topology::sub_shapes;
use crate::bop_algo::CheckStatus;
use crate::bop_tools::compute_curve_tolerance;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::Shape as Continuity;

impl BOPAlgoArgumentAnalyzer {
    /// The non-degenerated edges with a C0 curve and the faces with a C0 surface of each
    /// argument, each once (`TestContinuity`).
    pub(super) fn test_continuity(&mut self) -> Result<(), Failure> {
        for index in 0..2 {
            let shape = self.argument(index).clone();
            if shape.is_null() {
                continue;
            }
            let faulty = c0_sub_shapes(&shape)?;
            for position in 1..=faulty.extent() {
                self.add_fault(index, &[faulty.find_key(position)?], CheckStatus::GeomAbsC0);
            }
        }
        Ok(())
    }

    /// The edges whose measured distance to a face exceeds their tolerance, with the face, the
    /// distance and its parameter (`TestCurveOnSurface`).
    pub(super) fn test_curve_on_surface(&mut self) -> Result<(), Failure> {
        for index in 0..2 {
            let shape = self.argument(index).clone();
            if shape.is_null() {
                continue;
            }
            for face in sub_shapes(&shape, ShapeEnum::Face)? {
                for edge in sub_shapes(&face, ShapeEnum::Edge)? {
                    self.check_curve_on_surface(index, &face, &edge)?;
                }
            }
        }
        Ok(())
    }

    fn check_curve_on_surface(
        &mut self,
        index: usize,
        face: &Shape,
        edge: &Shape,
    ) -> Result<(), Failure> {
        let (typed_face, typed_edge) = (topods::face(face)?, topods::edge(edge)?);
        let (mut distance, mut parameter) = (0.0, 0.0);
        if !compute_curve_tolerance(&typed_face, &typed_edge, &mut distance, &mut parameter)? {
            return Ok(());
        }
        if distance <= BRepTool::edge_tolerance(&typed_edge)? {
            return Ok(());
        }
        let result = self.add_fault(index, &[edge, face], CheckStatus::InvalidCurveOnSurface);
        if index == 0 {
            result.set_max_distance1(distance);
            result.set_max_parameter1(parameter);
        } else {
            result.set_max_distance2(distance);
            result.set_max_parameter2(parameter);
        }
        Ok(())
    }
}

/// The non-degenerated edges with a C0 3D curve, then the faces with a C0 surface.
fn c0_sub_shapes(shape: &Shape) -> Result<IndexedMapOfShape, Failure> {
    let mut faulty = IndexedMapOfShape::default();
    for edge in sub_shapes(shape, ShapeEnum::Edge)? {
        let typed = topods::edge(&edge)?;
        if BRepTool::degenerated(&typed)? {
            continue;
        }
        let (curve, _, _) = BRepTool::curve(&typed)?;
        if curve
            .get()
            .ok_or_else(|| null_geometry("curve", &edge))?
            .continuity()
            == Continuity::C0
        {
            faulty.add(edge);
        }
    }
    for face in sub_shapes(shape, ShapeEnum::Face)? {
        let surface = BRepTool::surface(&topods::face(&face)?)?;
        if surface
            .get()
            .ok_or_else(|| null_geometry("surface", &face))?
            .continuity()
            == Continuity::C0
        {
            faulty.add(face);
        }
    }
    Ok(faulty)
}

/// The failure of a sub-shape without geometry, which the native test dereferences.
fn null_geometry(kind: &str, shape: &Shape) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!(
            "BOPAlgo_ArgumentAnalyzer::TestContinuity: {:?} has no {kind}; expected a shape with geometry",
            shape.shape_type()
        ),
        "",
    )
}
