//! IntPatch quadric-parametric intersection probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_intpatch_impprm.cxx: IntPatch_ImpPrmIntersection and
//! IntPatch_RstInt.
use super::common::{Reader, attempt, end, flag, int};
use super::ics_geometry::surface;
use super::int_patch_impimp::read_pnt2s;
use super::int_patch_report::{Domains, line, point};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkgeomalgo::int_patch::{
    IntPatchImpPrmIntersection, IntPatchLine, IntPatchPoint, IntPatchPointLine, IntPatchRLine,
    IntPatchRstInt, IntPatchWLine,
};
use rustycad_tkgeomalgo::int_start::TheArc;
use rustycad_tkgeomalgo::int_surf::{IntSurfLineOn2S, IntSurfTypeTrans};

/// `ga_ipl_impprm`.
fn impprm(dm: &Domains, x: &IntPatchImpPrmIntersection) {
    flag(x.is_done());
    attempt(|| x.is_empty().map(flag));
    attempt(|| {
        let n = x.nb_pnts()?;
        int(n);
        for i in 0..=n + 1 {
            attempt(|| x.point(i).map(|p| point(dm, p)));
        }
        Ok(())
    });
    attempt(|| {
        let n = x.nb_lines()?;
        int(n);
        for i in 0..=n + 1 {
            attempt(|| x.line(i).map(|l| line(dm, l)));
        }
        Ok(())
    });
}

/// `ga_ip_impprm_case`.
pub fn impprm_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (s1, s2) = (surface(&mut r), surface(&mut r));
        let (tolarc, toltang, fleche, pas) = (r.real(), r.real(), r.real(), r.real());
        let mode: i32 = r.integer();
        let (u, v) = (r.real(), r.real());
        let (d1, d2) = (Adaptor3dTopolTool::with_surface(&s1)?, Adaptor3dTopolTool::with_surface(&s2)?);
        let dm = Domains::new(&d1, &d2);
        impprm(&dm, &IntPatchImpPrmIntersection::new());
        if mode == 0 {
            let x = IntPatchImpPrmIntersection::with_surfaces(&s1, &d1, &s2, &d2, tolarc, toltang, fleche, pas)?;
            impprm(&dm, &x);
            return Ok(());
        }
        let mut x = IntPatchImpPrmIntersection::new();
        if mode == 2 {
            x.set_start_point(u, v);
        }
        attempt(|| x.perform(&s1, &d1, &s2, &d2, tolarc, toltang, fleche, pas));
        impprm(&dm, &x);
        if mode == 3 {
            attempt(|| x.perform(&s1, &d1, &s2, &d2, tolarc, toltang, fleche, pas));
            impprm(&dm, &x);
        }
        Ok(())
    });
    end();
}

/// The arc of rank `rank` (from 1) of the domain, or a null arc.
fn arc_of_rank(d: &Adaptor3dTopolTool, rank: i32) -> Result<TheArc, Failure> {
    let (mut k, mut found) = (1, TheArc::default());
    d.init();
    while d.more() {
        if k == rank {
            found = d.value()?;
        }
        d.next();
        k += 1;
    }
    Ok(found)
}

/// The line of `ga_ip_rstint_case`: walking (kind 0) or restriction on the arc.
fn rst_line(kind: i32, pts: &Handle<IntSurfLineOn2S>, arc: &TheArc, onfirst: bool) -> Handle<dyn IntPatchPointLine> {
    let (tin, tout) = (IntSurfTypeTrans::In, IntSurfTypeTrans::Out);
    if kind == 0 {
        return Handle::new(IntPatchWLine::new(pts, false, tin, tout)).cast();
    }
    let rl = IntPatchRLine::new(false, tin, tout);
    rl.add(pts);
    if !arc.is_null() {
        if onfirst {
            rl.set_arc_on_s2(arc);
        } else {
            rl.set_arc_on_s1(arc);
        }
    }
    Handle::new(rl).cast()
}

/// The end vertices of the line (parameters 1 and `n`), then its first and last points.
fn add_ends(l: &Handle<dyn IntPatchPointLine>, pts: &IntSurfLineOn2S, tol: f64) -> Result<(), Failure> {
    let n: i32 = pts.nb_points();
    for i in [1, n] {
        let q = pts.value(i)?;
        let mut p = IntPatchPoint::new();
        p.set_value_tangent(q.value(), tol, false);
        let (a, b, c, d) = q.parameters();
        p.set_parameters(a, b, c, d);
        p.set_parameter(f64::from(i));
        l.add_vertex(&p, false);
    }
    let w: Handle<IntPatchWLine> = l.cast();
    if let Some(w) = w.get() {
        w.set_first_point(1);
        w.set_last_point(2);
        return Ok(());
    }
    let r: Handle<IntPatchRLine> = l.cast();
    let r: &IntPatchRLine = r.get().expect("restriction line");
    r.set_first_point(1);
    r.set_last_point(2);
    Ok(())
}

/// `ga_ip_rstint_case`.
pub fn rstint_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (s, o) = (surface(&mut r), surface(&mut r));
        let (onfirst, tol) = (r.flag(), r.real());
        let (kind, arc) = (r.integer(), r.integer());
        let (ends, twice) = (r.flag(), r.flag());
        let n: i32 = r.integer();
        let pts = Handle::new(IntSurfLineOn2S::new(None));
        for _ in 0..n {
            pts.add(&read_pnt2s(&mut r));
        }
        let (ds, dother) = (Adaptor3dTopolTool::with_surface(&s)?, Adaptor3dTopolTool::with_surface(&o)?);
        let dm = if onfirst { Domains::new(&ds, &dother) } else { Domains::new(&dother, &ds) };
        let arc: TheArc = if kind == 0 { TheArc::default() } else { arc_of_rank(&dother, arc)? };
        let pl: Handle<dyn IntPatchPointLine> = rst_line(kind, &pts, &arc, onfirst);
        if ends && n > 0 {
            add_ends(&pl, &pts, tol)?;
        }
        let l: Handle<dyn IntPatchLine> = pl.cast();
        attempt(|| IntPatchRstInt::put_vertex_on_line(&l, &s, &ds, &o, onfirst, tol));
        line(&dm, &l);
        if twice {
            attempt(|| IntPatchRstInt::put_vertex_on_line(&l, &s, &ds, &o, onfirst, tol));
            line(&dm, &l);
        }
        Ok(())
    });
    end();
}
