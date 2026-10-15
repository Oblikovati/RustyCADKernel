// SPDX-License-Identifier: GPL-2.0-only
//! Exact ray picking on a body (ADR 0078): hits on the exact B-rep faces, keyed by
//! stable face index, with surface parameters, ordered by distance. The body is
//! only read.
mod face_hits;
mod hit;
mod ray;

pub use hit::{PickCompleteness, PickHit, PickResult};
pub use ray::PickRay;

use crate::query_error::{BodyQuery, BodyQueryError, BodyQueryFailure, QueryEntity};
use crate::topology::{FaceIndex, enumerate_for};
use face_hits::{FaceIntersection, face_intersection};
use rustycad_tkbrep::topods::Shape;

/// Intersects `ray` with every face of `body` and returns the hits nearest first.
///
/// Each face is intersected on its exact surface (`IntCurvesFace_Intersector`) and
/// restricted to the face by its edges. Hits on a shared edge or vertex are
/// reported once per face, ties ordered by face index. A ray lying in a planar
/// face is reported in `coplanar_faces` instead of as hits.
///
/// ```
/// use rustycad_kernel::pick::{PickCompleteness, PickRay, ray_pick};
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let body = BRepPrimAPIMakeBox::from_sizes(1.0, 1.0, 1.0)?.shape()?.clone();
/// let ray = PickRay::new([0.25, 0.5, 3.0], [0.0, 0.0, -1.0])?;
/// let result = ray_pick(&body, &ray)?;
/// let distances: Vec<f64> = result.hits.iter().map(|hit| hit.distance).collect();
/// assert_eq!(distances, [2.0, 3.0]);
/// assert_eq!(result.completeness, PickCompleteness::Analytic);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn ray_pick(body: &Shape, ray: &PickRay) -> Result<PickResult, BodyQueryError> {
    let topology = enumerate_for(body, BodyQuery::RayPick)?;
    let mut result = PickResult {
        hits: Vec::new(),
        coplanar_faces: Vec::new(),
        completeness: PickCompleteness::Analytic,
    };
    for position in 0..topology.face_count() as u32 {
        let index = FaceIndex(position);
        let face = topology.face(index).expect("index below the count");
        absorb(&mut result, face_intersection(&face, index, ray)?, index);
    }
    result.hits.sort_by(PickHit::pick_order);
    Ok(result)
}

fn absorb(result: &mut PickResult, found: FaceIntersection, index: FaceIndex) {
    result.hits.extend(found.hits);
    if found.coplanar {
        result.coplanar_faces.push(index);
    }
    if !found.analytic {
        result.completeness = PickCompleteness::SampledCandidates;
    }
}

/// A native failure of the pick query.
fn native(entity: QueryEntity, context: &str, failure: &impl std::fmt::Display) -> BodyQueryError {
    BodyQueryError::native(BodyQuery::RayPick, entity, context, failure)
}

/// A pick refusal of `class` about `entity`.
fn refused(
    entity: QueryEntity,
    class: BodyQueryFailure,
    cause: impl Into<String>,
) -> BodyQueryError {
    BodyQueryError::new(BodyQuery::RayPick, entity, class, cause)
}
