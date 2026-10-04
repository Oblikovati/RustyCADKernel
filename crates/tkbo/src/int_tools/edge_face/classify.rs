use super::EdgeFace;
use super::touch::check_touch;
use crate::int_tools::Range;
use rustycad_tkernel::{ncollection::Sequence, standard::Failure};
use rustycad_tkg3d::{adaptor3d::Adaptor3dCurve, top_abs::ShapeEnum};

pub(super) fn append_projectable_ranges(
    edge_face: &mut EdgeFace,
    ranges: &Sequence<Range>,
) -> Result<(), Failure> {
    let curve = edge_face.curve.get().expect("initialized edge-face curve");
    let context = edge_face.context.get().expect("edge-face context");
    for range in ranges.iter() {
        let middle = (range.first() + range.last()) * 0.5;
        if !context.is_valid_point_for_face(
            curve.value(middle)?,
            &edge_face.face,
            edge_face.criteria,
        )? {
            continue;
        }
        let mut part = crate::int_tools::CommonPrt::default();
        part.set_edge1(&edge_face.edge);
        part.set_range1(*range);
        edge_face.common_parts.append(part);
    }
    Ok(())
}

pub(super) fn classify_common_parts(edge_face: &mut EdgeFace) -> Result<(), Failure> {
    let curve = edge_face
        .curve
        .get()
        .expect("initialized edge-face curve")
        .shallow_copy()?;
    for index in 1..=edge_face.common_parts.len() {
        classify_common_part(
            edge_face,
            index,
            curve.get().expect("copied edge-face curve"),
        )?;
    }
    edge_face.done = true;
    Ok(())
}

fn classify_common_part(
    edge_face: &mut EdgeFace,
    index: usize,
    curve: &(impl Adaptor3dCurve + ?Sized),
) -> Result<(), Failure> {
    let source_part = edge_face.common_parts.value(index)?;
    let (first, last) = source_part.range1_bounds();
    let all_null = source_part.all_null_flag();
    let first_point = curve.value(first)?;
    let last_point = curve.value(last)?;
    let parameter = common_part_type(first, last, first_point, all_null, curve, edge_face)?;
    let part = edge_face.common_parts.change_value(index)?;
    part.set_bounding_points(first_point, last_point);
    if let Some(vertex_parameter) = parameter {
        part.set_type(ShapeEnum::Vertex);
        part.set_vertex_parameter1(vertex_parameter);
        part.set_range1_bounds(first, last);
    } else {
        part.set_type(ShapeEnum::Edge);
    }
    Ok(())
}

fn common_part_type(
    first: f64,
    last: f64,
    first_point: rustycad_tkmath::gp::Point3,
    all_null: bool,
    curve: &(impl Adaptor3dCurve + ?Sized),
    edge_face: &mut EdgeFace,
) -> Result<Option<f64>, Failure> {
    if all_null {
        return Ok(None);
    }
    let resolution = curve.resolution(edge_face.criteria)?;
    if is_full_range(first, last, edge_face.range, resolution)
        && first_point.distance(&curve.value(last)?) > edge_face.criteria * 2.0
    {
        return Ok(None);
    }
    let middle = (first + last) * 0.5;
    if is_full_range(first, last, edge_face.range, resolution)
        && first_point.distance(&curve.value(middle)?) > edge_face.criteria * 2.0
    {
        return Ok(None);
    }
    Ok(Some(check_touch(edge_face, first, last)?.unwrap_or(middle)))
}

fn is_full_range(first: f64, last: f64, range: Range, resolution: f64) -> bool {
    (first - range.first()).abs() < resolution && (last - range.last()).abs() < resolution
}
