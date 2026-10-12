//! Native per-axis segment collection publishes U before V, keeping each uncompleted axis private on failure.
use super::{StdPrsIsolineDestination, StdPrsIsolineTriangulationInputs, StdPrsSegOnIso};
use super::{
    isoline_segment_sort::sort_isoline_segments,
    isoline_triangle_crossing::{IsolineTriangle, triangle_isoline_segment},
};
use rustycad_tkernel::{
    ncollection::HSequence,
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkmath::{gp::Point3, poly::Triangulation};
pub(super) fn collect_triangle_isolines(
    inputs: &StdPrsIsolineTriangulationInputs<'_>,
    output: &mut StdPrsIsolineDestination<'_>,
) -> Result<(), Failure> {
    for axis in 0..2 {
        let count = inputs.parameters[axis].read().length();
        let mut batches = Vec::new();
        for rank in 1..=count {
            let parameter = *inputs.parameters[axis].read().value(rank as usize)?;
            let segments = collect_isoline_crossings(inputs, axis == 0, parameter)?;
            if !segments.is_empty() {
                batches.push(segments);
            }
        }
        for segments in batches {
            publish_isoline_segments(output, axis == 0, segments);
        }
    }
    Ok(())
}
fn collect_isoline_crossings(
    inputs: &StdPrsIsolineTriangulationInputs<'_>,
    is_u: bool,
    parameter: f64,
) -> Result<Vec<StdPrsSegOnIso>, Failure> {
    let (mut triangles, mut rank) = (Vec::new(), 1);
    while rank <= inputs.mesh.read().nb_triangles() {
        let triangle = read_isoline_triangle(&inputs.mesh.snapshot(), rank)?;
        if let Some(mut segment) =
            triangle_isoline_segment(inputs.surface, is_u, parameter, &triangle)?
        {
            transform_isoline_segment(&mut segment, inputs);
            triangles.push(segment);
        }
        rank += 1;
    }
    Ok(triangles)
}
fn read_isoline_triangle(mesh: &Triangulation, rank: i32) -> Result<IsolineTriangle, Failure> {
    let (a, b, c) = mesh.triangle(rank)?.get();
    // XYZ reads precede every UV read natively, including when both arrays contain checked faults.
    let xyz = [mesh.node(a)?, mesh.node(b)?, mesh.node(c)?];
    let uv = [mesh.uv_node(a)?, mesh.uv_node(b)?, mesh.uv_node(c)?];
    Ok(IsolineTriangle { xyz, uv })
}
fn transform_isoline_segment(
    segment: &mut StdPrsSegOnIso,
    inputs: &StdPrsIsolineTriangulationInputs<'_>,
) {
    let location = inputs.location.snapshot();
    if location.is_identity() {
        return;
    }
    let transform = location.transformation();
    segment.pnts[0].pnt.transform(transform);
    segment.pnts[1].pnt.transform(transform);
}
fn publish_isoline_segments(
    output: &mut StdPrsIsolineDestination<'_>,
    is_u: bool,
    mut segments: Vec<StdPrsSegOnIso>,
) {
    sort_isoline_segments(&mut segments);
    let mut points = new_triangle_isoline(output, is_u);
    let mut previous = 0.;
    for segment in segments {
        if !points.sequence(|seq| seq.is_empty())
            && (segment.pnts[0].param - previous).abs() > precision::p_confusion()
        {
            points = new_triangle_isoline(output, is_u);
        }
        points.append(segment.pnts[0].pnt);
        points.append(segment.pnts[1].pnt);
        previous = segment.pnts[1].param;
    }
}
fn new_triangle_isoline(
    output: &mut StdPrsIsolineDestination<'_>,
    is_u: bool,
) -> Handle<HSequence<Point3>> {
    let points = Handle::new(HSequence::default());
    output.append_handle(is_u, points.clone());
    points
}
