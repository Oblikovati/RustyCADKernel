// SPDX-License-Identifier: GPL-2.0-only
//! Read-only scalar and semantic views; ownership remains with the projected body.
use super::{geometry::*, rows::*};
use crate::sketch_region_prism::{
    owned_request::names::*, surface_charts::PrismTrimRole,
    tolerance_inspection::StoredToleranceObservation,
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::predicates::roots::metric_bounds::{MetricCurve, MetricEmbedding};
impl PrismBodyMember {
    /// Borrow or copy the retained owned operand without live lookup; `row.key()`.
    pub(crate) fn key(&self) -> &SemanticMemberKey {
        &self.key
    }
    /// Borrow or copy the retained owned operand without live lookup; `row.output_slot()`.
    pub(crate) fn output_slot(&self) -> Option<SemanticNameSlot> {
        self.output_slot
    }
    /// Read the rounded compatibility observation, not exact affine transport; `row.rounded_world_point()`.
    pub(crate) fn rounded_world_point(&self) -> Option<[f64; 3]> {
        self.rounded_world_point
    }
    /// Borrow or copy the retained owned operand without live lookup; `row.space_curve()`.
    pub(crate) fn space_curve(&self) -> Option<PrismBodySpaceCurveView<'_>> {
        self.curve.as_ref().map(PrismBodySpaceCurveView::borrow)
    }
    /// Read raw/effective metadata and its cap decision; these are not world errors; `row.tolerance()`.
    pub(crate) fn tolerance(&self) -> Option<StoredToleranceObservation> {
        self.tolerance
    }
}
impl PrismBodyOccurrence {
    /// Borrow or copy the retained owned operand without live lookup; `row.key()`.
    pub(crate) fn key(&self) -> &SemanticUseKey {
        &self.key
    }
    /// Borrow or copy the retained owned operand without live lookup; `row.orientation()`.
    pub(crate) fn orientation(&self) -> Orientation {
        self.orientation
    }
    /// Borrow or copy the retained owned operand without live lookup; `row.placement()`.
    pub(crate) fn placement(&self) -> MetricEmbedding<3> {
        self.placement
    }
    /// Read containment children; cyclic traversal is retained separately; `row.children()`.
    pub(crate) fn children(&self) -> &[SemanticUseKey] {
        &self.children
    }
    /// Read stored local coordinates paired with this occurrence placement; `row.local_vertex()`.
    pub(crate) fn local_vertex(&self) -> Option<[f64; 3]> {
        self.local_vertex
    }
}
impl PrismBodyFace {
    /// Borrow or copy the retained owned operand without live lookup; `row.key()`.
    pub(crate) fn key(&self) -> &SemanticMemberKey {
        &self.key
    }
    /// Borrow or copy the retained owned operand without live lookup; `row.surface()`.
    pub(crate) fn surface(&self) -> PrismBodySurfaceView<'_> {
        PrismBodySurfaceView::borrow(&self.surface)
    }
    /// Borrow or copy the retained owned operand without live lookup; `row.trims()`.
    pub(crate) fn trims(&self) -> &[PrismBodyTrim] {
        &self.trims
    }
}
impl PrismBodyTrim {
    /// Borrow or copy the retained owned operand without live lookup; `row.occurrence()`.
    pub(crate) fn occurrence(&self) -> &SemanticUseKey {
        &self.occurrence
    }
    /// Borrow or copy the retained owned operand without live lookup; `row.role()`.
    pub(crate) fn role(&self) -> PrismTrimRole {
        self.role
    }
    /// Borrow or copy the retained owned operand without live lookup; `row.pcurve()`.
    pub(crate) fn pcurve(&self) -> MetricCurve<'_, 2> {
        self.pcurve.borrow()
    }
    /// Borrow or copy the retained owned operand without live lookup; `row.native_interval()`.
    pub(crate) fn native_interval(&self) -> [f64; 2] {
        self.native_interval
    }
    /// Borrow or copy the retained owned operand without live lookup; `row.space_curve()`.
    pub(crate) fn space_curve(&self) -> PrismBodySpaceCurveView<'_> {
        PrismBodySpaceCurveView::borrow(&self.space)
    }
}
impl PrismBodyEdgeEndpoints {
    /// Borrow or copy the retained owned operand without live lookup; `row.edge()`.
    pub(crate) fn edge(&self) -> &SemanticMemberKey {
        &self.edge
    }
    /// Borrow or copy the retained owned operand without live lookup; `row.first()`.
    pub(crate) fn first(&self) -> &SemanticMemberKey {
        &self.first
    }
    /// Borrow or copy the retained owned operand without live lookup; `row.last()`.
    pub(crate) fn last(&self) -> &SemanticMemberKey {
        &self.last
    }
}
impl PrismBodyCoedge {
    /// Borrow or copy the retained owned operand without live lookup; `row.occurrence()`.
    pub(crate) fn occurrence(&self) -> &SemanticUseKey {
        &self.occurrence
    }
    /// Borrow or copy the retained owned operand without live lookup; `row.face()`.
    pub(crate) fn face(&self) -> &SemanticMemberKey {
        &self.face
    }
    /// Borrow or copy the retained owned operand without live lookup; `row.wire()`.
    pub(crate) fn wire(&self) -> &SemanticMemberKey {
        &self.wire
    }
    /// Borrow or copy the retained owned operand without live lookup; `row.edge()`.
    pub(crate) fn edge(&self) -> &SemanticMemberKey {
        &self.edge
    }
    /// Borrow or copy the retained owned operand without live lookup; `row.reversed()`.
    pub(crate) fn reversed(&self) -> bool {
        self.reversed
    }
}
impl PrismBodyWireTraversal {
    /// Borrow or copy the retained owned operand without live lookup; `row.occurrence()`.
    pub(crate) fn occurrence(&self) -> &SemanticUseKey {
        &self.occurrence
    }
    /// Borrow or copy the retained owned operand without live lookup; `row.coedges()`.
    pub(crate) fn coedges(&self) -> &[SemanticUseKey] {
        &self.coedges
    }
}
