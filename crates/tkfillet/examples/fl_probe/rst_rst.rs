//! The restriction-restriction fillets of BRepBlend (`fl_rst_rst`); the twin of the
//! restriction-restriction case of tools/oracle/tkfillet_probe_brepblend.cxx.
use super::blend_func::{
    app_queries, curve2d, guide, matrix, sections_at, shape_of, surface, vec3, vector,
};
use super::common::{Reader, attempt, dir, end, flag, int, pnt, pnt2d, real};
use super::fillet::law;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::blend::{BlendAppFunction, BlendPoint, BlendRstRstFunction};
use rustycad_tkfillet::brep_blend::{BRepBlendRstRstConstRad, BRepBlendRstRstEvolRad};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::gp::{Circle3, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

fn vec2(v: Vector2) {
    real(v.x());
    real(v.y());
}

/// The class members the probe calls beyond the traits.
trait RstRstProbe: BlendRstRstFunction {
    fn circle(&mut self, t: f64, x: [f64; 2]) -> Result<(f64, f64, Circle3), Failure>;
    fn center(&self, p1: Point3, p2: Point3) -> Result<(bool, Point3, Vector3), Failure>;
}

macro_rules! impl_probe {
    ($class:ty) => {
        impl RstRstProbe for $class {
            fn circle(&mut self, t: f64, x: [f64; 2]) -> Result<(f64, f64, Circle3), Failure> {
                self.section_circle(t, (x[0], x[1]))
            }
            fn center(&self, p1: Point3, p2: Point3) -> Result<(bool, Point3, Vector3), Failure> {
                self.center_circle_rst1_rst2(p1, p2, Vector3::new(1., 0., 0.))
            }
        }
    };
}

impl_probe!(BRepBlendRstRstConstRad);
impl_probe!(BRepBlendRstRstEvolRad);

fn equations(f: &mut dyn BlendAppFunction, x: &Vector<'_>) {
    let mut fv = Vector::filled(1, 2, 0.0).expect("vector");
    let mut d = Matrix::filled(1, 2, 1, 2, 0.0).expect("matrix");
    let ok = f.value(x, &mut fv);
    attempt(|| f.take_failure().map(|()| (flag(ok), vector(&fv)).1));
    let ok = f.derivatives(x, &mut d);
    attempt(|| f.take_failure().map(|()| (flag(ok), matrix(&d)).1));
    let ok = f.values(x, &mut fv, &mut d);
    attempt(|| {
        f.take_failure().map(|()| {
            flag(ok);
            vector(&fv);
            matrix(&d);
        })
    });
}

fn tangents(f: &dyn BlendRstRstFunction, solved: bool) {
    flag(f.is_tangency_point());
    attempt(|| f.tangent_on_rst1().map(vec3));
    attempt(|| f.tangent2d_on_rst1().map(vec2));
    attempt(|| f.tangent_on_rst2().map(vec3));
    attempt(|| f.tangent2d_on_rst2().map(vec2));
    pnt(f.point_on_rst1());
    pnt(f.point_on_rst2());
    pnt2d(f.pnt2d_on_rst1());
    pnt2d(f.pnt2d_on_rst2());
    if solved {
        real(f.parameter_on_rst1());
        real(f.parameter_on_rst2());
    }
}

fn print_circle((pdeb, pfin, circle): (f64, f64, Circle3)) {
    real(pdeb);
    real(pfin);
    let frame = circle.position();
    pnt(frame.location());
    dir(frame.direction());
    dir(frame.x_direction());
    real(circle.radius());
}

/// `fl_rst_rst_members`.
fn members<F: RstRstProbe>(f: &mut F, x: [f64; 2], t: f64, tol: f64) {
    let mut xv = Vector::filled(1, 2, 0.0).expect("vector");
    (xv[1], xv[2]) = (x[0], x[1]);
    equations(f, &xv);
    let mut solved = false;
    attempt(|| {
        solved = f.is_solution(&xv, tol)?;
        flag(solved);
        Ok(())
    });
    tangents(f, solved);
    attempt(|| {
        f.decroch(&xv).map(|(status, vectors)| {
            int(status as i32);
            vectors.into_iter().for_each(vec3);
        })
    });
    attempt(|| {
        f.center(f.point_on_rst1(), f.point_on_rst2())
            .map(|(found, center, med)| {
                flag(found);
                pnt(center);
                vec3(med);
            })
    });
    attempt(|| f.circle(t, x).map(print_circle));
    let point = BlendPoint::on_rsts(Point3::default(), Point3::default(), t, [0.; 4], x);
    sections_at(f, &point);
    app_queries(f, tol);
}

/// The fields after the curves of `fl_rst_rst`.
struct RstRstRequest {
    shape: i32,
    t: f64,
    trim: Option<(f64, f64)>,
    x: [f64; 2],
    tol: f64,
}

macro_rules! run_rst_rst {
    ($f:expr, $refs:expr, $request:expr) => {{
        let (f, request) = ($f, $request);
        f.set_shape(shape_of(0));
        let _ = f.get_shape();
        f.set_shape(shape_of(request.shape));
        f.set_refs($refs);
        if let Some((first, last)) = request.trim {
            attempt(|| f.set_interval(first, last));
        }
        attempt(|| f.set_param(request.t));
        members(f, request.x, request.t, request.tol);
    }};
}

/// `fl_rst_rst kind S1 Rst1 S2 Rst2 C Law radius choix shape t trim first last X1 X2 tol`.
pub fn rst_rst_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let s1: Handle<dyn Adaptor3dSurface> = surface(&mut r);
    let rst1: Handle<dyn Adaptor2dCurve2d> = curve2d(&mut r);
    let s2: Handle<dyn Adaptor3dSurface> = surface(&mut r);
    let rst2: Handle<dyn Adaptor2dCurve2d> = curve2d(&mut r);
    let (c, evol) = (guide(&mut r), law(&mut r));
    let (radius, choix, shape, t) = (r.real(), r.integer(), r.integer(), r.real());
    let (trim, first, last) = (r.flag(), r.real(), r.real());
    let x = [r.real(), r.real()];
    let request = RstRstRequest {
        shape,
        t,
        trim: trim.then_some((first, last)),
        x,
        tol: r.real(),
    };
    let inputs = (&s1, &rst1, &s2, &rst2);
    if kind == 0 {
        let mut f = BRepBlendRstRstConstRad::new(inputs, &c).expect("function");
        f.set_radius(radius, choix);
        run_rst_rst!(&mut f, inputs, &request);
    } else {
        let mut f = BRepBlendRstRstEvolRad::new(inputs, &c, &evol).expect("function");
        f.set_choix(choix);
        run_rst_rst!(&mut f, inputs, &request);
    }
    end();
}
