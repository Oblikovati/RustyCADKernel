// SPDX-License-Identifier: GPL-2.0-only
//! Published views borrow private owned values and do not enter compatibility objects.
use super::{encoding::*, types::*};
impl FrozenPrismProjection {
    /// Borrow captured definitions; example: `projection.nodes().len()`.
    pub(crate) fn nodes(&self) -> &[FrozenPrismNode] {
        &self.nodes
    }
    /// Borrow all directed occurrences; example: `projection.uses().len()`.
    pub(crate) fn uses(&self) -> &[FrozenPrismUse] {
        &self.uses
    }
    /// Borrow exact captured face charts; example: `projection.faces().len()`.
    pub(crate) fn faces(&self) -> &[FrozenPrismFace] {
        &self.faces
    }
    /// Borrow typed operation history; example: `projection.history().derived.len()`.
    pub(crate) fn history(&self) -> &FrozenPrismHistory {
        &self.history
    }
    /// Borrow explicit current gaps and historical observations; example: `projection.coverage().current`.
    pub(crate) fn coverage(&self) -> &super::coverage::FrozenPrismCoverage {
        &self.coverage
    }
    /// Borrow actual requested/effective settings; example: `projection.limits().achieved_error`.
    pub(crate) fn limits(&self) -> &FrozenPrismLimits {
        &self.limits
    }
    /// Borrow stable-key graph incidence; example: `projection.incidence().counts`.
    pub(crate) fn incidence(&self) -> &super::graph::FrozenGraphIncidence {
        &self.graph
    }
    /// Borrow zero-equality certificates over frozen operands; example: `projection.endpoint_proofs().len()`.
    pub(crate) fn endpoint_proofs(&self) -> &[super::endpoints::FrozenEndpointProof] {
        &self.endpoints
    }
    /// Borrow fixed capture bounds (excess is refused); example: `projection.capture_policy().maximum_polynomial_poles`.
    pub(crate) fn capture_policy(&self) -> &super::domain::FrozenCapturePolicy {
        &self.capture_policy
    }
    /// No accepted body is produced by this diagnostic Interface; example: `assert!(!projection.is_valid_body())`.
    pub(crate) fn is_valid_body(&self) -> bool {
        false
    }
    /// Stable owned replay bytes, including typed limitations; example: `projection.replay_bytes()`.
    pub(crate) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = format!(
            "version={};operation={:?};region={:?};graph={:?};history={:?};coverage={:?};",
            self.version, self.operation, self.region, self.graph, self.history, self.coverage
        )
        .into_bytes();
        for occurrence in &self.uses {
            placement(occurrence.placement, &mut bytes);
            bytes.extend(
                format!(
                    "{:?};{:?};{:?};",
                    occurrence.key, occurrence.orientation, occurrence.children
                )
                .bytes(),
            );
        }
        bytes.extend(format!("capture_policy={:?};", self.capture_policy).bytes());
        bytes.extend(format!("endpoints={:?};", self.endpoints).bytes());
        bytes.extend(geometry_bytes(&self.nodes, &self.faces));
        limits(&self.limits, &mut bytes);
        bytes
    }
}
pub(super) fn geometry_bytes(nodes: &[FrozenPrismNode], faces: &[FrozenPrismFace]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for node in nodes {
        node_bytes(node, &mut bytes);
    }
    for face in faces {
        face_bytes(face, &mut bytes);
    }
    bytes
}
fn node_bytes(node: &FrozenPrismNode, bytes: &mut Vec<u8>) {
    bytes.extend(format!("node={:?};", node.key).bytes());
    bytes.push(u8::from(node.point.is_some()));
    if let Some(point) = node.point {
        numbers(point, bytes);
    }
    bytes.push(u8::from(node.raw_tolerance.is_some()));
    if let Some(raw) = node.raw_tolerance {
        numbers([raw], bytes);
    }
    bytes.push(u8::from(node.effective_tolerance.is_some()));
    if let Some(tolerance) = node.effective_tolerance {
        numbers([tolerance], bytes);
    }
    bytes.push(u8::from(node.curve.is_some()));
    if let Some(curve) = &node.curve {
        space(curve, bytes);
    }
}
fn face_bytes(face: &FrozenPrismFace, bytes: &mut Vec<u8>) {
    bytes.extend(format!("face={:?};proof={:?};", face.key, face.proof).bytes());
    chart_source(&face.source, bytes);
    surface(&face.surface, bytes);
    for trim in &face.trims {
        bytes.extend(
            format!(
                "trim={:?};source={:?};parents={:?};role={:?};",
                trim.occurrence, trim.source, trim.parents, trim.role
            )
            .bytes(),
        );
        expectation(&trim.expected, bytes);
        curve(&trim.pcurve, bytes);
        numbers(trim.interval, bytes);
        space(&trim.space, bytes);
    }
}
fn limits(value: &FrozenPrismLimits, bytes: &mut Vec<u8>) {
    bytes.extend(
        format!(
            "effective={:?}/{:?}/{:?};",
            value.effective_seed, value.effective_extrusion, value.effective_roots
        )
        .bytes(),
    );
    numbers(value.seed, bytes);
    numbers(value.extrusion, bytes);
    numbers([value.spatial_limit, value.parameter_limit], bytes);
    bytes.extend(
        format!(
            "roots={:?};effective={:?}/{:?};achieved={:?}",
            value.roots, value.effective_spatial, value.effective_parameter, value.achieved_error
        )
        .bytes(),
    );
}

impl std::fmt::Debug for FrozenOriginalOccurrence {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FrozenOriginalOccurrence")
            .field("key", &self.key)
            .field("orientation", &self.orientation)
            .field("linear", &self.placement.linear)
            .field("origin", &self.placement.origin)
            .field("scale", &self.placement.scale)
            .finish()
    }
}

impl std::fmt::Debug for FrozenRetiredIntermediate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FrozenRetiredIntermediate")
            .field("source", &self.source)
            .field("relation", &self.relation)
            .field("orientation", &self.orientation)
            .field("linear", &self.placement.linear)
            .field("origin", &self.placement.origin)
            .field("scale", &self.placement.scale)
            .finish()
    }
}
