// Rust translation of BRepClass3d_SolidExplorer::OtherSegment (OCC578, OCC454) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `OtherSegment`: a segment from the classified point towards a point of a face,
//! steep enough on that face to cross it cleanly.
use super::{BRepClass3dSolidExplorer, FacePointSample};
use crate::brep_class::BRepClassFaceClassifier;
use crate::brep_top_adaptor::BRepTopAdaptorFClass2d;
use crate::int_curves_face::IntCurvesFaceIntersector;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag, ExtPS};
use rustycad_tkmath::gp::{self, Direction3, Line3, Point2, Point3, Vector3};

/// The segment found so far: the line, its length and whether one was found
/// (`L`, `_Par`, `ptfound`), the best steepness `maxscal` and the count of points
/// found `NbPointsOK`, which all run over the faces and the restarts, as does the
/// face point `APoint` with its derivatives (a face without projection starts from
/// the point of the face before).
struct SegmentSearch<'o> {
    line: &'o mut Line3,
    par: &'o mut f64,
    found: bool,
    maxscal: f64,
    nb_points_ok: i32,
    sample: FacePointSample,
}

/// `IsInfiniteUV`: a bit per infinite bound (1 U1, 2 V1, 4 U2, 8 V2).
fn is_infinite_uv([u1, v1, u2, v2]: [f64; 4]) -> i32 {
    [u1, v1, u2, v2]
        .iter()
        .enumerate()
        .filter(|(_, x)| precision::is_infinite(**x))
        .fold(0, |flags, (bit, _)| flags | (1 << bit))
}

/// `std::max(a, b)`.
fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

impl BRepClass3dSolidExplorer {
    /// `OtherSegment(P, L, Par)`: from the face after the one of the last segment,
    /// returns 0 with `line` and `par` set to a segment from `point` to a point of a
    /// face (the projection of `point` on an infinite face, else a point found by
    /// `PointInTheFace` near that projection whose line makes the largest angle with
    /// the face, above 0.2 at once), 1 when `point` is on an infinite face or IN or ON
    /// a face through its projection, 3 when it is on the surface but outside the
    /// face, 2 for a face of degenerate UV bounds. After the last face without a
    /// segment the search restarts from the first face with the next edge parameter
    /// (0.512345, 0.4, 0.6, 0.3, 0.7, 0.2, 0.8, 0.1, 0.9, then halved; below 1e-4 the
    /// segment along +X of length 1) and the faces whose infinite point is IN
    /// unrestricted. A shape without faces becomes rejecting (0, `par` 0);
    /// `explorer.other_segment(&p, &mut line, &mut par)?`.
    pub fn other_segment(
        &mut self,
        point: &Point3,
        line: &mut Line3,
        par: &mut f64,
    ) -> Result<i32, Failure> {
        let sample = FacePointSample::default();
        let mut search = SegmentSearch {
            line,
            par,
            found: false,
            maxscal: 0.0,
            nb_points_ok: 0,
            sample,
        };
        let mut test_invert = false;
        loop {
            self.my_first_face += 1;
            let mut nb_faces: i32 = 0;
            for face in Explorer::new(&self.my_shape.clone(), ShapeEnum::Face, ShapeEnum::Shape)? {
                nb_faces += 1;
                if self.my_first_face > nb_faces {
                    continue;
                }
                if let Some(flag) =
                    self.segment_to_face(point, &topods::face(&face)?, test_invert, &mut search)?
                {
                    return Ok(flag);
                }
            }
            if nb_faces == 0 {
                *search.par = 0.0;
                self.my_reject = true;
                return Ok(0);
            }
            if search.found {
                return Ok(0);
            }
            self.my_first_face = 0;
            if self.next_param_on_edge() {
                *search.line = along_x(point)?;
                *search.par = 1.0;
                return Ok(0);
            }
            test_invert = true;
        }
    }

    /// The next edge parameter of the restart; true when it fell below 1e-4 (the
    /// segment along +X is then returned).
    fn next_param_on_edge(&mut self) -> bool {
        const CYCLE: [(f64, f64); 8] = [
            (0.512345, 0.4),
            (0.4, 0.6),
            (0.6, 0.3),
            (0.3, 0.7),
            (0.7, 0.2),
            (0.2, 0.8),
            (0.8, 0.1),
            (0.1, 0.9),
        ];
        if let Some((_, next)) = CYCLE
            .iter()
            .find(|(current, _)| *current == self.my_param_on_edge)
        {
            self.my_param_on_edge = *next;
            return false;
        }
        self.my_param_on_edge *= 0.5;
        self.my_param_on_edge < 0.0001
    }

    /// One face of `OtherSegment`: `Some(flag)` returns at once.
    fn segment_to_face(
        &mut self,
        point: &Point3,
        face_in: &Face,
        test_invert: bool,
        search: &mut SegmentSearch<'_>,
    ) -> Result<Option<i32>, Failure> {
        let restricted = !test_invert || self.keeps_restriction(face_in)?;
        let surface = BRepAdaptorSurface::with_face(face_in, restricted)?;
        let rectangle = [
            surface.first_u_parameter()?,
            surface.first_v_parameter()?,
            surface.last_u_parameter()?,
            surface.last_v_parameter()?,
        ];
        let face = topods::face(&face_in.oriented(Orientation::Forward))?;
        if degenerate_bounds(rectangle) {
            return Ok(Some(2)); // avoid process faces from uncorrected shells
        }
        let saved_param = self.my_param_on_edge;
        if let Some(flag) = project_on_face(point, &face, rectangle, search)? {
            return Ok(Some(flag));
        }
        let steep = self.search_face_points(point, &face, &surface, rectangle, search)?;
        self.my_param_on_edge = saved_param;
        Ok((steep || search.maxscal > 0.2).then_some(0))
    }

    /// With `aTestInvert`: false (and a new unrestricted intersector for a mapped
    /// face) when the infinite point of the face is IN, else true.
    fn keeps_restriction(&mut self, face: &Face) -> Result<bool, Failure> {
        let classifier = BRepTopAdaptorFClass2d::new(face, precision::confusion())?;
        if classifier.perform_infinite_point()? != State::In {
            return Ok(true);
        }
        if self.my_map_of_inter.is_bound(face) {
            self.my_map_of_inter.unbind(face);
            let intersector =
                IntCurvesFaceIntersector::new(face, precision::confusion(), false, false)?;
            self.my_map_of_inter
                .bind(face.clone().into_shape(), intersector);
        }
        Ok(false)
    }

    /// The `PointInTheFace` loop: up to 200 indices while fewer than 16 points were
    /// found (the count runs over all the faces); keeps the steepest segment, true
    /// as soon as its steepness exceeds 0.2.
    fn search_face_points(
        &mut self,
        point: &Point3,
        face: &Face,
        surface: &BRepAdaptorSurface,
        rectangle: [f64; 4],
        search: &mut SegmentSearch<'_>,
    ) -> Result<bool, Failure> {
        let mut index: i32 = 0;
        loop {
            index += 1;
            let mut param = self.my_param_on_edge;
            let mut sample = search.sample;
            let found = self.point_in_the_face_on_surface_d1(
                face,
                &mut sample,
                &mut param,
                &mut index,
                surface,
                rectangle,
            )?;
            search.sample = sample;
            self.my_param_on_edge = param;
            if found {
                search.nb_points_ok += 1;
                if keep_steeper(point, &sample, search)? {
                    return Ok(true);
                }
            }
            if !(index < 200 && search.nb_points_ok < 16) {
                return Ok(false);
            }
        }
    }
}

/// `|U2 - U1| < epsU || |V2 - V1| < epsV` with the relative epsilons.
fn degenerate_bounds([u1, v1, u2, v2]: [f64; 4]) -> bool {
    let eps: f64 = precision::p_confusion();
    let eps_u = std_max(eps * std_max(u2.abs(), u1.abs()), eps);
    let eps_v = std_max(eps * std_max(v2.abs(), v1.abs()), eps);
    (u2 - u1).abs() < eps_u || (v2 - v1).abs() < eps_v
}

/// The projection of the point on the whole face surface (Extrema_ExtPS, MIN): on the
/// surface (squared distance below 1e-24) it decides 1 (infinite face, or IN/ON the
/// face) or 3; on an infinite face the segment goes to the projection (0); else the
/// search starts from the nearest projection inside the bounds (from the bounds'
/// centre, and the point of the face before, without a projection); `None` searches.
fn project_on_face(
    point: &Point3,
    face: &Face,
    rectangle: [f64; 4],
    search: &mut SegmentSearch<'_>,
) -> Result<Option<i32>, Failure> {
    let [u1, v1, u2, v2] = rectangle;
    let inf_flag = is_infinite_uv(rectangle);
    // default values
    (search.sample.u, search.sample.v) = ((u1 + u2) * 0.5, (v1 + v2) * 0.5);
    let tol = precision::p_confusion();
    let ga = GeomAdaptorSurface::with_surface(&BRepTool::surface(face)?)?;
    let ext = ExtPS::with_surface(*point, &ga, tol, tol, ExtFlag::Min, ExtAlgo::Grad)?;
    if !(ext.is_done() && ext.nb_ext()? > 0) {
        return Ok(None);
    }
    let (near, dist2_min) = nearest_projection(&ext, rectangle)?;
    if dist2_min < 1.0e-24 {
        return Ok(Some(on_surface_flag(inf_flag, face, &ext, near)?));
    }
    let projection = ext.point(near)?;
    search.sample.point = *projection.value();
    if inf_flag != 0 {
        let v = Vector3::between(*point, search.sample.point);
        *search.par = v.magnitude();
        *search.line = Line3::new(*point, Direction3::from_vector(v)?);
        search.found = true;
        return Ok(Some(0));
    }
    // set the parameters found by extrema
    (search.sample.u, search.sample.v) = projection.parameter();
    Ok(None)
}

/// The first projection, or a nearer one whose parameters lie in the bounds.
fn nearest_projection(ext: &ExtPS<'_>, [u1, v1, u2, v2]: [f64; 4]) -> Result<(i32, f64), Failure> {
    let (mut near, mut dist2_min) = (1, ext.square_distance(1)?);
    for i in 2..=ext.nb_ext()? {
        let (ux, vx) = ext.point(i)?.parameter();
        if ux >= u1 && ux <= u2 && vx >= v1 && vx <= v2 {
            let dist2 = ext.square_distance(i)?;
            if dist2 < dist2_min {
                (near, dist2_min) = (i, dist2);
            }
        }
    }
    Ok((near, dist2_min))
}

/// A point on the face surface: 1 on an infinite face, else 1 when its projection is
/// IN or ON the face (BRepClass_FaceClassifier, Precision::PConfusion()), else 3
/// ("the point is on surface but outside face").
fn on_surface_flag(inf_flag: i32, face: &Face, ext: &ExtPS<'_>, near: i32) -> Result<i32, Failure> {
    if inf_flag != 0 {
        return Ok(1);
    }
    let (u, v) = ext.point(near)?.parameter();
    let mut classifier = BRepClassFaceClassifier::new();
    classifier.perform(
        face,
        &Point2::new(u, v),
        precision::p_confusion(),
        false,
        0.1,
    )?;
    Ok(if matches!(classifier.state(), State::In | State::On) {
        1
    } else {
        3
    })
}

/// Keeps the segment to the found point when it is steeper on the face than the
/// best so far (|N.V| / (|N| |V|)); true when that exceeds 0.2.
fn keep_steeper(
    point: &Point3,
    sample: &FacePointSample,
    search: &mut SegmentSearch<'_>,
) -> Result<bool, Failure> {
    let v = Vector3::between(*point, sample.point);
    let par = v.magnitude();
    let resolution = gp::resolution();
    if !(par > resolution
        && sample.d1u.magnitude() > resolution
        && sample.d1v.magnitude() > resolution)
    {
        return Ok(false);
    }
    let norm = sample.d1u.crossed(sample.d1v);
    let tt = norm.magnitude();
    if tt <= resolution {
        return Ok(false);
    }
    let tt = norm.dot(v).abs() / (tt * par);
    if tt <= search.maxscal {
        return Ok(false);
    }
    search.maxscal = tt;
    *search.line = Line3::new(*point, Direction3::from_vector(v)?);
    *search.par = par;
    search.found = true;
    Ok(search.maxscal > 0.2)
}

/// The segment of length 1 along +X ("the point is on the solid and this solid is
/// reduced to a face").
fn along_x(point: &Point3) -> Result<Line3, Failure> {
    let bidon = Point3::new(point.x() + 1.0, point.y(), point.z());
    Ok(Line3::new(
        *point,
        Direction3::from_vector(Vector3::between(*point, bidon))?,
    ))
}
