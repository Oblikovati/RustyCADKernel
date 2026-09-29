//! LocalAnalysis probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_localanalysis.cxx.
use super::common::{Reader, end, flag, int, real, show};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{
    GeomBezierCurve, GeomCircle, GeomCurve, GeomCylindricalSurface, GeomLine, GeomPlane,
    GeomSphericalSurface, GeomSurface, GeomToroidalSurface,
};
use rustycad_tkgeomalgo::local_analysis::*;
use rustycad_tkgeombase::geom_lprop::SLProps;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis2, Axis3, Direction2, Point3};

const SHAPES: [Shape; 7] = [
    Shape::C0,
    Shape::G1,
    Shape::C1,
    Shape::G2,
    Shape::C2,
    Shape::C3,
    Shape::CN,
];

fn curve(r: &mut Reader) -> Result<Handle<dyn GeomCurve>, Failure> {
    match r.integer() {
        0 => {
            let (c, n) = (r.pnt(), r.dir());
            Ok(Handle::new(GeomCircle::new(Axis2::new(c, n)?, r.real())?).cast())
        }
        1 => {
            let p = r.pnt();
            Ok(Handle::new(GeomLine::from_point(p, r.dir())).cast())
        }
        _ => {
            let mut poles: Array1<'static, Point3> =
                Array1::with_bounds(1, 4).expect("probe poles");
            for i in 1..=4 {
                poles[i] = r.pnt();
            }
            Ok(Handle::new(GeomBezierCurve::new(&poles)?).cast())
        }
    }
}

fn surface(r: &mut Reader) -> Result<Handle<dyn GeomSurface>, Failure> {
    let kind = r.integer();
    let (p, d) = (r.pnt(), r.dir());
    let (r1, r2) = (r.real(), r.real());
    let ax = Axis3::from_normal(p, d)?;
    Ok(match kind {
        0 => Handle::new(GeomPlane::new(ax)).cast(),
        1 => Handle::new(GeomCylindricalSurface::new(ax, r1)?).cast(),
        2 => Handle::new(GeomSphericalSurface::new(ax, r1)?).cast(),
        _ => Handle::new(GeomToroidalSurface::new(ax, r1, r2)?).cast(),
    })
}

/// The ga_la_dump token: the report with `|` for line breaks and `_` for spaces.
fn dump(write: impl FnOnce(&mut Vec<u8>) -> std::io::Result<()>) {
    let mut text: Vec<u8> = Vec::new();
    write(&mut text).expect("probe dump");
    let s: String = String::from_utf8_lossy(&text)
        .replace('\n', "|")
        .replace(' ', "_");
    print!("\"{s}\" ");
}

fn curve_report(c: &LocalAnalysisCurveContinuity) {
    flag(c.is_done());
    if !c.is_done() {
        int(c.status_error().map_or(-1, |s| s as i32));
    }
    show(c.continuity_status(), |s| int(s as i32));
    for value in [
        c.c0_value(),
        c.c1_angle(),
        c.c1_ratio(),
        c.c2_angle(),
        c.c2_ratio(),
        c.g1_angle(),
        c.g2_angle(),
        c.g2_curvature_variation(),
    ] {
        show(value, real);
    }
    for test in [c.is_c0(), c.is_c1(), c.is_c2(), c.is_g1(), c.is_g2()] {
        show(test, flag);
    }
    dump(|o| dump_curve(c, o));
}

/// `curve1 u1 curve2 u2 order eps*8`: every query of LocalAnalysis_CurveContinuity.
pub fn curve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let run = |r: &mut Reader| -> Result<(), Failure> {
        let c1 = curve(r)?;
        let u1 = r.real();
        let c2 = curve(r)?;
        let u2 = r.real();
        let order = SHAPES[r.integer() as usize];
        let e: Vec<f64> = (0..8).map(|_| r.real()).collect();
        let eps = CurveContinuityTolerances {
            eps_nul: e[0],
            eps_c0: e[1],
            eps_c1: e[2],
            eps_c2: e[3],
            eps_g1: e[4],
            eps_g2: e[5],
            percent: e[6],
            maxlen: e[7],
        };
        curve_report(&LocalAnalysisCurveContinuity::new(
            &c1, u1, &c2, u2, order, &eps,
        )?);
        Ok(())
    };
    if let Err(error) = run(&mut r) {
        print!("{} ", error.exception_type());
    }
    end();
}

fn surface_report(c: &LocalAnalysisSurfaceContinuity) {
    flag(c.is_done());
    if !c.is_done() {
        int(c.status_error().map_or(-1, |s| s as i32));
    }
    show(c.continuity_status(), |s| int(s as i32));
    let values = [
        c.c0_value(),
        c.c1u_angle(),
        c.c1u_ratio(),
        c.c1v_angle(),
        c.c1v_ratio(),
        c.c2u_angle(),
        c.c2u_ratio(),
        c.c2v_angle(),
        c.c2v_ratio(),
        c.g1_angle(),
        c.g2_curvature_gap(),
    ];
    for value in values {
        show(value, real);
    }
    for test in [c.is_c0(), c.is_c1(), c.is_c2(), c.is_g1(), c.is_g2()] {
        show(test, flag);
    }
    dump(|o| dump_surface(c, o));
}

fn surface_analysis(r: &mut Reader, mode: i32) -> Result<LocalAnalysisSurfaceContinuity, Failure> {
    let (s1, s2) = (surface(r)?, surface(r)?);
    let order = SHAPES[r.integer() as usize];
    let e: Vec<f64> = (0..7).map(|_| r.real()).collect();
    let eps = SurfaceContinuityTolerances {
        eps_nul: e[0],
        eps_c0: e[1],
        eps_c1: e[2],
        eps_c2: e[3],
        eps_g1: e[4],
        percent: e[5],
        maxlen: e[6],
    };
    match mode {
        0 => {
            let (u1, v1, u2, v2) = (r.real(), r.real(), r.real(), r.real());
            LocalAnalysisSurfaceContinuity::new(&s1, (u1, v1), &s2, (u2, v2), order, &eps)
        }
        1 => {
            let (p1, dx, dy) = (r.pnt2d(), r.real(), r.real());
            let (p2, ex, ey) = (r.pnt2d(), r.real(), r.real());
            let u = r.real();
            let l1: Handle<dyn Geom2dCurve> =
                Handle::new(Geom2dLine::from_point(p1, Direction2::new(dx, dy)?)).cast();
            let l2: Handle<dyn Geom2dCurve> =
                Handle::new(Geom2dLine::from_point(p2, Direction2::new(ex, ey)?)).cast();
            LocalAnalysisSurfaceContinuity::on_curves((&l1, &l2), u, &s1, &s2, order, &eps)
        }
        _ => {
            let (u1, v1, u2, v2) = (r.real(), r.real(), r.real(), r.real());
            let mut c = LocalAnalysisSurfaceContinuity::with_tolerances(&eps);
            let mut a = SLProps::with_parameters(s1, u1, v1, 2, e[0])?;
            let mut b = SLProps::with_parameters(s2, u2, v2, 2, e[0])?;
            c.compute_analysis(&mut a, &mut b, order)?;
            Ok(c)
        }
    }
}

/// `mode surface1 surface2 order eps*7 params`: LocalAnalysis_SurfaceContinuity.
pub fn surface_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mode = r.integer();
    match surface_analysis(&mut r, mode) {
        Ok(c) => surface_report(&c),
        Err(error) => print!("{} ", error.exception_type()),
    }
    end();
}
