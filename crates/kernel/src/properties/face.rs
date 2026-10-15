// SPDX-License-Identifier: GPL-2.0-only
//! Per-face integration: analytic surface, closed trims, Green's theorem per trim.
use super::accumulate::{AREA, AREA_MOMENT, Moments};
use super::boundary::{TrimUse, closes, face_loops, sub_shapes};
use super::measured::{
    FaceProperties, IntegrationMethod, LoopRole, LoopSummary, Measured, MeasuredPoint,
};
use super::native::native_face_area;
use super::planar::planar_field;
use super::revolution::Frame;
use super::surface::face_integrand;
use super::trim_integral::{FaceIntegrand, integrate_trim};
use crate::topology::{BodyTopology, FaceIndex};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Area, centroid, surface kind and loops of the face at `index` of an enumerated body
/// (ADR 0076 indices; loops list their edges by `EdgeIndex`), from its B-rep data.
///
/// Planes, cylinders, cones, spheres and tori trimmed by lines and circles in their
/// parameter plane integrate in closed form; see [`IntegrationMethod`] for the others.
///
/// ```
/// use rustycad_kernel::properties::{face_properties, IntegrationMethod};
/// use rustycad_kernel::topology::{BodyTopology, FaceIndex};
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
///
/// let mut maker = BRepPrimAPIMakeBox::from_sizes(2.0, 3.0, 4.0).unwrap();
/// let topology = BodyTopology::enumerate(maker.shape().unwrap()).unwrap();
/// let properties = face_properties(&topology, FaceIndex(0)).unwrap();
/// assert_eq!(properties.method, IntegrationMethod::ClosedForm);
/// assert!((properties.area.value - 12.0).abs() <= properties.area.error_bound);
/// assert_eq!(properties.loops[0].edges.len(), 4);
/// ```
pub fn face_properties(
    topology: &BodyTopology,
    index: FaceIndex,
) -> Result<FaceProperties, Failure> {
    let face = topology
        .face(index)
        .ok_or_else(|| unknown_face(index, topology))?;
    let (area, centroid, method) = face_measures(&face)?;
    let surface = BRepAdaptorSurface::with_face(&face, false)?;
    Ok(FaceProperties {
        face: index,
        surface: surface.get_type()?,
        orientation: face.orientation(),
        area,
        centroid,
        loops: loop_summaries(&face_loops(&face)?, topology)?,
        method,
    })
}

fn unknown_face(index: FaceIndex, topology: &BodyTopology) -> Failure {
    let count = topology.face_count();
    let message = format!(
        "face index {} is not in the body; expected 0..{count}",
        index.0
    );
    Failure::with_type("Standard_OutOfRange", &message, "")
}

/// Area, area centroid and method of `face`, natively when no Green path exists.
fn face_measures(face: &Face) -> Result<(Measured, MeasuredPoint, IntegrationMethod), Failure> {
    let reference = vertex_mean(face)?;
    let (area, centroid, method) = match face_moments(face, reference)? {
        Some(moments) => (
            moments.measured(AREA, 1.0),
            moments.centroid(reference, (AREA, AREA_MOMENT), 1.0),
            moments.method,
        ),
        None => native_face_area(face)?,
    };
    Ok((area, centroid, method))
}

/// The eight integrals of `face` about `reference`; None when a wire does not close in the
/// parameter plane (a periodic face without seam edges: the native path applies).
/// The flux quantities are those of the forward face; callers apply the orientation.
pub(crate) fn face_moments(face: &Face, reference: [f64; 3]) -> Result<Option<Moments>, Failure> {
    let loops = face_loops(face)?;
    let Some(first) = loops.iter().flatten().next() else {
        return Ok(Some(Moments::default()));
    };
    if !all_close(&loops)? {
        return Ok(None);
    }
    let u_start = first.d1(first.first)?.0[0];
    let surface = BRepAdaptorSurface::with_face(face, false)?;
    let integrand = face_integrand(surface, reference, u_start)?;
    let mut moments = Moments::default();
    for trim in loops.iter().flatten() {
        moments.add(&integrate_trim(&integrand, trim)?, 1.0);
    }
    Ok(Some(moments))
}

fn all_close(loops: &[Vec<TrimUse>]) -> Result<bool, Failure> {
    for uses in loops {
        if !closes(uses)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn loop_summaries(
    loops: &[Vec<TrimUse>],
    topology: &BodyTopology,
) -> Result<Vec<LoopSummary>, Failure> {
    let unit = FaceIntegrand::Planar(planar_field(&parameter_plane()));
    loops
        .iter()
        .map(|uses| loop_summary(&unit, uses, topology))
        .collect()
}

fn loop_summary(
    unit: &FaceIntegrand,
    uses: &[TrimUse],
    topology: &BodyTopology,
) -> Result<LoopSummary, Failure> {
    let mut moments = Moments::default();
    for trim in uses {
        moments.add(&integrate_trim(unit, trim)?, 1.0);
    }
    let parameter_area = moments.measured(AREA, 1.0);
    let role = match parameter_area.value >= 0.0 {
        true => LoopRole::Outer,
        false => LoopRole::Inner,
    };
    let edges = uses
        .iter()
        .filter_map(|trim| topology.edge_index(&trim.edge))
        .collect();
    Ok(LoopSummary {
        edges,
        parameter_area,
        role,
    })
}

/// The `(u, v)` plane itself, so that the planar area integral is the parameter area.
fn parameter_plane() -> Frame {
    Frame {
        offset: [0.0; 3],
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        z: [0.0, 0.0, 1.0],
    }
}

/// The mean of the vertex points of `shape`: a nearby reference that keeps the moment
/// integrals small; `vertex_mean(&solid)?`.
pub(crate) fn vertex_mean(shape: &Shape) -> Result<[f64; 3], Failure> {
    let vertices = sub_shapes(shape, ShapeEnum::Vertex)?;
    let mut sum = [0.0; 3];
    for vertex in &vertices {
        let point = BRepTool::pnt(&topods::vertex(vertex)?)?;
        for (k, coordinate) in point.coordinates().iter().enumerate() {
            sum[k] += coordinate;
        }
    }
    let count = vertices.len().max(1) as f64;
    Ok(sum.map(|s| s / count))
}
