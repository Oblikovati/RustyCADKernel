// SPDX-License-Identifier: GPL-2.0-only
//! `extrude`: a non-destructive linear sweep of one certified planar region.
use super::admission::admit_resolution;
use super::decline::{
    DeclineCause, Decliner, OperationDecline, OperationKind, OperationStage as Stage,
};
use super::operation_result::OperationResult;
use super::plane_math::{dot3, norm3};
use super::regions::PlanarRegion;
use super::resolution::{Measure, Resolution};
use super::sweep_history::{IndexedSweep, sweep_history};
use super::validation::*;
use rustycad_tkbrep::topods::{self, Shape, Solid};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkmath::gp::Vector3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

const DECLINE: Decliner = Decliner {
    operation: OperationKind::Extrude,
};

/// Options of one extrusion; the resolution has no implicit default.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtrudeOptions {
    /// Tolerances and cap of the operation.
    pub resolution: Resolution,
}

/// Sweep `region` along the world vector `direction` into a solid.
///
/// The region is not modified: the sweep copies its face. The result carries complete
/// history (both caps generated from the region, one side face per curve, cap edges per
/// curve, lateral edges per vertex), ordered validation and the achieved tolerance; any
/// gap is a typed decline instead of a partial solid.
///
/// ```
/// use rustycad_tkbo::profile_extrusion::*;
/// let profile = SketchProfile { plane: ProfilePlane::world_xy(), vertices: vec![],
///     curves: vec![ProfileCurve::Circle { id: ProfileCurveId(1), center: [0., 0.], radius: 1. }] };
/// let display = DisplayTolerance { chordal: 1e-3, angular: 0.1 };
/// let resolution = Resolution::new(1e-6, 1e-7, 1e-9, display, ToleranceCap::Refuse(1e-6));
/// let set = build_planar_regions(&profile, resolution)?;
/// let result = extrude(&set.regions()[0], [0., 0., 2.], &ExtrudeOptions { resolution })?;
/// assert!(result.volume().encloses(2.0 * std::f64::consts::PI));
/// assert_eq!(result.faces().len(), 3);
/// # Ok::<(), OperationDecline>(())
/// ```
pub fn extrude(
    region: &PlanarRegion,
    direction: [f64; 3],
    options: &ExtrudeOptions,
) -> Result<OperationResult, OperationDecline> {
    let resolution = options.resolution;
    admit_resolution(&resolution).map_err(|c| DECLINE.at(Stage::Admission, c))?;
    let height = admit_direction(region, direction, &resolution)
        .map_err(|c| DECLINE.at(Stage::Admission, c))?;
    let (mut maker, solid) = sweep(region, direction)?;
    let indexed =
        sweep_history(&mut maker, &solid, region).map_err(|c| DECLINE.at(Stage::History, c))?;
    let evidence = SelfIntersectionEvidence::AffineSweepOfCertifiedRegion;
    let mut validation =
        ordered_validation(&solid, evidence).map_err(|c| DECLINE.at(Stage::Validation, c))?;
    let achieved = achieved_tolerance(&solid)
        .map_err(|f| DECLINE.native(Stage::Tolerance, "solid tolerance", f))?;
    let mut findings = Vec::new();
    tolerance_stage(achieved, resolution.cap, &mut validation, &mut findings)
        .map_err(|c| DECLINE.at(Stage::Tolerance, c))?;
    let IndexedSweep {
        faces,
        edges,
        vertices,
        history,
    } = indexed;
    Ok(OperationResult {
        faces: typed(&faces, topods::face)
            .map_err(|f| DECLINE.native(Stage::History, "face index", f))?,
        edges: typed(&edges, topods::edge)
            .map_err(|f| DECLINE.native(Stage::History, "edge index", f))?,
        vertices: typed(&vertices, topods::vertex)
            .map_err(|f| DECLINE.native(Stage::History, "vertex index", f))?,
        solid,
        history,
        validation,
        findings,
        achieved,
        volume: prism_volume(region.area(), height),
        resolution,
    })
}

/// Signed height of `direction` over the region plane; declines a vector shorter than the
/// weld tolerance or within the angular tolerance of the plane.
fn admit_direction(
    region: &PlanarRegion,
    direction: [f64; 3],
    resolution: &Resolution,
) -> Result<f64, DeclineCause> {
    let invalid = |reason| DeclineCause::InvalidDirection { reason, direction };
    let length = norm3(direction);
    if !length.is_finite() {
        return Err(invalid("nonfinite direction"));
    }
    if length < resolution.weld {
        return Err(invalid("direction shorter than the weld tolerance"));
    }
    let height = dot3(direction, region.normal());
    if height.abs() < length * resolution.angular.sin() {
        return Err(invalid(
            "direction within the angular tolerance of the region plane",
        ));
    }
    Ok(height)
}

fn sweep(
    region: &PlanarRegion,
    direction: [f64; 3],
) -> Result<(BRepPrimAPIMakePrism, Solid), OperationDecline> {
    let native = |f| DECLINE.native(Stage::Construction, "linear sweep", f);
    let [x, y, z] = direction;
    // Copy = true: the sweep owns a copy of the face, so the input region stays unchanged.
    let mut maker = BRepPrimAPIMakePrism::new(region.face(), Vector3::new(x, y, z), true, true)
        .map_err(native)?;
    let shape: Shape = maker.shape().map_err(native)?.clone();
    let solid = topods::solid(&shape).map_err(native)?;
    Ok((maker, solid))
}

fn typed<T>(
    map: &IndexedMapOfShape,
    cast: fn(&Shape) -> Result<T, rustycad_tkernel::standard::Failure>,
) -> Result<Vec<T>, rustycad_tkernel::standard::Failure> {
    (1..=map.extent())
        .map(|index| cast(map.find_key(index)?))
        .collect()
}

/// Volume of a prism: base area times height, exact for an affine sweep; the error
/// propagates the area error and one rounding of the product.
fn prism_volume(area: Measure, height: f64) -> Measure {
    let value = area.value * height.abs();
    let error = area.error * height.abs() + 2.0 * f64::EPSILON * value.abs();
    Measure {
        value,
        error,
        kind: area.kind,
    }
}
