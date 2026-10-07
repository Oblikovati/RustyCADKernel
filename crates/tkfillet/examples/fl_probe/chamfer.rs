//! The chamfer functions of BlendFunc and their inverses (`fl_chamfer`,
//! `fl_chamfer_inv`); the twin of the chamfer cases of
//! tools/oracle/tkfillet_probe_blendfunc.cxx.
use super::blend_func::{curve2d, guide, inverse, read_x, surface, to_vector};
use super::common::{Reader, attempt, dir, end, pnt, real};
use super::fillet::blend_members;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::blend::{BlendFuncInv, BlendFunction};
use rustycad_tkfillet::blend_func::{
    ChAsym, ChAsymInv, ChamfInv, Chamfer, ConstThroat, ConstThroatInv, ConstThroatWithPenetration,
    ConstThroatWithPenetrationInv, GenChamfInvFunction, GenChamferFunction,
};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkmath::gp::Line3;
use rustycad_tkmath::math::Vector;

/// The fields after the surfaces and the guide of `fl_chamfer`.
struct ChamferRequest {
    kind: i32,
    d1: f64,
    d2: f64,
    choix: i32,
    t: f64,
    trim: Option<(f64, f64)>,
    x: [f64; 4],
    tol: f64,
}

fn print_line((pdeb, pfin, line): (f64, f64, Line3)) -> Result<(), Failure> {
    real(pdeb);
    real(pfin);
    pnt(line.location());
    dir(line.direction());
    Ok(())
}

/// `fl_chamfer_run`: the interval, the parameter and the shared members.
fn run<F: BlendFunction>(
    f: &mut F,
    request: &ChamferRequest,
    line: impl FnOnce(&mut F) -> Result<(f64, f64, Line3), Failure>,
) {
    if let Some((first, last)) = request.trim {
        attempt(|| f.set_interval(first, last));
    }
    let _ = f.set_param(request.t);
    blend_members(f, request.x, request.t, request.tol, |f| {
        print_line(line(f)?)
    });
}

fn run_gen<F: GenChamferFunction>(mut f: F, request: &ChamferRequest) {
    f.set_distances(request.d1, request.d2, request.choix);
    let uv = request.x;
    run(&mut f, request, |f| f.section_line(uv));
}

/// `fl_chamfer S1 S2 C kind d1 d2 choix t trim first last X1..X4 tol`.
pub fn chamfer_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (s1, s2, c) = (surface(&mut r), surface(&mut r), guide(&mut r));
    let (kind, d1, d2, choix, t) = (r.integer(), r.real(), r.real(), r.integer(), r.real());
    let (trim, first, last) = (r.flag(), r.real(), r.real());
    let (x, tol) = (read_x(&mut r), r.real());
    let trim = trim.then_some((first, last));
    let request = ChamferRequest {
        kind,
        d1,
        d2,
        choix,
        t,
        trim,
        x,
        tol,
    };
    match request.kind {
        3 => {
            let mut f = ChAsym::new(&s1, &s2, &c).expect("function");
            f.set_dist_angle(d1, d2, choix);
            run(&mut f, &request, |f| f.section_line(x));
        }
        0 => run_gen(Chamfer::new(&s1, &s2, &c), &request),
        1 => run_gen(ConstThroat::new(&s1, &s2, &c), &request),
        _ => run_gen(ConstThroatWithPenetration::new(&s1, &s2, &c), &request),
    }
    end();
}

fn with_distances<F: GenChamfInvFunction>(mut f: F, (d1, d2, choix): (f64, f64, i32)) -> F {
    f.set_distances(d1, d2, choix);
    f
}

/// `Set(OnFirst, COnSurf)`, then `fl_inverse`.
fn invert<F: BlendFuncInv>(
    mut f: F,
    (first, cs): (bool, &Handle<dyn Adaptor2dCurve2d>),
    x: &Vector<'_>,
    tol: f64,
) {
    let _ = f.set_curve(first, cs);
    inverse(&mut f, x, tol);
}

/// `fl_chamfer_inv S1 S2 C P2d D2d kind d1 d2 choix first X1..X4 tol`.
pub fn chamfer_inv_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (s1, s2, c) = (surface(&mut r), surface(&mut r), guide(&mut r));
    let cs = curve2d(&mut r);
    let (kind, d1, d2, choix, first) = (r.integer(), r.real(), r.real(), r.integer(), r.flag());
    let (x, tol) = (to_vector(read_x(&mut r)), r.real());
    let (set, on) = ((d1, d2, choix), (first, &cs));
    match kind {
        3 => {
            let mut f = ChAsymInv::new(&s1, &s2, &c).expect("function");
            f.set_dist_angle(d1, d2, choix);
            invert(f, on, &x, tol);
        }
        0 => invert(
            with_distances(ChamfInv::new(&s1, &s2, &c), set),
            on,
            &x,
            tol,
        ),
        1 => invert(
            with_distances(ConstThroatInv::new(&s1, &s2, &c), set),
            on,
            &x,
            tol,
        ),
        _ => invert(
            with_distances(ConstThroatWithPenetrationInv::new(&s1, &s2, &c), set),
            on,
            &x,
            tol,
        ),
    }
    end();
}
