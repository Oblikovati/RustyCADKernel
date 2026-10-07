//! The line builders of BRepBlend (`fl_surf_rst_walk`, `fl_rst_rst_walk`); the twin of
//! tools/oracle/tkfillet_probe_linebuilders.cxx.
use super::blend_func::surface;
use super::brep_blend::geom_curve2d_kind;
use super::common::{Reader, attempt, end, flag, int, pnt2d, real};
use super::walking::{blend_point, el_spine, extremity, trimmed_surface};
use rustycad_tkernel::standard::Handle;
use rustycad_tkfillet::blend::BlendSurfCurvFuncInv;
use rustycad_tkfillet::blend_func::ConstRadInv;
use rustycad_tkfillet::brep_blend::{
    BRepBlendCurvPointRadInv, BRepBlendLineHandle, BRepBlendRstRstConstRad,
    BRepBlendRstRstLineBuilder, BRepBlendSurfCurvConstRadInv, BRepBlendSurfPointConstRadInv,
    BRepBlendSurfRstConstRad, BRepBlendSurfRstLineBuilder, RstRstInverses, RstRstWalkSettings,
    SurfRstInverses, SurfRstWalkSettings, WalkingDomain,
};
use rustycad_tkfillet::chfi_ds::lock;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{
    Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface, Adaptor3dTopolTool,
};
use rustycad_tkmath::math::Vector;
use std::rc::Rc;

/// `fl_trimmed_curve2d`.
fn trimmed_curve2d(r: &mut Reader<'_>) -> Handle<dyn Adaptor2dCurve2d> {
    let curve = geom_curve2d_kind(r);
    let (first, last) = (r.real(), r.real());
    Handle::new(Geom2dAdaptorCurve::with_range(&curve, first, last).expect("trimmed curve")).cast()
}

/// `fl_builder_line`.
fn builder_line(line: &BRepBlendLineHandle) {
    let line = lock(line);
    int(line.nb_points());
    for i in 1..=line.nb_points() {
        let p = line.point(i).expect("line point");
        blend_point(p);
        attempt(|| p.parameter_on_c1().map(real));
        attempt(|| p.parameter_on_c2().map(real));
    }
    extremity(line.start_point_on_first());
    extremity(line.start_point_on_second());
    extremity(line.end_point_on_first());
    extremity(line.end_point_on_second());
    attempt(|| line.transition_on_s1().map(|t| int(t as i32)));
    attempt(|| line.transition_on_s2().map(|t| int(t as i32)));
}

fn surf_rst_state(b: &BRepBlendSurfRstLineBuilder) {
    flag(b.is_done());
    flag(b.decroch_start());
    flag(b.decroch_end());
    if b.is_done() {
        builder_line(b.line().expect("line"));
    }
}

fn domain(s: &Handle<dyn Adaptor3dSurface>) -> WalkingDomain {
    Rc::new(Adaptor3dTopolTool::with_surface(s).expect("domain"))
}

fn vector_of(values: &[f64]) -> Vector<'static> {
    let mut v = Vector::with_bounds(1, values.len() as i32).expect("vector");
    for (i, x) in (1..).zip(values) {
        v[i] = *x;
    }
    v
}

/// The functions of a surface-restriction marching (those of ChFi3d_FilBuilder).
struct SurfRstFunctions {
    f: BRepBlendSurfRstConstRad,
    finv: ConstRadInv,
    finvp: BRepBlendSurfPointConstRadInv,
    finvc: BRepBlendSurfCurvConstRadInv,
}

impl SurfRstFunctions {
    fn inverses(&mut self) -> (&mut BRepBlendSurfRstConstRad, SurfRstInverses<'_>) {
        let inv = SurfRstInverses {
            finv: &mut self.finv,
            finv_p: &mut self.finvp,
            finv_c: &mut self.finvc,
        };
        (&mut self.f, inv)
    }
}

/// The request after the functions.
struct SurfRstRequest {
    mode: i32,
    range: (f64, f64),
    settings: SurfRstWalkSettings,
    appro: bool,
    x: [f64; 3],
    rec: (bool, bool, bool),
    pextra: f64,
}

/// `fl_surf_rst_walk`: see tools/oracle/tkfillet_probe_linebuilders.cxx.
pub fn surf_rst_walk_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let mode = r.integer();
    let (s1, s2) = (trimmed_surface(r), trimmed_surface(r));
    let rst = trimmed_curve2d(r);
    let (sref, pcref) = (surface(r), trimmed_curve2d(r));
    let spine = el_spine(r);
    let (radius, choix, choixinv) = (r.real(), r.integer(), r.integer());
    let range = (r.real(), r.real());
    let settings = SurfRstWalkSettings {
        max_step: r.real(),
        tol3d: r.real(),
        tol2d: r.real(),
        tol_guide: r.real(),
        fleche: r.real(),
    };
    let appro = r.flag();
    let x = [r.real(), r.real(), r.real()];
    let rec = (r.flag(), r.flag(), r.flag());
    let q = SurfRstRequest {
        mode,
        range,
        settings,
        appro,
        x,
        rec,
        pextra: r.real(),
    };
    let guide: Handle<dyn Adaptor3dCurve> = spine.cast();
    let mut f = BRepBlendSurfRstConstRad::new((&s1, &s2, &rst), &guide).expect("function");
    f.set_refs(&sref, &pcref);
    f.set_radius(radius, choix);
    let hc: Handle<dyn Adaptor3dCurve> =
        Handle::new(Adaptor3dCurveOnSurface::with_curve(&rst, &s2).expect("curve on surface"))
            .cast();
    let mut finvc = BRepBlendSurfCurvConstRadInv::new(&s1, &hc, &guide);
    finvc.set_radius(radius, choix);
    let mut finvp = BRepBlendSurfPointConstRadInv::new(&s1, &guide);
    finvp.set_radius(radius, choix);
    let mut finv = ConstRadInv::new(&s1, &sref, &guide);
    attempt(|| rustycad_tkfillet::blend::BlendFuncInv::set_curve(&mut finv, false, &pcref));
    finv.set_radius(radius, choixinv);
    let mut fs = SurfRstFunctions {
        f,
        finv,
        finvp,
        finvc,
    };
    let mut b = BRepBlendSurfRstLineBuilder::new(&s1, &domain(&s1), &s2, &rst, &domain(&s2));
    attempt(|| b.line().map(|_| ()));
    run_surf_rst(&mut b, &mut fs, &q);
    end();
}

fn run_surf_rst(
    b: &mut BRepBlendSurfRstLineBuilder,
    fs: &mut SurfRstFunctions,
    q: &SurfRstRequest,
) {
    if q.mode == 3 {
        let mut found = None;
        attempt(|| {
            let (f, inv) = fs.inverses();
            let first =
                b.perform_first_section(f, inv, q.range, &vector_of(&q.x), q.settings, q.rec)?;
            flag(first.is_some());
            let (psol, sol) = first.unwrap_or((0.0, [0.0; 3]));
            real(psol);
            sol.into_iter().for_each(real);
            found = first;
            Ok(())
        });
        surf_rst_state(b);
        if let Some((psol, sol)) = found {
            let (f, inv) = fs.inverses();
            attempt(|| {
                b.perform(
                    f,
                    inv,
                    (psol, q.pextra),
                    q.settings,
                    &vector_of(&sol),
                    false,
                )
            });
        }
        surf_rst_state(b);
        return;
    }
    let (f, inv) = fs.inverses();
    attempt(|| b.perform(f, inv, q.range, q.settings, &vector_of(&q.x), q.appro));
    surf_rst_state(b);
    if q.mode == 1 {
        let (f, inv) = fs.inverses();
        attempt(|| b.complete(f, inv, q.pextra).map(flag));
        surf_rst_state(b);
    }
    if q.mode == 2 && b.is_done() {
        arc_to_recadre(b, q.pextra);
    }
}

/// Mode 2: ArcToRecadre at the last point moved by `shift`.
fn arc_to_recadre(b: &BRepBlendSurfRstLineBuilder, shift: f64) {
    let p = {
        let line = lock(b.line().expect("line"));
        *line.point(line.nb_points()).expect("last point")
    };
    let (u, v) = p.parameters_on_s().expect("parameters");
    let s = vector_of(&[u + shift, v + shift, p.parameter_on_c().expect("parameter")]);
    let mut pon = 0.0;
    attempt(|| {
        let (index, a, c) = b.arc_to_recadre(&s, 0, &mut pon)?;
        int(index);
        pnt2d(a);
        pnt2d(c);
        real(pon);
        Ok(())
    });
}

fn rst_rst_state(b: &BRepBlendRstRstLineBuilder) {
    flag(b.is_done());
    [
        b.decroch1_start(),
        b.decroch2_start(),
        b.decroch1_end(),
        b.decroch2_end(),
    ]
    .into_iter()
    .for_each(flag);
    if b.is_done() {
        builder_line(b.line().expect("line"));
    }
}

/// The functions of a restriction-restriction marching (those of ChFi3d_FilBuilder).
struct RstRstFunctions {
    f: BRepBlendRstRstConstRad,
    finv: [BRepBlendSurfCurvConstRadInv; 2],
    finvp: [BRepBlendCurvPointRadInv; 2],
}

impl RstRstFunctions {
    fn inverses(&mut self) -> (&mut BRepBlendRstRstConstRad, RstRstInverses<'_>) {
        let [finv1, finv2] = &mut self.finv;
        let [finvp1, finvp2] = &mut self.finvp;
        let inv = RstRstInverses {
            finv: [finv1, finv2],
            finv_p: [finvp1, finvp2],
        };
        (&mut self.f, inv)
    }
}

/// The request after the functions.
struct RstRstRequest {
    mode: i32,
    range: (f64, f64),
    settings: RstRstWalkSettings,
    appro: bool,
    x: [f64; 2],
    rec: [(bool, bool); 2],
    pextra: f64,
}

/// A trimmed surface and its restriction.
fn restriction(r: &mut Reader<'_>) -> (Handle<dyn Adaptor3dSurface>, Handle<dyn Adaptor2dCurve2d>) {
    let s = trimmed_surface(r);
    (s, trimmed_curve2d(r))
}

/// A reference surface and its curve.
fn reference(r: &mut Reader<'_>) -> (Handle<dyn Adaptor3dSurface>, Handle<dyn Adaptor2dCurve2d>) {
    let s = surface(r);
    (s, trimmed_curve2d(r))
}

fn curve_on(
    rst: &Handle<dyn Adaptor2dCurve2d>,
    s: &Handle<dyn Adaptor3dSurface>,
) -> Handle<dyn Adaptor3dCurve> {
    Handle::new(Adaptor3dCurveOnSurface::with_curve(rst, s).expect("curve on surface")).cast()
}

/// `fl_rst_rst_walk`: see tools/oracle/tkfillet_probe_linebuilders.cxx.
pub fn rst_rst_walk_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let mode = r.integer();
    let ((s1, rst1), (s2, rst2)) = (restriction(r), restriction(r));
    let ((sref1, pcref1), (sref2, pcref2)) = (reference(r), reference(r));
    let spine = el_spine(r);
    let (radius, choix, ch1, ch2) = (r.real(), r.integer(), r.integer(), r.integer());
    let range = (r.real(), r.real());
    let settings = RstRstWalkSettings {
        max_step: r.real(),
        tol3d: r.real(),
        tol_guide: r.real(),
        fleche: r.real(),
    };
    let appro = r.flag();
    let x = [r.real(), r.real()];
    let rec = [(r.flag(), r.flag()), (r.flag(), r.flag())];
    let q = RstRstRequest {
        mode,
        range,
        settings,
        appro,
        x,
        rec,
        pextra: r.real(),
    };
    let guide: Handle<dyn Adaptor3dCurve> = spine.cast();
    let mut f = BRepBlendRstRstConstRad::new((&s1, &rst1, &s2, &rst2), &guide).expect("function");
    f.set_refs((&sref1, &pcref1, &sref2, &pcref2));
    f.set_radius(radius, choix);
    let (hc1, hc2) = (curve_on(&rst1, &s1), curve_on(&rst2, &s2));
    let mut finv1 = BRepBlendSurfCurvConstRadInv::new(&sref1, &hc2, &guide);
    let mut finvp1 = BRepBlendCurvPointRadInv::new(&guide, &hc2);
    let mut finv2 = BRepBlendSurfCurvConstRadInv::new(&sref2, &hc1, &guide);
    let mut finvp2 = BRepBlendCurvPointRadInv::new(&guide, &hc1);
    finv1.set_rst(&pcref1);
    finv2.set_rst(&pcref2);
    finv1.set_radius(radius, ch1);
    finvp1.set_choix(choix);
    finv2.set_radius(radius, ch2);
    finvp2.set_choix(choix);
    let mut fs = RstRstFunctions {
        f,
        finv: [finv1, finv2],
        finvp: [finvp1, finvp2],
    };
    let mut b =
        BRepBlendRstRstLineBuilder::new((&s1, &rst1, &domain(&s1)), (&s2, &rst2, &domain(&s2)));
    attempt(|| b.line().map(|_| ()));
    run_rst_rst(&mut b, &mut fs, &q);
    end();
}

fn run_rst_rst(b: &mut BRepBlendRstRstLineBuilder, fs: &mut RstRstFunctions, q: &RstRstRequest) {
    if q.mode == 3 {
        let mut found = None;
        attempt(|| {
            let (f, inv) = fs.inverses();
            let first =
                b.perform_first_section(f, inv, q.range, &vector_of(&q.x), q.settings, q.rec)?;
            flag(first.is_some());
            let (psol, sol) = first.unwrap_or((0.0, [0.0; 2]));
            real(psol);
            sol.into_iter().for_each(real);
            found = first;
            Ok(())
        });
        rst_rst_state(b);
        if let Some((psol, sol)) = found {
            let (f, inv) = fs.inverses();
            attempt(|| {
                b.perform(
                    f,
                    inv,
                    (psol, q.pextra),
                    q.settings,
                    &vector_of(&sol),
                    false,
                )
            });
        }
        rst_rst_state(b);
        return;
    }
    let (f, inv) = fs.inverses();
    attempt(|| b.perform(f, inv, q.range, q.settings, &vector_of(&q.x), q.appro));
    rst_rst_state(b);
    if q.mode == 1 {
        let (f, inv) = fs.inverses();
        attempt(|| b.complete(f, inv, q.pextra).map(flag));
        rst_rst_state(b);
    }
}
