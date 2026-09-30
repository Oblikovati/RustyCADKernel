//! HelixGeom probe operations of the builders, the Rust twin of the builder part of
//! tools/oracle/tkhelix_probe_helixgeom.cxx.
use super::common::{Reader, attempt, dir, end, flag, int, pnt, real};
use super::helix_geom::{curve, helix_fields, shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkhelix::helix_geom::{
    HelixGeomBuilderHelix, HelixGeomBuilderHelixCoil, HelixGeomBuilderHelixGen,
};
use rustycad_tkmath::gp::Axis2;

/// `hx_builder_state`: statuses, settings, reached tolerance and curves.
fn builder_state(b: &dyn HelixGeomBuilderHelixGen) -> Result<(), Failure> {
    let (cont, deg, seg) = b.approx_parameters();
    let p = b.curve_parameters();
    int(b.error_status());
    int(b.warning_status());
    int(cont as i32);
    int(deg);
    int(seg);
    real(b.tolerance());
    [p.t1, p.t2, p.pitch, p.r_start, p.taper_angle]
        .into_iter()
        .for_each(real);
    flag(p.is_cw);
    real(b.tolerance_reached());
    int(b.curves().length());
    for c in b.curves().iter() {
        curve(c)?;
    }
    Ok(())
}

/// `hx_builder_run`: `set tol cont maxdeg maxseg helix`, then Perform and the state.
fn builder_run(r: &mut Reader<'_>, b: &mut dyn HelixGeomBuilderHelixGen) {
    let (set, tol, cont, deg, seg) = (r.flag(), r.real(), r.integer(), r.integer(), r.integer());
    let f = helix_fields(r);
    if set {
        b.set_tolerance(tol);
        b.set_approx_parameters(shape(cont), deg, seg);
        b.set_curve_parameters(f.t1, f.t2, f.pitch, f.r_start, f.taper, f.cw);
    }
    attempt(|| b.perform());
    attempt(|| builder_state(b));
}

/// `nruns (run)...`: HelixGeom_BuilderHelixCoil.
pub fn coil_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut b = HelixGeomBuilderHelixCoil::new();
    attempt(|| builder_state(&b));
    for _ in 0..r.integer() {
        builder_run(&mut r, &mut b);
    }
    end();
}

/// `P N X nruns (run)...`: HelixGeom_BuilderHelix at gp_Ax2(P, N, X).
pub fn helix_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (o, n, x) = (r.pnt(), r.dir(), r.dir());
    let mut b = HelixGeomBuilderHelix::new();
    attempt(|| builder_state(&b));
    // The corpus only spells valid frames; gp_Ax2 would raise for a parallel X.
    b.set_position(Axis2::with_x_direction(o, n, x).expect("probe frame"));
    pnt(b.position().location());
    dir(b.position().direction());
    dir(b.position().x_direction());
    for _ in 0..r.integer() {
        builder_run(&mut r, &mut b);
    }
    end();
}
