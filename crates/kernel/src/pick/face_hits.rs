// SPDX-License-Identifier: GPL-2.0-only
//! The exact intersection of a pick ray with one face (IntCurvesFace_Intersector):
//! closed-form line/surface roots on planes and quadrics, restricted to the face by
//! UV classification against its edges.
use super::hit::PickHit;
use super::ray::PickRay;
use super::{native, refused};
use crate::query_error::{BodyQueryError, BodyQueryFailure, QueryEntity};
use crate::topology::FaceIndex;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::precision;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::int_curves_face::IntCurvesFaceIntersector;

/// The hits of the ray on one face and how they were found.
pub(super) struct FaceIntersection {
    pub hits: Vec<PickHit>,
    /// Plane, cylinder, cone, sphere or torus: intersected in closed form.
    pub analytic: bool,
    /// A planar face whose plane contains the ray.
    pub coplanar: bool,
}

/// Intersects `ray` with `face` (stable index `index`) over distances `[0, inf)`.
pub(super) fn face_intersection(
    face: &Face,
    index: FaceIndex,
    ray: &PickRay,
) -> Result<FaceIntersection, BodyQueryError> {
    let entity = QueryEntity::Face(index);
    let mut intersector = IntCurvesFaceIntersector::new(face, precision::confusion(), true, true)
        .map_err(|failure| native(entity, "face intersector", &failure))?;
    let kind = intersector
        .surface_type()
        .map_err(|failure| native(entity, "surface type", &failure))?;
    intersector
        .perform_line(&ray.line(), 0.0, precision::infinite())
        .map_err(|failure| native(entity, "line intersection", &failure))?;
    require_done(&intersector, entity)?;
    let coplanar = kind == SurfaceType::Plane && intersector.is_parallel();
    Ok(FaceIntersection {
        hits: face_hits(&intersector, index)?,
        analytic: is_analytic(kind),
        coplanar: coplanar && plane_contains_ray(face, ray, entity)?,
    })
}

fn is_analytic(kind: SurfaceType) -> bool {
    matches!(
        kind,
        SurfaceType::Plane
            | SurfaceType::Cylinder
            | SurfaceType::Cone
            | SurfaceType::Sphere
            | SurfaceType::Torus
    )
}

fn require_done(
    intersector: &IntCurvesFaceIntersector,
    entity: QueryEntity,
) -> Result<(), BodyQueryError> {
    if intersector.is_done() {
        return Ok(());
    }
    let cause = "face intersector not done: expected a surface with extension in U and V";
    Err(refused(entity, BodyQueryFailure::Native, cause))
}

fn face_hits(
    intersector: &IntCurvesFaceIntersector,
    index: FaceIndex,
) -> Result<Vec<PickHit>, BodyQueryError> {
    (1..=intersector.nb_pnt())
        .map(|rank| face_hit(intersector, rank, index))
        .collect()
}

fn face_hit(
    intersector: &IntCurvesFaceIntersector,
    rank: i32,
    face: FaceIndex,
) -> Result<PickHit, BodyQueryError> {
    let entity = QueryEntity::Face(face);
    let read = |failure| native(entity, "intersection point", &failure);
    let point = intersector.pnt(rank).map_err(read)?;
    let hit = PickHit {
        face,
        distance: intersector.w_parameter(rank).map_err(read)?,
        uv: [
            intersector.u_parameter(rank).map_err(read)?,
            intersector.v_parameter(rank).map_err(read)?,
        ],
        point: [point.x(), point.y(), point.z()],
        on_boundary: intersector.state(rank).map_err(read)? == State::On,
    };
    finite_hit(hit, entity)
}

fn finite_hit(hit: PickHit, entity: QueryEntity) -> Result<PickHit, BodyQueryError> {
    let values = [hit.distance, hit.uv[0], hit.uv[1]];
    if values
        .iter()
        .chain(&hit.point)
        .all(|value| value.is_finite())
    {
        return Ok(hit);
    }
    let cause = format!("pick hit {hit:?}: expected finite distance, parameters and point");
    Err(refused(entity, BodyQueryFailure::NonFiniteGeometry, cause))
}

fn plane_contains_ray(
    face: &Face,
    ray: &PickRay,
    entity: QueryEntity,
) -> Result<bool, BodyQueryError> {
    let surface = BRepAdaptorSurface::with_face(face, false)
        .map_err(|failure| native(entity, "face surface", &failure))?;
    let plane = surface
        .plane()
        .map_err(|failure| native(entity, "face plane", &failure))?;
    let [x, y, z] = ray.origin();
    Ok(plane.distance(Point3::new(x, y, z)) <= precision::confusion())
}
