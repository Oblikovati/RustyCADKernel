//! BRepIntCurveSurface probe operations, the Rust twin of
//! tools/oracle/tktopalgo_probe_brepintcurvesurface.cxx: lines and curves intersected
//! with shapes read by `tp_shape`; faces printed as their index among the faces of the
//! shape, states and transitions as integers.
use super::brep_approx::bra_bspline;
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use super::shape::tp_shape;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Face, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine, GeomParabola};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis2, Direction3, Line3, Point3};
use rustycad_tktopalgo::brep_int_curve_surface::BRepIntCurveSurfaceInter;

/// `tp_bics_face_index`.
fn face_index(shape: &Shape, face: &Face) -> Result<i32, Failure> {
    let mut i = 0;
    for f in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        i += 1;
        if f.is_same(face) {
            return Ok(i);
        }
    }
    Ok(-1)
}

/// `tp_bics_current`.
fn current(shape: &Shape, s: &BRepIntCurveSurfaceInter) {
    attempt(|| face_index(shape, &s.face()?).map(int));
    attempt(|| {
        let p = s.point()?;
        pnt(p.pnt());
        real(p.u());
        real(p.v());
        real(p.w());
        int(p.transition() as i32);
        Ok(())
    });
    attempt(|| s.pnt().map(pnt));
    attempt(|| s.u().map(real));
    attempt(|| s.v().map(real));
    attempt(|| s.w().map(real));
    attempt(|| s.state().map(|x| int(x as i32)));
    attempt(|| s.transition().map(|x| int(x as i32)));
}

/// `tp_bics_walk`.
fn walk(shape: &Shape, s: &mut BRepIntCurveSurfaceInter) {
    let mut n = 0;
    while s.more() && n < 64 {
        current(shape, s);
        attempt(|| s.next());
        n += 1;
    }
    int(n);
    flag(s.more());
    current(shape, s);
    attempt(|| s.next());
    flag(s.more());
    attempt(|| face_index(shape, &s.face()?).map(int));
}

/// The frame `P N X` of a circle or a parabola.
fn frame(r: &mut Reader) -> Result<Axis2, Failure> {
    let p = r.pnt();
    let (n, x) = (r.dir(), r.dir());
    Axis2::with_x_direction(p, n, x)
}

/// `tp_bics_curve`: `kind fields first last`.
fn curve(r: &mut Reader) -> Result<GeomAdaptorCurve, Failure> {
    let kind = r.integer();
    let c: Handle<dyn GeomCurve> = match kind {
        0 => {
            let p = r.pnt();
            Handle::new(GeomLine::from_point(p, r.dir())).cast()
        }
        2 => bra_bspline(r)?.cast(),
        _ => {
            let a = frame(r)?;
            let v = r.real();
            if kind == 1 {
                Handle::new(GeomCircle::new(a, v)?).cast()
            } else {
                Handle::new(GeomParabola::new(a, v)?).cast()
            }
        }
    };
    let (first, last) = (r.real(), r.real());
    GeomAdaptorCurve::with_range(&c, first, last)
}

/// `bics_line`: `shape tol n (P D)...`.
pub fn line_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let sh = tp_shape(&mut r)?;
        let tol = r.real();
        let mut s = BRepIntCurveSurfaceInter::new();
        for _ in 0..r.integer() {
            let p = r.pnt();
            let d = r.dir();
            attempt(|| s.init_line(&sh.shape, Line3::new(p, d), tol));
            walk(&sh.shape, &mut s);
        }
        Ok(())
    });
    end();
}

/// `bics_curve`: `shape tol mode reload n (curve)...`.
pub fn curve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let sh = tp_shape(&mut r)?;
        let tol = r.real();
        let (mode, reload, n) = (r.integer(), r.flag(), r.integer());
        let mut s = BRepIntCurveSurfaceInter::new();
        if mode == 0 {
            s.load(&sh.shape, tol)?;
            if reload {
                s.load(&sh.shape, tol)?;
            }
            flag(s.more());
        }
        for _ in 0..n {
            let c = curve(&mut r)?;
            if mode == 0 {
                attempt(|| s.init(&c));
            } else {
                attempt(|| s.init_curve(&sh.shape, &c, tol));
            }
            walk(&sh.shape, &mut s);
        }
        Ok(())
    });
    end();
}

/// `bics_state`: `shape tol`.
pub fn state_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let sh = tp_shape(&mut r)?;
        let tol = r.real();
        let mut s = BRepIntCurveSurfaceInter::new();
        flag(s.more());
        current(&sh.shape, &s);
        let origin = Point3::new(0.0, 0.0, 0.0);
        let l: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
            origin,
            Direction3::new(0.0, 0.0, 1.0)?,
        ))
        .cast();
        attempt(|| s.init(&GeomAdaptorCurve::with_range(&l, -1.0, 1.0)?));
        flag(s.more());
        s.load(&sh.shape, tol)?;
        flag(s.more());
        current(&sh.shape, &s);
        attempt(|| s.next());
        walk(&sh.shape, &mut s);
        let mut empty = Compound::default();
        BRepBuilder::default().make_compound(&mut empty);
        s.load(&empty.into_shape(), tol)?;
        flag(s.more());
        attempt(|| s.next());
        flag(s.more());
        attempt(|| s.next());
        flag(s.more());
        current(&sh.shape, &s);
        Ok(())
    });
    end();
}
