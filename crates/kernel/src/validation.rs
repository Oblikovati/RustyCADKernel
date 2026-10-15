// SPDX-License-Identifier: GPL-2.0-only
//! Ordered validation of any owned body (ADR 0072), with findings named by the stable
//! [`crate::topology::BodyTopology`] indices (ADR 0076).
//!
//! [`validate_body`] lives in `rustycad_tkbo::body_check`, which cannot depend on this
//! facade; its [`TopologyRef`] uses the same `TopExp::MapShapes` order, so [`body_entity`]
//! converts a finding's subject or context to a [`FaceIndex`], [`EdgeIndex`] or
//! [`VertexIndex`] without re-exploring the body.
pub use rustycad_tkbo::body_check::*;

use crate::topology::{EdgeIndex, FaceIndex, VertexIndex};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// A validated entity by its stable body index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BodyEntity {
    /// A face, as indexed by `BodyTopology`.
    Face(FaceIndex),
    /// An edge, as indexed by `BodyTopology`.
    Edge(EdgeIndex),
    /// A vertex, as indexed by `BodyTopology`.
    Vertex(VertexIndex),
    /// A wire, shell, solid or compound: its kind and `MapShapes` index.
    Container {
        /// The shape kind.
        kind: ShapeEnum,
        /// Zero-based index among the body's sub-shapes of that kind.
        index: u32,
    },
}

/// The stable entity a finding names; None when the shape is not under the body.
///
/// ```
/// use rustycad_kernel::topology::{BodyTopology, EdgeIndex};
/// use rustycad_kernel::validation::{body_entity, BodyEntity, TopologyRef};
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// let name = TopologyRef { kind: ShapeEnum::Edge, index: Some(3) };
/// assert_eq!(body_entity(name), Some(BodyEntity::Edge(EdgeIndex(3))));
/// ```
pub fn body_entity(name: TopologyRef) -> Option<BodyEntity> {
    let index = name.index?;
    Some(match name.kind {
        ShapeEnum::Face => BodyEntity::Face(FaceIndex(index)),
        ShapeEnum::Edge => BodyEntity::Edge(EdgeIndex(index)),
        ShapeEnum::Vertex => BodyEntity::Vertex(VertexIndex(index)),
        kind => BodyEntity::Container { kind, index },
    })
}
