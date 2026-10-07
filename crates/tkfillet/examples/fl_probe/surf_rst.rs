//! The surface-restriction fillets of BRepBlend (`fl_surf_rst`); the twin of the
//! surface-restriction case of tools/oracle/tkfillet_probe_brepblend.cxx.
use super::blend_func::{
    app_queries, curve2d, guide, matrix, sections_at, shape_of, surface, vec3, vector,
};
use super::common::{Reader, attempt, dir, end, flag, pnt, pnt2d, real};
use super::fillet::law;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::blend::{BlendAppFunction, BlendPoint, BlendSurfRstFunction};
use rustycad_tkfillet::brep_blend::{BRepBlendSurfRstConstRad, BRepBlendSurfRstEvolRad};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::gp::{Circle3, Point3, Vector2};
use rustycad_tkmath::math::{Matrix, Vector};

fn vec2(v: Vector2) {
    real(v.x());
    real(v.y());
}

/// The section access every surface-restriction class gives the probe.
trait SurfRstProbe: BlendSurfRstFunction {
    fn circle(&mut self, t: f64, x: [f64; 3]) -> Result<(f64, f64, Circle3), Failure>;
}

impl SurfRstProbe for BRepBlendSurfRstConstRad {
    fn circle(&mut self, t: f64, x: [f64; 3]) -> Result<(f64, f64, Circle3), Failure> {
        self.section_circle(t, (x[0], x[1], x[2]))
    }
}

impl SurfRstProbe for BRepBlendSurfRstEvolRad {
    fn circle(&mut self, t: f64, x: [f64; 3]) -> Result<(f64, f64, Circle3), Failure> {
        self.section_circle(t, (x[0], x[1], x[2]))
    }
}

fn equations(f: &mut dyn BlendAppFunction, x: &Vector<'_>) {
    let mut fv = Vector::filled(1, 3, 0.0).expect("vector");
    let mut d = Matrix::filled(1, 3, 1, 3, 0.0).expect("matrix");
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

fn tangents(f: &dyn BlendSurfRstFunction, solved: bool) {
    flag(f.is_tangency_point());
    attempt(|| f.tangent_on_s().map(vec3));
    attempt(|| f.tangent2d_on_s().map(vec2));
    attempt(|| f.tangent_on_rst().map(vec3));
    attempt(|| f.tangent2d_on_rst().map(vec2));
    pnt(f.point_on_s());
    pnt(f.point_on_rst());
    pnt2d(f.pnt2d_on_s());
    pnt2d(f.pnt2d_on_rst());
    if solved {
        real(f.parameter_on_rst());
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

/// `fl_surf_rst_members`.
fn members<F: SurfRstProbe>(f: &mut F, x: [f64; 3], t: f64, tol: f64) {
    let xv = vector_of(x);
    equations(f, &xv);
    let mut solved = false;
    attempt(|| {
        solved = f.is_solution(&xv, tol)?;
        flag(solved);
        Ok(())
    });
    tangents(f, solved);
    attempt(|| {
        f.decroch(&xv).map(|(found, ns, tg)| {
            flag(found);
            vec3(ns);
            vec3(tg);
        })
    });
    attempt(|| f.circle(t, x).map(print_circle));
    sections_at(
        f,
        &BlendPoint::on_surface_curve(Point3::default(), Point3::default(), t, x),
    );
    app_queries(f, tol);
}

fn vector_of(x: [f64; 3]) -> Vector<'static> {
    let mut v = Vector::filled(1, 3, 0.0).expect("vector");
    for (i, value) in (1..).zip(x) {
        v[i] = value;
    }
    v
}

/// The fields after the curves of `fl_surf_rst`.
struct SurfRstRequest {
    shape: i32,
    t: f64,
    trim: Option<(f64, f64)>,
    x: [f64; 3],
    tol: f64,
}

macro_rules! run_surf_rst {
    ($f:expr, $srst:expr, $rst:expr, $request:expr) => {{
        let (f, request) = ($f, $request);
        f.set_shape(shape_of(0));
        let _ = f.get_shape();
        f.set_shape(shape_of(request.shape));
        f.set_refs($srst, $rst);
        if let Some((first, last)) = request.trim {
            attempt(|| f.set_interval(first, last));
        }
        attempt(|| f.set_param(request.t));
        members(f, request.x, request.t, request.tol);
    }};
}

/// `fl_surf_rst kind S SRst Rst C Law radius choix shape t trim first last X1..X3 tol`.
pub fn surf_rst_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let (s, srst): (Handle<dyn Adaptor3dSurface>, Handle<dyn Adaptor3dSurface>) =
        (surface(&mut r), surface(&mut r));
    let rst: Handle<dyn Adaptor2dCurve2d> = curve2d(&mut r);
    let (c, evol) = (guide(&mut r), law(&mut r));
    let (radius, choix, shape, t) = (r.real(), r.integer(), r.integer(), r.real());
    let (trim, first, last) = (r.flag(), r.real(), r.real());
    let x = [r.real(), r.real(), r.real()];
    let request = SurfRstRequest {
        shape,
        t,
        trim: trim.then_some((first, last)),
        x,
        tol: r.real(),
    };
    if kind == 0 {
        let mut f = BRepBlendSurfRstConstRad::new((&s, &srst, &rst), &c).expect("function");
        f.set_radius(radius, choix);
        run_surf_rst!(&mut f, &srst, &rst, &request);
    } else {
        let mut f = BRepBlendSurfRstEvolRad::new((&s, &srst, &rst), &c, &evol).expect("function");
        f.set_choix(choix);
        run_surf_rst!(&mut f, &srst, &rst, &request);
    }
    end();
}
