//! IntImp probe operations, the Rust twin of tools/oracle/tkgeomalgo_probe_intimp.cxx.
//! The native probe drives the templates through their TKGeomAlgo instances; this
//! twin instantiates the generics with the same tools (`HSurfaceTool`,
//! `IntSurfQuadricTool`, and [`HCurveTool`], the calls of
//! `IntCurveSurface_TheHCurveTool`).
use super::common::{Reader, end, flag, int, pnt, real, show, xyz};
use super::int_surf::quadric;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomCylindricalSurface, GeomLine, GeomPlane, GeomSphericalSurface,
    GeomSurface, GeomToroidalSurface,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeomalgo::int_imp::*;
use rustycad_tkgeomalgo::int_surf::IntSurfQuadricTool;
use rustycad_tkmath::gp::{Axis2, Axis3, Direction2, Point3, Vector3};
use rustycad_tkmath::math::{FunctionSetRoot, Matrix, Vector};

type Curve = Handle<dyn Adaptor3dCurve>;

/// `IntCurveSurface_TheHCurveTool`: direct calls on the curve adaptor.
struct HCurveTool;

impl TheCurveTool for HCurveTool {
    type Curve = Curve;
    fn first_parameter(curve: &Curve) -> Result<f64, Failure> {
        curve.first_parameter()
    }
    fn last_parameter(curve: &Curve) -> Result<f64, Failure> {
        curve.last_parameter()
    }
    fn resolution(curve: &Curve, r3d: f64) -> Result<f64, Failure> {
        curve.resolution(r3d)
    }
    fn value(curve: &Curve, w: f64) -> Result<Point3, Failure> {
        curve.value(w)
    }
    fn d1(curve: &Curve, w: f64) -> Result<(Point3, Vector3), Failure> {
        curve.d1(w)
    }
}

const ISOS: [IntImpConstIsoparametric; 4] = [
    IntImpConstIsoparametric::UIsoparametricOnCaro1,
    IntImpConstIsoparametric::VIsoparametricOnCaro1,
    IntImpConstIsoparametric::UIsoparametricOnCaro2,
    IntImpConstIsoparametric::VIsoparametricOnCaro2,
];

fn surface(r: &mut Reader) -> Adaptor3dSurfaceHandle {
    let kind = r.integer();
    let (p, d) = (r.pnt(), r.dir());
    let (r1, r2) = (r.real(), r.real());
    let bounds = (r.real(), r.real(), r.real(), r.real());
    let ax = Axis3::from_normal(p, d).expect("probe frame");
    let s: Handle<dyn GeomSurface> = match kind {
        0 => Handle::new(GeomPlane::new(ax)).cast(),
        1 => Handle::new(GeomCylindricalSurface::new(ax, r1).expect("probe cylinder")).cast(),
        2 => Handle::new(GeomSphericalSurface::new(ax, r1).expect("probe sphere")).cast(),
        _ => Handle::new(GeomToroidalSurface::new(ax, r1, r2).expect("probe torus")).cast(),
    };
    Handle::new(GeomAdaptorSurface::with_bounds(&s, bounds).expect("probe adaptor")).cast()
}

fn curve(r: &mut Reader) -> Curve {
    let kind = r.integer();
    let (p, d) = (r.pnt(), r.dir());
    let (rad, first, last) = (r.real(), r.real(), r.real());
    let c: Handle<dyn GeomCurve> = if kind == 0 {
        Handle::new(GeomLine::from_point(p, d)).cast()
    } else {
        let ax = Axis2::new(p, d).expect("probe frame");
        Handle::new(GeomCircle::new(ax, rad).expect("probe circle")).cast()
    };
    Handle::new(GeomAdaptorCurve::with_range(&c, first, last).expect("probe adaptor")).cast()
}

fn vector(v: &Vector<'_>) {
    for i in v.lower()..=v.upper() {
        real(v[i]);
    }
}

fn matrix(m: &Matrix<'_>) {
    for i in m.lower_row()..=m.upper_row() {
        for j in m.lower_col()..=m.upper_col() {
            real(m[(i, j)]);
        }
    }
}

fn dir2d(d: Direction2) {
    real(d.x());
    real(d.y());
}

/// Prints a fallible block's exception type, as GA_TRY does.
fn attempt(block: impl FnOnce() -> Result<(), Failure>) {
    if let Err(error) = block() {
        print!("{} ", error.exception_type());
    }
}

/// `index`: ChoixRef.
pub fn choix_case(values: &[f64]) {
    let mut r = Reader::new(values);
    show(choix_ref(r.integer()), |iso| int(iso as i32));
    end();
}

/// `dpuv(4 xyz) eps(4)`: IntImp_ComputeTangence with zeroed outputs.
pub fn tangence_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let dpuv: [Vector3; 4] = [0; 4].map(|_| Vector3::from(r.xyz()));
    let eps: [f64; 4] = [0; 4].map(|_| r.real());
    let (mut tg, mut iso) = ([0.0; 4], [IntImpConstIsoparametric::default(); 4]);
    flag(compute_tangence(&dpuv, &eps, &mut tg, &mut iso));
    tg.iter().for_each(|&t| real(t));
    iso.iter().for_each(|&i| int(i as i32));
    end();
}

fn params(r: &mut Reader) -> Array1<'static, f64> {
    let mut param: Array1<'static, f64> = Array1::with_bounds(1, 4).expect("probe params");
    for i in 1..=4 {
        param[i] = r.real();
    }
    param
}

fn zerpar_report(
    f: &mut IntImpZerParFunc<HSurfaceTool>,
    iso: i32,
    param: &Array1<'_, f64>,
) -> Result<(), Failure> {
    let zero3 = || Vector::filled(1, 3, 0.0);
    let (mut uv, mut inf, mut sup, mut tol) = (zero3()?, zero3()?, zero3()?, zero3()?);
    f.compute_parameters(
        ISOS[iso as usize],
        param,
        &mut uv,
        [&mut inf, &mut sup, &mut tol],
    );
    [&uv, &inf, &sup, &tol].into_iter().for_each(vector);
    let mut fv = zero3()?;
    let mut d = Matrix::filled(1, 3, 1, 3, 0.0).expect("probe matrix");
    flag(f.value(&uv, &mut fv)?);
    vector(&fv);
    real(f.root());
    let mut x2 = zero3()?;
    for i in 1..=3 {
        x2[i] = uv[i] + 0.01 * f64::from(i);
    }
    flag(f.values(&x2, &mut fv, &mut d)?);
    vector(&fv);
    matrix(&d);
    let mut d2 = Matrix::filled(1, 3, 1, 3, 0.0).expect("probe matrix");
    flag(f.derivatives(&uv, &mut d2)?);
    matrix(&d2);
    real(f.root());
    pnt(f.point());
    let mut out: Array1<'static, f64> = Array1::with_bounds(1, 4).expect("probe params");
    out.init(0.0);
    let mut best = IntImpConstIsoparametric::UIsoparametricOnCaro1;
    flag(f.is_tangent(&uv, &mut out, &mut best));
    (1..=4).for_each(|i| real(out[i]));
    int(best as i32);
    show(f.direction(), |d| xyz(d.xyz()));
    show(f.direction_on_s1(), dir2d);
    show(f.direction_on_s2(), dir2d);
    Ok(())
}

/// `s1 s2 iso u1 v1 u2 v2`: IntWalk_TheFunctionOfTheInt2S's template.
pub fn zerpar_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (s1, s2) = (surface(&mut r), surface(&mut r));
    let iso = r.integer();
    let param = params(&mut r);
    attempt(|| {
        let mut f = IntImpZerParFunc::<HSurfaceTool>::new(&s1, &s2)?;
        zerpar_report(&mut f, iso, &param)
    });
    end();
}

fn int2s_report(s: &IntImpInt2S<HSurfaceTool>) {
    flag(s.is_done());
    show(s.is_empty(), flag);
    show(s.point(), |p| {
        pnt(p.value());
        let (a, b, c, d) = p.parameters();
        [a, b, c, d].into_iter().for_each(real);
    });
    show(s.is_tangent(), flag);
    show(s.direction(), |d| xyz(d.xyz()));
    show(s.direction_on_s1(), dir2d);
    show(s.direction_on_s2(), dir2d);
}

/// `s1 s2 tol mode iso u1 v1 u2 v2`: IntWalk_TheInt2S's template.
pub fn int2s_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (s1, s2) = (surface(&mut r), surface(&mut r));
    let tol = r.real();
    let (mode, iso) = (r.integer(), r.integer());
    let param = params(&mut r);
    attempt(|| {
        if mode == 0 {
            let s = IntImpInt2S::<HSurfaceTool>::with_start(&param, &s1, &s2, tol)?;
            int2s_report(&s);
            return Ok(());
        }
        let mut s = IntImpInt2S::<HSurfaceTool>::new(&s1, &s2, tol)?;
        let mut rs = FunctionSetRoot::without_tolerance(s.function(), 15)?;
        int(s.perform_iso(&param, &mut rs, ISOS[iso as usize])? as i32);
        int2s_report(&s);
        Ok(())
    });
    end();
}

type SurfFunction = IntImpZerImpFunc<HSurfaceTool, IntSurfQuadricTool>;

fn zerimp_report(f: &mut SurfFunction, tol: f64, x: &Vector<'_>) -> Result<(), Failure> {
    f.set_tolerance(tol);
    real(f.tolerance());
    let mut fv = Vector::filled(1, 1, 0.0)?;
    let mut d = Matrix::filled(1, 1, 1, 2, 0.0).expect("probe matrix");
    flag(f.value(x, &mut fv)?);
    real(fv[1]);
    real(f.root());
    pnt(f.point());
    show(f.is_tangent(), flag);
    flag(f.derivatives(x, &mut d)?);
    matrix(&d);
    show(f.is_tangent(), flag);
    let mut x2 = Vector::filled(1, 2, 0.0)?;
    (x2[1], x2[2]) = (x[1] + 0.02, x[2] - 0.01);
    flag(f.values(&x2, &mut fv, &mut d)?);
    real(fv[1]);
    matrix(&d);
    show(f.is_tangent(), flag);
    show(f.direction3d(), |v| xyz(*v.xyz()));
    show(f.direction2d(), dir2d);
    pnt(f.point());
    Ok(())
}

/// `surface quadric tol u v`: IntPatch_TheSurfFunction's template.
pub fn zerimp_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = surface(&mut r);
    let q = quadric(&mut r);
    let tol = r.real();
    let mut x = Vector::filled(1, 2, 0.0).expect("probe point");
    (x[1], x[2]) = (r.real(), r.real());
    attempt(|| zerimp_report(&mut SurfFunction::with_surfaces(&s, &q), tol, &x));
    end();
}

/// `surface curve u v w tol margin`: IntCurveSurface_TheExactHInter's template.
pub fn intcs_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = surface(&mut r);
    let c = curve(&mut r);
    let (u, v, w, tol, margin) = (r.real(), r.real(), r.real(), r.real(), r.real());
    attempt(|| {
        let f = IntImpZerCSParFunc::<HSurfaceTool, HCurveTool>::new(&s, &c);
        let mut e = IntImpIntCS::new([u, v, w], &f, tol, margin)?;
        flag(e.is_done());
        show(e.is_empty(), flag);
        show(e.point(), pnt);
        show(e.parameter_on_surface(), |(a, b)| {
            real(a);
            real(b);
        });
        show(e.parameter_on_curve(), real);
        real(e.function().root());
        Ok(())
    });
    end();
}
