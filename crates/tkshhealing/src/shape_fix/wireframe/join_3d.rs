use crate::shape_construct::ShapeConstructCurveJoin2dRequest;
use crate::{
    shape_analysis::ShapeAnalysisEdge,
    shape_construct::{ShapeConstructCurve, ShapeConstructCurveJoin3d},
};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Builder, Edge, Face},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkg3d::top_abs::Orientation;

#[path = "join_3d_seam.rs"]
mod seam;

pub(super) struct JoinedEdge3d {
    pub edge: Edge,
    pub replace_first: bool,
}

type JoinedCurve3d = (Handle<GeomBSplineCurve>, (f64, f64), bool, bool, bool);
type JoinedCurve2d = (Handle<Geom2dBSplineCurve>, (f64, f64), bool);

pub(super) fn join_edges_3d(
    first: &Edge,
    second: &Edge,
    faces: &[Face],
) -> Result<Option<JoinedEdge3d>, Failure> {
    let Some((curve, range, mut replace_first, reverse_first, reverse_second)) =
        joined_curve(first, second)?
    else {
        return Ok(None);
    };
    if !compatible_orientation_classes(first, second) {
        return Ok(None);
    }
    let mut edge = make_joined_edge(first, second, &curve, range)?;
    if !append_face_pcurves(
        &mut edge,
        first,
        second,
        faces,
        (reverse_first, reverse_second),
        &mut replace_first,
    )? {
        return Ok(None);
    }
    preserve_nonmanifold_orientation(&mut edge, first, second, replace_first);
    Ok(Some(JoinedEdge3d {
        edge,
        replace_first,
    }))
}

fn append_face_pcurves(
    joined: &mut Edge,
    first: &Edge,
    second: &Edge,
    faces: &[Face],
    reversed_3d: (bool, bool),
    replace_first: &mut bool,
) -> Result<bool, Failure> {
    for face in faces {
        let analysis = ShapeAnalysisEdge::new();
        let first_seam = analysis.is_seam_on_face(first, face)?;
        let second_seam = analysis.is_seam_on_face(second, face)?;
        if first_seam != second_seam {
            return Ok(false);
        }
        if first_seam {
            if !seam::append_seam_pcurves(joined, first, second, face, reversed_3d, replace_first)?
            {
                return Ok(false);
            }
            continue;
        }
        if !append_face_pcurve(joined, first, second, face, reversed_3d, replace_first)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn append_face_pcurve(
    joined: &Edge,
    first: &Edge,
    second: &Edge,
    face: &Face,
    reversed_3d: (bool, bool),
    replace_first: &mut bool,
) -> Result<bool, Failure> {
    let Some((curve, range, reversed_first)) = joined_pcurve(first, second, face, reversed_3d)?
    else {
        return Ok(false);
    };
    if reversed_first {
        *replace_first = false;
    }
    install_face_pcurve(joined, face, &curve, range)?;
    Ok(true)
}

fn joined_pcurve(
    first: &Edge,
    second: &Edge,
    face: &Face,
    reversed_3d: (bool, bool),
) -> Result<Option<JoinedCurve2d>, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let first_curve = analysis.pcurve_on_face(first, face, false)?;
    let second_curve = analysis.pcurve_on_face(second, face, false)?;
    if first_curve.0.is_null() || second_curve.0.is_null() {
        return Ok(None);
    }
    let mut request = ShapeConstructCurveJoin2dRequest {
        first: first_curve.0,
        second: second_curve.0,
        first_orientation: effective_orientation(first, reversed_3d.0),
        second_orientation: effective_orientation(second, reversed_3d.1),
        first_range: (first_curve.1, first_curve.2),
        second_range: (second_curve.1, second_curve.2),
        is_error: joined_same_vertex(first, second),
    };
    let result = ShapeConstructCurve.join_curves_2d(&mut request)?;
    let Some(result) = result else {
        return Ok(None);
    };
    Ok(Some((
        result.curve,
        composed_pcurve_range(request.first_range, request.second_range),
        result.reversed_first,
    )))
}

pub(super) fn effective_orientation(edge: &Edge, reversed_3d: bool) -> Orientation {
    if !is_manifold_orientation(edge.orientation()) {
        return edge.orientation();
    }
    if (edge.orientation() == Orientation::Reversed) != reversed_3d {
        return Orientation::Reversed;
    }
    Orientation::Forward
}

pub(super) fn is_manifold_orientation(orientation: Orientation) -> bool {
    matches!(orientation, Orientation::Forward | Orientation::Reversed)
}

fn compatible_orientation_classes(first: &Edge, second: &Edge) -> bool {
    is_manifold_orientation(first.orientation()) == is_manifold_orientation(second.orientation())
}

fn preserve_nonmanifold_orientation(
    joined: &mut Edge,
    first: &Edge,
    second: &Edge,
    replace_first: bool,
) {
    if is_manifold_orientation(first.orientation()) {
        return;
    }
    let orientation = if replace_first {
        first.orientation()
    } else {
        second.orientation()
    };
    joined.set_orientation(orientation);
}

fn joined_same_vertex(first: &Edge, second: &Edge) -> bool {
    let analysis = ShapeAnalysisEdge::new();
    let first_vertices = (analysis.first_vertex(first), analysis.last_vertex(first));
    let second_vertices = (analysis.first_vertex(second), analysis.last_vertex(second));
    first_vertices.0.is_same(&first_vertices.1) || second_vertices.1.is_same(&second_vertices.0)
}

fn composed_pcurve_range(first: (f64, f64), second: (f64, f64)) -> (f64, f64) {
    (first.0, first.1 + second.1 - second.0)
}

fn install_face_pcurve(
    edge: &Edge,
    face: &Face,
    curve: &Handle<Geom2dBSplineCurve>,
    range: (f64, f64),
) -> Result<(), Failure> {
    let curve: Handle<dyn Geom2dCurve> = curve.clone().cast();
    let builder = BRepBuilder::default();
    builder.update_edge_pcurve_on_face(edge, &curve, face, 0.)?;
    builder.range_on_face(edge, face, range.0, range.1)?;
    builder.same_range(edge, false)
}

fn joined_curve(first: &Edge, second: &Edge) -> Result<Option<JoinedCurve3d>, Failure> {
    let Some((first_curve, second_curve, mut first_range, mut second_range)) =
        input_curves_3d(first, second)?
    else {
        return Ok(None);
    };
    let result = ShapeConstructCurve.join_curves_3d(
        &first_curve,
        &second_curve,
        first.orientation(),
        second.orientation(),
        &mut first_range,
        &mut second_range,
    )?;
    Ok(result.map(|result| joined_curve_data(result, first_range, second_range)))
}

type InputCurves3d = (
    Handle<dyn GeomCurve>,
    Handle<dyn GeomCurve>,
    (f64, f64),
    (f64, f64),
);

fn input_curves_3d(first: &Edge, second: &Edge) -> Result<Option<InputCurves3d>, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let first_curve = analysis.curve_3d(first, false)?;
    let second_curve = analysis.curve_3d(second, false)?;
    if first_curve.0.is_null() || second_curve.0.is_null() {
        return Ok(None);
    }
    Ok(Some((
        first_curve.0,
        second_curve.0,
        (first_curve.1, first_curve.2),
        (second_curve.1, second_curve.2),
    )))
}

fn joined_curve_data(
    result: ShapeConstructCurveJoin3d,
    first_range: (f64, f64),
    second_range: (f64, f64),
) -> JoinedCurve3d {
    let range = composed_pcurve_range(first_range, second_range);
    (
        result.curve,
        range,
        !result.reversed_first,
        result.reversed_first,
        result.reversed_second,
    )
}

fn make_joined_edge(
    first: &Edge,
    second: &Edge,
    curve: &Handle<GeomBSplineCurve>,
    range: (f64, f64),
) -> Result<Edge, Failure> {
    let curve: Handle<dyn GeomCurve> = curve.clone().cast();
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge_with_curve(&mut edge, &curve, tolerance(first, second)?)?;
    let bounded_range = (
        range.0.max(curve.first_parameter()),
        range.1.min(curve.last_parameter()),
    );
    builder.range(&edge, bounded_range.0, bounded_range.1, true)?;
    add_joined_endpoints(&mut edge, first, second)?;
    edge.set_orientation(Orientation::Forward);
    builder.same_range(&edge, false)?;
    Ok(edge)
}

fn tolerance(first: &Edge, second: &Edge) -> Result<f64, Failure> {
    Ok(BRepTool::edge_tolerance(first)?.max(BRepTool::edge_tolerance(second)?))
}

fn add_joined_endpoints(edge: &mut Edge, first: &Edge, second: &Edge) -> Result<(), Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let endpoints = (analysis.first_vertex(first), analysis.last_vertex(second));
    let mut shape = edge.clone().into_shape();
    Builder.add(&mut shape, &endpoints.0.oriented(Orientation::Forward))?;
    Builder.add(&mut shape, &endpoints.1.oriented(Orientation::Reversed))?;
    *edge = rustycad_tkbrep::topods::edge(&shape)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{compatible_orientation_classes, preserve_nonmanifold_orientation};
    use rustycad_tkbrep::topods::Edge;
    use rustycad_tkg3d::top_abs::Orientation;

    #[test]
    fn join_accepts_matching_manifold_and_nonmanifold_classes() {
        assert!(compatible_pair(Orientation::Forward, Orientation::Reversed));
        assert!(compatible_pair(
            Orientation::Internal,
            Orientation::External
        ));
        assert!(!compatible_pair(
            Orientation::Forward,
            Orientation::Internal
        ));
    }

    #[test]
    fn nonmanifold_join_keeps_native_replacement_orientation() {
        let first = oriented_edge(Orientation::Internal);
        let second = oriented_edge(Orientation::External);
        let mut joined = Edge::default();
        preserve_nonmanifold_orientation(&mut joined, &first, &second, false);
        assert_eq!(joined.orientation(), Orientation::External);
    }

    fn compatible_pair(first: Orientation, second: Orientation) -> bool {
        compatible_orientation_classes(&oriented_edge(first), &oriented_edge(second))
    }

    fn oriented_edge(orientation: Orientation) -> Edge {
        let mut edge = Edge::default();
        edge.set_orientation(orientation);
        edge
    }
}
