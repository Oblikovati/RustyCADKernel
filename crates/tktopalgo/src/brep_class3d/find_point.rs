// Rust translation of the FindAPointInTheFace members of BRepClass3d_SolidExplorer.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The static `FindAPointInTheFace` overloads of the solid explorer: a point inside a
//! face, a little inside the face from a point of one of its edges.
use super::BRepClass3dSolidExplorer;
use crate::brep_class::{BRepClassEdge, FacePassiveClassifier};
use crate::brep_top_adaptor::BRepTopAdaptorFClass2d;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_epsilon;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::{self, Direction2, Line2, Point2, Point3, Vector2, Vector3};

/// The in/out values of `FindAPointInTheFace` and `PointInTheFace`: the point, its
/// surface parameters and the first derivatives there. The members write them as the
/// native references are written (some on the way even when no point is found), and
/// leave the others as the caller set them.
#[derive(Clone, Copy, Debug, Default)]
pub struct FacePointSample {
    /// `P`: the point.
    pub point: Point3,
    /// `u`: its U parameter.
    pub u: f64,
    /// `v`: its V parameter.
    pub v: f64,
    /// `theVecD1U`: the first derivative in U.
    pub d1u: Vector3,
    /// `theVecD1V`: the first derivative in V.
    pub d1v: Vector3,
}

/// `TolInit`: how far inside the face the search starts from the edge point.
const TOL_INIT: f64 = 0.00001;

/// The start of the search from one edge: the point a little inside the face from
/// the edge point at `param` of its range, and the unit inner normal there.
struct EdgeStart {
    origin: Point2,
    normal: Vector2,
}

impl BRepClass3dSolidExplorer {
    /// `FindAPointInTheFace(F, P, Param)`: a point `P` in the face; `param` in ]0, 1[
    /// picks the edge point the search starts from;
    /// `BRepClass3dSolidExplorer::find_a_point_in_the_face(&face, &mut p, &mut 0.5)?`.
    pub fn find_a_point_in_the_face(
        face: &Face,
        point: &mut Point3,
        param: &mut f64,
    ) -> Result<bool, Failure> {
        let (mut u, mut v) = (0.0, 0.0);
        Self::find_a_point_in_the_face_uv_param(face, point, &mut u, &mut v, param)
    }

    /// `FindAPointInTheFace(F, P, u, v, Param)`: the point and its parameters;
    /// `BRepClass3dSolidExplorer::find_a_point_in_the_face_uv_param(&face, &mut p, &mut u, &mut v, &mut 0.5)?`.
    pub fn find_a_point_in_the_face_uv_param(
        face: &Face,
        point: &mut Point3,
        u: &mut f64,
        v: &mut f64,
        param: &mut f64,
    ) -> Result<bool, Failure> {
        let mut sample = FacePointSample {
            point: *point,
            u: *u,
            v: *v,
            ..FacePointSample::default()
        };
        let found = Self::find_a_point_in_the_face_d1(face, &mut sample, param)?;
        (*point, *u, *v) = (sample.point, sample.u, sample.v);
        Ok(found)
    }

    /// `FindAPointInTheFace(F, P, u, v, Param, theVecD1U, theVecD1V)`: for each edge
    /// with a pcurve on the FORWARD face, starts 1e-5 inside the face from the edge
    /// point at `param` of its range, along the inner normal, up to the nearest other
    /// edge; then tries 0.41234 of that distance, repeatedly, until the point is IN the
    /// face (else false) with non-parallel derivatives (true), or the distance is
    /// below Precision::PConfusion() (false). Without an edge that crosses the ray, the
    /// next edge is tried; `BRepClass3dSolidExplorer::find_a_point_in_the_face_d1(&face, &mut s, &mut 0.5)?`.
    pub fn find_a_point_in_the_face_d1(
        face_in: &Face,
        sample: &mut FacePointSample,
        param: &mut f64,
    ) -> Result<bool, Failure> {
        let face = topods::face(&face_in.oriented(Orientation::Forward))?;
        for edge in Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edge = topods::edge(&edge)?;
            let Some(start) = edge_start(&edge, &face, *param)? else {
                continue;
            };
            let Some(distance) = nearest_crossing(&face, &edge, &start)? else {
                continue;
            };
            return shrink_to_inner_point(&face, &start, distance, sample);
        }
        Ok(false)
    }

    /// `FindAPointInTheFace(F, P, u, v)`: `Param` 0.1;
    /// `BRepClass3dSolidExplorer::find_a_point_in_the_face_uv(&face, &mut p, &mut u, &mut v)?`.
    pub fn find_a_point_in_the_face_uv(
        face: &Face,
        point: &mut Point3,
        u: &mut f64,
        v: &mut f64,
    ) -> Result<bool, Failure> {
        let mut param: f64 = 0.1;
        Self::find_a_point_in_the_face_uv_param(face, point, u, v, &mut param)
    }

    /// `FindAPointInTheFace(F, P)`; `BRepClass3dSolidExplorer::find_a_point_in_the_face_point(&face, &mut p)?`.
    pub fn find_a_point_in_the_face_point(
        face: &Face,
        point: &mut Point3,
    ) -> Result<bool, Failure> {
        let (mut u, mut v) = (0.0, 0.0);
        Self::find_a_point_in_the_face_uv(face, point, &mut u, &mut v)
    }

    /// `FindAPointInTheFace(F, u, v)`;
    /// `BRepClass3dSolidExplorer::find_a_point_in_the_face_parameters(&face, &mut u, &mut v)?`.
    pub fn find_a_point_in_the_face_parameters(
        face: &Face,
        u: &mut f64,
        v: &mut f64,
    ) -> Result<bool, Failure> {
        let mut point = Point3::default();
        Self::find_a_point_in_the_face_uv(face, &mut point, u, v)
    }
}

/// The edge point at `param` of the pcurve range, moved 1e-5 along the unit normal
/// that points inside the face (left of a FORWARD edge); `None` without a pcurve.
fn edge_start(edge: &Edge, face: &Face, param: f64) -> Result<Option<EdgeStart>, Failure> {
    let c = BRepAdaptorCurve2d::with_edge_on_face(edge, face)?;
    if c.geom2d_adaptor().curve().is_null() {
        return Ok(None);
    }
    let (first, last) = (c.first_parameter()?, c.last_parameter()?);
    let (p, t) = c.d1((last - first) * param + first)?;
    let (x, y) = (t.x(), t.y());
    let mut normal = if edge.orientation() == Orientation::Forward {
        Vector2::new(-y, x)
    } else {
        Vector2::new(y, -x)
    };
    normal.normalize()?;
    let origin = Point2::new(p.x() + TOL_INIT * normal.x(), p.y() + TOL_INIT * normal.y());
    Ok(Some(EdgeStart { origin, normal }))
}

/// The smallest parameter at which the ray from the start crosses another edge (not
/// EXTERNAL; the edge itself when it is the only edge of the face); `None` without a
/// crossing.
fn nearest_crossing(face: &Face, edge: &Edge, start: &EdgeStart) -> Result<Option<f64>, Failure> {
    let mut classifier = FacePassiveClassifier::new();
    let mut param_init: f64 = precision::infinite();
    let ray = Line2::new(start.origin, Direction2::from_vector(start.normal)?);
    classifier.reset(ray, param_init, real_epsilon());
    let mut found = false;
    let mut nb_edges: i32 = 0;
    for other in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        nb_edges += 1;
        let other = topods::edge(&other)?;
        if other.orientation() != Orientation::External && !other.is_equal(edge) {
            found |= compare_edge(&mut classifier, &other, face, &mut param_init)?;
        }
    }
    if nb_edges == 1 {
        found |= compare_edge(&mut classifier, edge, face, &mut param_init)?;
    }
    Ok(found.then_some(param_init))
}

/// Compares one edge with the ray; lowers `param_init` to its closest crossing.
fn compare_edge(
    classifier: &mut FacePassiveClassifier,
    edge: &Edge,
    face: &Face,
    param_init: &mut f64,
) -> Result<bool, Failure> {
    let data = BRepClassEdge::with_edge_face(edge.clone(), face.clone());
    classifier.compare(&data, edge.orientation())?;
    if classifier.closest_intersection() == 0 || *param_init <= classifier.parameter() {
        return Ok(false);
    }
    *param_init = classifier.parameter();
    Ok(true)
}

/// The repeated 0.41234 shrink of the distance to the nearest crossing.
fn shrink_to_inner_point(
    face: &Face,
    start: &EdgeStart,
    mut param_init: f64,
    sample: &mut FacePointSample,
) -> Result<bool, Failure> {
    loop {
        param_init *= 0.41234;
        sample.u = start.origin.x() + param_init * start.normal.x();
        sample.v = start.origin.y() + param_init * start.normal.y();
        // Additional check
        let classifier = BRepTopAdaptorFClass2d::new(face, precision::confusion())?;
        if classifier.perform(Point2::new(sample.u, sample.v), true)? != State::In {
            return Ok(false);
        }
        let d1 = BRepAdaptorSurface::with_face(face, false)?.d1(sample.u, sample.v)?;
        (sample.point, sample.d1u, sample.d1v) = (d1.point, d1.d1u, d1.d1v);
        if sample.d1u.cross_magnitude(sample.d1v) > gp::resolution() {
            return Ok(true);
        }
        if param_init < precision::p_confusion() {
            return Ok(false);
        }
    }
}
