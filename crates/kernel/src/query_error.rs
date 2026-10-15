// SPDX-License-Identifier: GPL-2.0-only
//! Typed failures of the read-only body queries: topology enumeration,
//! presentation tessellation and ray picking (ADR 0076).
use crate::topology::{EdgeIndex, FaceIndex};
use std::fmt;

/// The read-only body query that failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BodyQuery {
    /// [`crate::topology::BodyTopology::enumerate`].
    Enumerate,
    /// [`crate::presentation::tessellate`].
    Tessellate,
    /// [`crate::pick::ray_pick`].
    RayPick,
}

/// The entity of the body a failure is about, by its stable index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum QueryEntity {
    /// The whole body or the query request itself.
    Body,
    /// One face by its stable face index.
    Face(FaceIndex),
    /// One edge by its stable edge index.
    Edge(EdgeIndex),
}

/// Why a body query failed; each class is a refusal, never a partial result.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BodyQueryFailure {
    /// The body references no topology.
    NullBody,
    /// Display options outside their documented domain.
    InvalidOptions,
    /// A ray with a non-finite origin or a zero or non-finite direction.
    InvalidRay,
    /// A native kernel call raised a failure; the cause keeps its text.
    Native,
    /// The detached copy did not map an original entity to exactly one image.
    IncompleteCopyHistory,
    /// The mesher finished with defect status flags or did not complete.
    IncompleteMesh,
    /// A face of the body received no triangulation.
    MissingFaceMesh,
    /// A non-degenerated edge received no polygon on any adjacent face mesh.
    MissingEdgePolygon,
    /// A face whose orientation in the body is neither FORWARD nor REVERSED, so
    /// its material side is undefined.
    UnsupportedOrientation,
    /// A mesh node or a hit evaluated to a non-finite coordinate.
    NonFiniteGeometry,
}

/// A typed body-query failure: operation, entity, failure class and cause.
///
/// ```
/// use rustycad_kernel::query_error::{BodyQuery, BodyQueryError, BodyQueryFailure, QueryEntity};
/// let error = BodyQueryError::new(
///     BodyQuery::Tessellate,
///     QueryEntity::Body,
///     BodyQueryFailure::NullBody,
///     "body is null: expected a shape with topology",
/// );
/// assert_eq!(error.class, BodyQueryFailure::NullBody);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyQueryError {
    /// The query that failed.
    pub operation: BodyQuery,
    /// The entity the failure is about.
    pub entity: QueryEntity,
    /// The failure class.
    pub class: BodyQueryFailure,
    /// The offending value, the expected shape and any native cause.
    pub cause: String,
}

impl BodyQueryError {
    /// A failure from its four parts; see the type example.
    pub fn new(
        operation: BodyQuery,
        entity: QueryEntity,
        class: BodyQueryFailure,
        cause: impl Into<String>,
    ) -> Self {
        Self {
            operation,
            entity,
            class,
            cause: cause.into(),
        }
    }

    /// A [`BodyQueryFailure::Native`] failure keeping the native text after `context`;
    /// `BodyQueryError::native(BodyQuery::RayPick, QueryEntity::Body, "load", &failure)`.
    pub fn native(
        operation: BodyQuery,
        entity: QueryEntity,
        context: &str,
        failure: &impl fmt::Display,
    ) -> Self {
        let cause = format!("{context}: {failure}");
        Self::new(operation, entity, BodyQueryFailure::Native, cause)
    }
}

impl fmt::Display for BodyQueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:?} {:?} {:?}: {}",
            self.operation, self.entity, self.class, self.cause
        )
    }
}

impl std::error::Error for BodyQueryError {}
