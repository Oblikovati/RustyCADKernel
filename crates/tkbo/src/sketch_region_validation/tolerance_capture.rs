// SPDX-License-Identifier: GPL-2.0-only
//! Coefficient snapshots contain no mutable geometry or cached adaptor objects.
use super::tolerance_curves::{CapturedMetricCurve, capture_plane_curve, capture_space_curve};
use super::tolerance_placement::{captured_placement, captured_plane_frame};
use super::tolerance_types::PlanarMetricCause;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Edge, Face},
};
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkmath::predicates::roots::{
    RootIsolationOptions,
    metric_bounds::{MetricEmbedding, PlanarResidualQuery},
};
pub(super) struct CapturedPlanarPair {
    curve: CapturedMetricCurve<3>,
    pcurve: CapturedMetricCurve<2>,
    interval: [f64; 2],
    curve_placement: MetricEmbedding<3>,
    plane_frame: MetricEmbedding<2>,
    plane_placement: MetricEmbedding<3>,
}
impl CapturedPlanarPair {
    pub(super) fn query(
        &self,
        limit: f64,
        policy: RootIsolationOptions,
    ) -> PlanarResidualQuery<'_> {
        PlanarResidualQuery {
            curve: self.curve.borrow(),
            pcurve: self.pcurve.borrow(),
            curve_embedding: self.curve_placement,
            plane_embedding: self.plane_frame,
            plane_placement: self.plane_placement,
            interval: self.interval,
            spatial_limit: limit,
            options: policy,
        }
    }
}
pub(super) fn capture_pair(
    edge: &Edge,
    face: &Face,
) -> Result<CapturedPlanarPair, PlanarMetricCause> {
    let (curve, pcurve, interval, curve_placement) = captured_curves(edge, face)?;
    let (plane_frame, plane_placement) = plane_frame(face)?;
    Ok(CapturedPlanarPair {
        curve,
        pcurve,
        interval,
        curve_placement,
        plane_frame,
        plane_placement,
    })
}
type CapturedEdgeCurves = (
    CapturedMetricCurve<3>,
    CapturedMetricCurve<2>,
    [f64; 2],
    MetricEmbedding<3>,
);
fn captured_curves(edge: &Edge, face: &Face) -> Result<CapturedEdgeCurves, PlanarMetricCause> {
    if !BRepTool::same_parameter(edge)? {
        return Err(PlanarMetricCause::MissingSameParameter);
    }
    let space = BRepTool::curve_with_location(edge)?;
    let plane = BRepTool::curve_on_face(edge, face)?;
    if !plane.is_stored {
        return Err(PlanarMetricCause::MissingStoredPCurve);
    }
    let interval = common_range([space.first, space.last], [plane.first, plane.last])?;
    let curve = capture_space_curve(&space.curve)?;
    let pcurve = capture_plane_curve(&plane.pcurve)?;
    Ok((
        curve,
        pcurve,
        interval,
        captured_placement(space.location.transformation())?,
    ))
}
fn plane_frame(face: &Face) -> Result<(MetricEmbedding<2>, MetricEmbedding<3>), PlanarMetricCause> {
    let (surface, location) = BRepTool::surface_with_location(face)?;
    let planar = surface.cast::<GeomPlane>();
    let Some(planar) = planar.get() else {
        return Err(PlanarMetricCause::Unsupported {
            representation: "surface",
            dynamic_type: surface
                .get()
                .map_or("null", |s| s.dynamic_type().name())
                .to_owned(),
        });
    };
    Ok((
        captured_plane_frame(planar.pln().position()),
        captured_placement(location.transformation())?,
    ))
}
fn common_range(curve: [f64; 2], pcurve: [f64; 2]) -> Result<[f64; 2], PlanarMetricCause> {
    if curve != pcurve {
        return Err(PlanarMetricCause::ParameterRanges { curve, pcurve });
    }
    Ok(curve)
}
