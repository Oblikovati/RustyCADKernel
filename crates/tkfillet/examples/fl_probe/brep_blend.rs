//! The BRepBlend probes, the Rust twin of tools/oracle/tkfillet_probe_brepblend.cxx.
use super::blend_func::{curve2d, guide, inverse, surface};
use super::common::{Reader, attempt, end, flag, int, real};
use super::fillet::law;
use rustycad_tkernel::standard::Handle;
use rustycad_tkfillet::blend::{
    BlendCurvPointFuncInv, BlendSurfCurvFuncInv, BlendSurfPointFuncInv,
};
use rustycad_tkfillet::brep_blend::{
    BRepBlendBlendTool, BRepBlendCurvPointRadInv, BRepBlendHCurve2dTool, BRepBlendHCurveTool,
    BRepBlendSurfCurvConstRadInv, BRepBlendSurfCurvEvolRadInv, BRepBlendSurfPointConstRadInv,
    BRepBlendSurfPointEvolRadInv,
};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::gp::{Axis2d, Circle2, Direction2, Point2};
use rustycad_tkmath::math::Vector;

/// `fl_curve2d_kind`: 5 numbers.
fn curve2d_kind(r: &mut Reader<'_>) -> Handle<dyn Adaptor2dCurve2d> {
    let kind = r.integer();
    let (a, b, c, d) = (r.real(), r.real(), r.real(), r.real());
    let curve: Handle<dyn Geom2dCurve> = if kind == 1 {
        let axis = Axis2d::new(
            Point2::new(a, b),
            Direction2::new(1., 0.).expect("direction"),
        );
        Handle::new(Geom2dCircle::from_circ2d(
            Circle2::from_x_axis(axis, c, true).expect("circle"),
        ))
        .cast()
    } else {
        let direction = Direction2::new(c, d).expect("direction");
        Handle::new(Geom2dLine::from_point(Point2::new(a, b), direction)).cast()
    };
    Handle::new(Geom2dAdaptorCurve::with_curve(&curve).expect("adaptor")).cast()
}

fn vector_of(values: &[f64]) -> Vector<'static> {
    let mut x = Vector::filled(1, values.len() as i32, 0.0).expect("vector");
    for (i, value) in (1..).zip(values) {
        x[i] = *value;
    }
    x
}

/// `fl_rad_inv kind S C C2 P2d D2d Law radius choix P X1..X3 tol`.
pub fn rad_inv_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let (s, c, c2) = (surface(&mut r), guide(&mut r), guide(&mut r));
    let (cs, evol) = (curve2d(&mut r), law(&mut r));
    let (radius, choix, p) = (r.real(), r.integer(), r.pnt());
    let x = read_x3(&mut r);
    let tol = r.real();
    let x3 = vector_of(&x);
    match kind {
        0 => {
            let mut f = BRepBlendSurfPointConstRadInv::new(&s, &c);
            f.set_radius(radius, choix);
            f.set_point(p);
            inverse(&mut f, &x3, tol);
        }
        1 => {
            let mut f = BRepBlendSurfPointEvolRadInv::new(&s, &c, &evol);
            f.set_choix(choix);
            f.set_point(p);
            inverse(&mut f, &x3, tol);
        }
        2 => {
            let mut f = BRepBlendSurfCurvConstRadInv::new(&s, &c2, &c);
            f.set_radius(radius, choix);
            f.set_rst(&cs);
            inverse(&mut f, &x3, tol);
        }
        3 => {
            let mut f = BRepBlendSurfCurvEvolRadInv::new(&s, &c2, &c, &evol);
            f.set_choix(choix);
            f.set_rst(&cs);
            inverse(&mut f, &x3, tol);
        }
        _ => {
            let mut f = BRepBlendCurvPointRadInv::new(&c, &c2);
            f.set_choix(choix);
            f.set_point(p);
            inverse(&mut f, &vector_of(&x[..2]), tol);
        }
    }
    end();
}

fn read_x3(r: &mut Reader<'_>) -> [f64; 3] {
    [r.real(), r.real(), r.real()]
}

/// `fl_blend_tool C2d P1 P2 C U0 U1`.
pub fn blend_tool_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let arc = curve2d_kind(&mut r);
    let (p1, p2) = (r.pnt2d(), r.pnt2d());
    let c = guide(&mut r);
    let (u0, u1) = (r.real(), r.real());
    let none = Handle::default();
    attempt(|| {
        let (prm, dist) = BRepBlendBlendTool::project(p1, &none, &arc)?;
        flag(true);
        real(prm);
        real(dist);
        Ok(())
    });
    attempt(|| {
        let found = BRepBlendBlendTool::inters(p1, p2, &none, &arc)?;
        flag(found.is_some());
        let (prm, dist) = found.unwrap_or((0., 0.));
        real(prm);
        real(dist);
        Ok(())
    });
    attempt(|| {
        BRepBlendBlendTool::bounds(&arc).map(|(a, b)| {
            real(a);
            real(b);
        })
    });
    attempt(|| BRepBlendHCurve2dTool::nb_samples(&arc, u0, u1).map(int));
    attempt(|| BRepBlendHCurveTool::nb_samples(&c, u0, u1).map(int));
    end();
}
