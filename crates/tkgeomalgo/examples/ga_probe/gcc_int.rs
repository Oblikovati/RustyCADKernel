//! GccInt probe operations, the Rust twin of tools/oracle/tkgeomalgo_probe_gccint.cxx.
use super::common::{Reader, end, flag, int, pnt2d, show};
use super::gcc_ent::{circ2d, elips2d, hypr2d, lin2d, parab2d};
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkgeomalgo::gcc_int::{BCirc, BElips, BHyper, BLine, BParab, Bisec};
use rustycad_tkmath::gp::{Axis22d, Circle2, Ellipse2, Hyperbola2, Line2, Parabola2};

// Every accessor of a bisector: its geometry or the raised exception type.
fn accessors(bisec: &Handle<dyn Bisec>) {
    int(bisec.arc_type() as i32);
    print!("{} ", bisec.dynamic_type().expect("bisector").name());
    flag(bisec.is_kind(<dyn Bisec as RuntimeTyped>::static_type()));
    show(bisec.point(), pnt2d);
    show(bisec.line(), lin2d);
    show(bisec.circle(), circ2d);
    show(bisec.hyperbola(), hypr2d);
    show(bisec.parabola(), parab2d);
    show(bisec.ellipse(), elips2d);
}

/// `kind x y dx dy a b`: 0 line, 1 circle (radius a), 2 ellipse (radii a b), 3 hyperbola
/// (radii a b), 4 parabola (focal a), built on the frame (x y, dx dy), direct sense.
pub fn bisec_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let (p, d) = (r.pnt2d(), r.dir2d());
    let (a, b) = (r.real(), r.real());
    let frame = Axis22d::new(p, d, true).expect("probe frame");
    let bisec: Handle<dyn Bisec> = match kind {
        0 => Handle::new(BLine::new(Line2::new(p, d))).cast(),
        1 => Handle::new(BCirc::new(Circle2::new(frame, a).expect("circle"))).cast(),
        2 => Handle::new(BElips::new(Ellipse2::new(frame, a, b).expect("ellipse"))).cast(),
        3 => Handle::new(BHyper::new(
            Hyperbola2::new(frame, a, b).expect("hyperbola"),
        ))
        .cast(),
        _ => Handle::new(BParab::new(Parabola2::new(frame, a).expect("parabola"))).cast(),
    };
    accessors(&bisec);
    end();
}
