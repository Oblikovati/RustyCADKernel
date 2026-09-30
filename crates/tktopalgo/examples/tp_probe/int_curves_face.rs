//! IntCurvesFace probe operations over faces read by `tp_face`, the Rust twin of
//! tools/oracle/tktopalgo_probe_intcurvesface.cxx. States and transitions are printed
//! as integers, faces of a shape as their index in the shape.
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use super::face::{TpFace, tp_face};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Face};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCircle, GeomCurve, GeomLine};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeomalgo::int_curve_surface::HCurve;
use rustycad_tkmath::gp::{Axis2, Line3, Point2, Point3};
use rustycad_tktopalgo::int_curves_face::{
    IntCurvesFaceIntersector, IntCurvesFaceShapeIntersector,
};

/// The point accessors of the two intersectors.
trait PointSource {
    fn is_done(&self) -> bool;
    fn nb_pnt(&self) -> i32;
    fn point(&self, i: i32) -> Result<(), Failure>;
}

impl PointSource for IntCurvesFaceIntersector {
    fn is_done(&self) -> bool {
        IntCurvesFaceIntersector::is_done(self)
    }
    fn nb_pnt(&self) -> i32 {
        IntCurvesFaceIntersector::nb_pnt(self)
    }
    fn point(&self, i: i32) -> Result<(), Failure> {
        real(self.u_parameter(i)?);
        real(self.v_parameter(i)?);
        real(self.w_parameter(i)?);
        pnt(self.pnt(i)?);
        int(self.transition(i)? as i32);
        int(self.state(i)? as i32);
        Ok(())
    }
}

impl PointSource for IntCurvesFaceShapeIntersector {
    fn is_done(&self) -> bool {
        IntCurvesFaceShapeIntersector::is_done(self)
    }
    fn nb_pnt(&self) -> i32 {
        IntCurvesFaceShapeIntersector::nb_pnt(self)
    }
    fn point(&self, i: i32) -> Result<(), Failure> {
        real(self.u_parameter(i)?);
        real(self.v_parameter(i)?);
        real(self.w_parameter(i)?);
        pnt(self.pnt(i)?);
        int(self.transition(i)? as i32);
        int(self.state(i)? as i32);
        Ok(())
    }
}

/// `tp_icf_points`: the points 1..NbPnt, then at 0 and NbPnt + 1.
fn points(c: &impl PointSource) {
    flag(c.is_done());
    int(c.nb_pnt());
    for i in 1..=c.nb_pnt() {
        attempt(|| c.point(i));
    }
    attempt(|| c.point(0));
    attempt(|| c.point(c.nb_pnt() + 1));
}

/// `tp_icf_results`: points, parallel flag and the classification of each UV.
fn results(c: &IntCurvesFaceIntersector) {
    points(c);
    flag(c.is_parallel());
    for i in 1..=c.nb_pnt() {
        attempt(|| {
            let uv = Point2::new(c.u_parameter(i)?, c.v_parameter(i)?);
            int(c.classify_uv_point(uv)? as i32);
            Ok(())
        });
    }
}

/// `face tol restr usebtol n (P D pmin pmax)...`: one intersector on several lines.
pub fn line_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = tp_face(&mut r)?;
        let tol = r.real();
        let (restr, usebtol) = (r.flag(), r.flag());
        let mut c = IntCurvesFaceIntersector::new(&f.face, tol, restr, usebtol)?;
        int(c.surface_type()? as i32);
        flag(c.get_use_bound_toler());
        flag(c.is_done());
        let b = c.bounding();
        flag(b.is_void());
        if !b.is_void() {
            b.get()?.into_iter().for_each(real);
        }
        for _ in 0..r.integer() {
            let (p, d) = (r.pnt(), r.dir());
            let (a, e) = (r.real(), r.real());
            attempt(|| {
                c.perform_line(&Line3::new(p, d), a, e)?;
                results(&c);
                Ok(())
            });
        }
        c.set_use_bound_toler(!usebtol);
        flag(c.get_use_bound_toler());
        flag(c.face().is_same(&f.face));
        Ok(())
    });
    end();
}

/// `tp_icf_curve`: a line, circle or Bezier curve adaptor on `[first, last]`.
fn curve(r: &mut Reader<'_>) -> Result<HCurve, Failure> {
    let kind = r.integer();
    let c: Handle<dyn GeomCurve> = match kind {
        0 => {
            let p = r.pnt();
            Handle::new(GeomLine::from_point(p, r.dir())).cast()
        }
        1 => {
            let (p, n) = (r.pnt(), r.dir());
            Handle::new(GeomCircle::new(Axis2::new(p, n)?, r.real())?).cast()
        }
        _ => {
            let n = r.integer();
            let mut poles = Array1::<Point3>::with_bounds(1, n)
                .map_err(|error| Failure::new(&format!("{error:?}")))?;
            for i in 1..=n {
                poles[i] = r.pnt();
            }
            Handle::new(GeomBezierCurve::new(&poles)?).cast()
        }
    };
    let (first, last) = (r.real(), r.real());
    Ok(Handle::new(GeomAdaptorCurve::with_range(&c, first, last)?).cast::<dyn Adaptor3dCurve>())
}

/// `face tol restr usebtol curve pmin pmax`: the results of Perform on a curve adaptor.
pub fn curve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = tp_face(&mut r)?;
        let tol = r.real();
        let (restr, usebtol) = (r.flag(), r.flag());
        let hc = curve(&mut r)?;
        let (a, e) = (r.real(), r.real());
        let mut c = IntCurvesFaceIntersector::new(&f.face, tol, restr, usebtol)?;
        c.perform_curve(&hc, a, e)?;
        results(&c);
        Ok(())
    });
    end();
}

/// `tp_icf_face_index`: the index of `face` among the faces of the compound.
fn face_index(faces: &[TpFace], face: &Face) -> i32 {
    faces
        .iter()
        .position(|candidate| candidate.face.is_same(face))
        .map_or(-1, |k| k as i32)
}

/// One line request of a shape intersector: Perform, PerformNearest or Perform of a
/// curve adaptor on the line.
fn shape_request(
    s: &mut IntCurvesFaceShapeIntersector,
    mode: i32,
    line: &Line3,
    (a, e): (f64, f64),
) -> Result<(), Failure> {
    match mode {
        0 => s.perform_line(line, a, e),
        1 => s.perform_nearest(line, a, e),
        _ => {
            let geom: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_lin(*line)).cast();
            let hc: HCurve = Handle::new(GeomAdaptorCurve::with_range(&geom, a, e)?).cast();
            s.perform_curve(&hc, a, e)
        }
    }
}

/// `nfaces (face)... tol reload mode n (P D pmin pmax)...`: a shape intersector.
pub fn shape_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let b = BRepBuilder::default();
        let mut comp = Compound::default();
        b.make_compound(&mut comp);
        let mut faces: Vec<TpFace> = Vec::new();
        for _ in 0..r.integer() {
            let f = tp_face(&mut r)?;
            b.add(&mut comp, &f.face)?;
            faces.push(f);
        }
        let tol = r.real();
        let (reload, mode) = (r.flag(), r.integer());
        let mut s = IntCurvesFaceShapeIntersector::new();
        flag(s.is_done());
        s.load(&comp, tol)?;
        if reload {
            s.load(&comp, tol)?;
        }
        for _ in 0..r.integer() {
            let (p, d) = (r.pnt(), r.dir());
            let (a, e) = (r.real(), r.real());
            attempt(|| {
                shape_request(&mut s, mode, &Line3::new(p, d), (a, e))?;
                points(&s);
                for j in 1..=s.nb_pnt() {
                    int(face_index(&faces, s.face(j)?));
                }
                attempt(|| s.face(0).map(|_| ()));
                attempt(|| s.face(s.nb_pnt() + 1).map(|_| ()));
                Ok(())
            });
        }
        s.sort_result()?;
        flag(s.is_done());
        int(s.nb_pnt());
        Ok(())
    });
    end();
}
