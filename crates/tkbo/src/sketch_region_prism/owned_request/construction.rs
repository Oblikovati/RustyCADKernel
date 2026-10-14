// SPDX-License-Identifier: GPL-2.0-only
//! Retain explicit constructor endpoints; coordinate equality never recovers identity.
use super::{names::*, request::OwnedPolynomialBoundary};
use crate::sketch_region::{ArrangementSource, RegionSourceId};
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{
    geom::{GeomBezierCurve, GeomCurve},
    top_abs::Orientation,
};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeVertex};

pub(super) struct ConstructedPolynomialSource {
    pub edge: Shape,
    pub native_edge: Shape,
    pub closure: Vertex,
    pub native_start: Shape,
    pub native_end: Shape,
    pub identity: RegionSourceId,
}
impl ConstructedPolynomialSource {
    pub(super) fn source(&self) -> ArrangementSource<'_> {
        ArrangementSource {
            identity: self.identity,
            shape: &self.edge,
        }
    }
}
pub(super) fn construct_owned_polynomial(
    source: &OwnedPolynomialBoundary,
) -> Result<ConstructedPolynomialSource, Failure> {
    construct_owned_polynomial_as(source, source_identity())
}
/// Construct one owned closed polynomial under an explicit authored source identity.
/// Example: `construct_owned_polynomial_as(&outer, RegionSourceId(outer.source.0))`.
pub(super) fn construct_owned_polynomial_as(
    source: &OwnedPolynomialBoundary,
    identity: RegionSourceId,
) -> Result<ConstructedPolynomialSource, Failure> {
    let curve = polynomial_curve(source)?;
    let first = source
        .world_poles
        .first()
        .ok_or_else(|| Failure::new("polynomial poles count=0; expected at least 2"))?;
    let closure =
        BRepBuilderAPIMakeVertex::new(Point3::new(first[0], first[1], first[2]))?.vertex()?;
    let edge =
        BRepBuilderAPIMakeEdge::from_curve_vertices_params(&curve, &closure, &closure, 0., 1.)?
            .edge()?;
    Ok(retain_constructor_handles(
        edge.into_shape(),
        closure,
        source.reversed_traversal,
        identity,
    ))
}
fn polynomial_curve(source: &OwnedPolynomialBoundary) -> Result<Handle<dyn GeomCurve>, Failure> {
    let points: Vec<_> = source
        .world_poles
        .iter()
        .map(|p| Point3::new(p[0], p[1], p[2]))
        .collect();
    let poles = Array1::from_shared(&points, 1);
    Ok(Handle::new(GeomBezierCurve::new(&poles)?).cast())
}
fn retain_constructor_handles(
    edge: Shape,
    closure: Vertex,
    reversed: bool,
    identity: RegionSourceId,
) -> ConstructedPolynomialSource {
    let orientation = if reversed {
        Orientation::Reversed
    } else {
        Orientation::Forward
    };
    ConstructedPolynomialSource {
        edge: edge.oriented(orientation),
        native_edge: edge,
        native_start: closure.oriented(Orientation::Forward),
        native_end: closure.oriented(Orientation::Reversed),
        closure,
        identity,
    }
}

fn source_identity() -> RegionSourceId {
    let slot = SemanticNameSlot {
        stage: SemanticNameStage::Original,
        role: SemanticNameRole::Boundary,
    };
    RegionSourceId(slot.token().value())
}
