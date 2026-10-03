// Rust translation of the max-angle estimate in BRepOffset_MakeSimpleOffset.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Vector3;

const NORMAL_EPSILON: f64 = 1.0e-12;
const EDGE_SAMPLE_COUNT: usize = 23;

/// Computes the largest sampled angle between normals on non-smooth edge junctions.
///
/// This follows `ComputeMaxAngleOnShape` and `tgtfaces`; box and single-face examples
/// are useful when estimating `GetSafeOffset`.
pub fn compute_simple_offset_max_angle(shape: &Shape) -> Result<f64, Failure> {
    let map = edge_face_ancestors(shape)?;
    let mut maximum: f64 = 0.0;
    for index in 1..=map.extent() {
        measure_map_edge(&map, index, &mut maximum)?;
    }
    Ok(maximum)
}

/// Applies OCCT's safe-distance estimate after computing the shape's max normal angle.
pub fn compute_simple_offset_safe_distance(
    shape: &Shape,
    expected_tolerance: f64,
) -> Result<f64, Failure> {
    if shape.is_null() {
        return Ok(0.0);
    }
    let max_angle = compute_simple_offset_max_angle(shape)?;
    let max_tolerance = BRepTool::max_tolerance(shape, ShapeEnum::Vertex)?;
    let estimate = (expected_tolerance - max_tolerance) / (2.0 * max_angle);
    Ok(native_max(estimate, 0.0))
}

fn native_max(first: f64, second: f64) -> f64 {
    if first < second { second } else { first }
}

fn edge_face_ancestors(shape: &Shape) -> Result<IndexedDataMapOfShapeListOfShape, Failure> {
    let mut map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shape, ShapeEnum::Edge, ShapeEnum::Face, &mut map)?;
    Ok(map)
}

fn measure_map_edge(
    map: &IndexedDataMapOfShapeListOfShape,
    index: usize,
    maximum: &mut f64,
) -> Result<(), Failure> {
    let edge = Edge::unchecked(map.find_key(index)?.clone());
    let ancestors = map.find_from_index(index)?;
    let Some((first, second, is_seam)) = edge_face_pair(&edge, ancestors)? else {
        return Ok(());
    };
    if BRepTool::continuity_on_faces(&edge, &first, &second)? > Continuity::C0 {
        return Ok(());
    }
    let _ = measure_pair_normals(&edge, &first, &second, is_seam, maximum);
    Ok(())
}

fn edge_face_pair(
    edge: &Edge,
    ancestors: &rustycad_tkernel::ncollection::List<Shape>,
) -> Result<Option<(Face, Face, bool)>, Failure> {
    let mut faces = ancestors.iter().map(|shape| Face::unchecked(shape.clone()));
    let Some(first) = faces.next() else {
        return Ok(None);
    };
    if let Some(second) = faces.find(|face| !first.is_same(face)) {
        return Ok(Some((first, second, false)));
    }
    if face_has_opposite_edge_occurrence(edge, &first)? {
        return Ok(Some((first.clone(), first, true)));
    }
    Ok(None)
}

fn face_has_opposite_edge_occurrence(edge: &Edge, face: &Face) -> Result<bool, Failure> {
    let edge_orientation = edge.orientation();
    let face_shape: Shape = face.clone().into();
    let mut explorer = Explorer::new(&face_shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while explorer.more() {
        let current = Edge::unchecked(explorer.current().clone());
        if edge.is_same(&current) && edge_orientation != current.orientation() {
            return Ok(true);
        }
        explorer.next()?;
    }
    Ok(false)
}

fn measure_pair_normals(
    edge: &Edge,
    first: &Face,
    second: &Face,
    is_seam: bool,
    maximum: &mut f64,
) -> Result<(), Failure> {
    if !pcurves_exist_on_ancestors(edge, first, second)? {
        return Ok(());
    }
    let Some((curve_first, curve_second)) = pcurve_adaptors(edge, first, second, is_seam)? else {
        return Ok(());
    };
    let (first_surface, second_surface) = surface_adaptors(first, second, is_seam)?;
    sample_normal_pairs(
        edge,
        first,
        second,
        &curve_first,
        &curve_second,
        &first_surface,
        &second_surface,
        maximum,
    )
}

fn pcurves_exist_on_ancestors(edge: &Edge, first: &Face, second: &Face) -> Result<bool, Failure> {
    if BRepTool::curve_on_face(edge, first)?.pcurve.is_null() {
        return Ok(false);
    }
    Ok(!BRepTool::curve_on_face(edge, second)?.pcurve.is_null())
}

fn pcurve_adaptors(
    edge: &Edge,
    first: &Face,
    second: &Face,
    is_seam: bool,
) -> Result<Option<(BRepAdaptorCurve2d, BRepAdaptorCurve2d)>, Failure> {
    let forward = Edge::unchecked(edge.oriented(Orientation::Forward));
    if BRepTool::curve_on_face(&forward, first)?.pcurve.is_null() {
        return Ok(None);
    }
    let first_curve = BRepAdaptorCurve2d::with_edge_on_face(&forward, first)?;
    let second_edge = if is_seam {
        Edge::unchecked(edge.oriented(Orientation::Reversed))
    } else {
        forward
    };
    if BRepTool::curve_on_face(&second_edge, second)?
        .pcurve
        .is_null()
    {
        return Ok(None);
    }
    let second_curve = BRepAdaptorCurve2d::with_edge_on_face(&second_edge, second)?;
    Ok(Some((first_curve, second_curve)))
}

fn surface_adaptors(
    first: &Face,
    second: &Face,
    is_seam: bool,
) -> Result<(BRepAdaptorSurface, BRepAdaptorSurface), Failure> {
    let first_surface = BRepAdaptorSurface::with_face(first, false)?;
    let second_face = if is_seam { first } else { second };
    let second_surface = BRepAdaptorSurface::with_face(second_face, false)?;
    Ok((first_surface, second_surface))
}

#[allow(clippy::too_many_arguments)]
fn sample_normal_pairs(
    edge: &Edge,
    first: &Face,
    second: &Face,
    curve_first: &BRepAdaptorCurve2d,
    curve_second: &BRepAdaptorCurve2d,
    surface_first: &BRepAdaptorSurface,
    surface_second: &BRepAdaptorSurface,
    maximum: &mut f64,
) -> Result<(), Failure> {
    let (range_start, range_end) = BRepTool::range(edge)?;
    let trim = (range_end - range_start) / 100.0;
    let sample_start = range_start + trim;
    let sample_end = range_end - trim;
    for sample in 0..=EDGE_SAMPLE_COUNT {
        let parameter =
            sample_start + (sample_end - sample_start) * sample as f64 / EDGE_SAMPLE_COUNT as f64;
        sample_edge_normal_pair(
            parameter,
            first,
            second,
            curve_first,
            curve_second,
            surface_first,
            surface_second,
            maximum,
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn sample_edge_normal_pair(
    parameter: f64,
    first: &Face,
    second: &Face,
    curve_first: &BRepAdaptorCurve2d,
    curve_second: &BRepAdaptorCurve2d,
    surface_first: &BRepAdaptorSurface,
    surface_second: &BRepAdaptorSurface,
    maximum: &mut f64,
) -> Result<(), Failure> {
    let Some(normal_first) = sample_face_normal(parameter, first, curve_first, surface_first)?
    else {
        return Ok(());
    };
    let Some(normal_second) = sample_face_normal(parameter, second, curve_second, surface_second)?
    else {
        return Ok(());
    };
    let angle = normal_first.angle(normal_second)?;
    if angle > *maximum {
        *maximum = angle;
    }
    Ok(())
}

fn sample_face_normal(
    parameter: f64,
    face: &Face,
    curve: &BRepAdaptorCurve2d,
    surface: &BRepAdaptorSurface,
) -> Result<Option<Vector3>, Failure> {
    let uv = curve.d0(parameter)?;
    let derivatives = surface.d1(uv.x(), uv.y())?;
    let mut normal = derivatives.d1u.crossed(derivatives.d1v);
    let magnitude = normal.magnitude();
    if !normal_magnitude_is_usable(magnitude) {
        return Ok(None);
    }
    normal /= magnitude;
    if face.orientation() == Orientation::Reversed {
        normal.reverse();
    }
    Ok(Some(normal))
}

fn normal_magnitude_is_usable(magnitude: f64) -> bool {
    magnitude > NORMAL_EPSILON
}

#[cfg(test)]
mod tests {
    use super::normal_magnitude_is_usable;

    #[test]
    fn non_finite_normal_magnitudes_follow_native_greater_than_check() {
        assert!(!normal_magnitude_is_usable(f64::NAN));
        assert!(!normal_magnitude_is_usable(1.0e-12));
        assert!(normal_magnitude_is_usable(f64::INFINITY));
    }
}
