use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_build::ShapeBuildEdge;
use crate::shape_fix::ShapeFixEdge;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Transform2;

pub(super) fn transform_edge_pcurve(
    edge: &Edge,
    face: &Face,
    transform: (bool, f64, Transform2),
) -> Result<(), Failure> {
    let (needed, u_factor, transform) = transform;
    if needed {
        transform_pcurves(edge, face, u_factor, transform)?;
    }
    repair_edge_curve(edge)
}

fn transform_pcurves(
    edge: &Edge,
    face: &Face,
    u_factor: f64,
    transform: Transform2,
) -> Result<(), Failure> {
    let analyzer = ShapeAnalysisEdge::new();
    let (curve, original_first, original_last) = analyzer.pcurve_on_face(edge, face, false)?;
    if curve.is_null() {
        return Ok(());
    }
    let (curve, first, last) = ShapeBuildEdge.transform_pcurve(
        &curve,
        transform,
        u_factor,
        original_first,
        original_last,
    )?;
    if BRepTool::is_closed_on_face(edge, face)? {
        return transform_seam_pcurve(
            edge,
            face,
            (curve, first, last),
            (original_first, original_last),
        );
    }
    update_pcurve(
        edge,
        face,
        &curve,
        first,
        last,
        (original_first, original_last),
    )
}

fn transform_seam_pcurve(
    edge: &Edge,
    face: &Face,
    first_curve: (Handle<dyn Geom2dCurve>, f64, f64),
    original_range: (f64, f64),
) -> Result<(), Failure> {
    let reversed = topods::edge(&edge.reversed())?;
    let (second_curve, _, _) = ShapeAnalysisEdge::new().pcurve_on_face(&reversed, face, false)?;
    if second_curve.is_null() {
        return update_pcurve(
            edge,
            face,
            &first_curve.0,
            first_curve.1,
            first_curve.2,
            original_range,
        );
    }
    let builder = BRepBuilder::default();
    let curves = if edge.orientation() == Orientation::Forward {
        (&first_curve.0, &second_curve)
    } else {
        (&second_curve, &first_curve.0)
    };
    builder.update_edge_pcurves_on_face(edge, curves, face, 0.0)?;
    builder.range_on_face(edge, face, first_curve.1, first_curve.2)?;
    invalidate_same_range_if_needed(edge, original_range, (first_curve.1, first_curve.2))
}

fn update_pcurve(
    edge: &Edge,
    face: &Face,
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
    original: (f64, f64),
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    builder.update_edge_pcurve_on_face(edge, curve, face, 0.0)?;
    builder.range_on_face(edge, face, first, last)?;
    invalidate_same_range_if_needed(edge, original, (first, last))?;
    Ok(())
}

fn invalidate_same_range_if_needed(
    edge: &Edge,
    original: (f64, f64),
    transformed: (f64, f64),
) -> Result<(), Failure> {
    let changed = original.0 != transformed.0 || original.1 != transformed.1;
    if changed && !BRepTool::degenerated(edge)? {
        BRepBuilder::default().same_range(edge, false)?;
    }
    Ok(())
}

fn repair_edge_curve(edge: &Edge) -> Result<(), Failure> {
    if BRepTool::same_range(edge)? {
        ShapeFixEdge::new().fix_add_curve_3d(edge)?;
        return Ok(());
    }
    repair_curve_on_copy(edge)
}

fn repair_curve_on_copy(edge: &Edge) -> Result<(), Failure> {
    let mut source = edge.clone();
    let manifold = matches!(
        source.orientation(),
        Orientation::Forward | Orientation::Reversed
    );
    if !manifold {
        source.set_orientation(Orientation::Forward);
    }
    let copy = ShapeBuildEdge.copy(&source, false)?;
    ShapeFixEdge::new().fix_add_curve_3d(&copy)?;
    let (curve, first, last) = ShapeAnalysisEdge::new().curve_3d(&copy, false)?;
    if curve.is_null() {
        return Ok(());
    }
    let builder = BRepBuilder::default();
    builder.update_edge_curve(edge, &curve, 0.0)?;
    ShapeBuildEdge.set_range_3d(edge, first, last)
}
