//! IntPatch_WLineTool and IntPatch_LineConstructor probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_intpatch_wltool.cxx: two surfaces, their domains and
//! lines of every kind, run through one algorithm; the lines it gives are printed.
use super::common::{Reader, attempt, end, flag, int, real};
use super::ics_geometry::surface;
use super::int_patch::{Fixture, point, read_point};
use super::int_patch_curves::{aline_curve, make_gline};
use super::int_patch_lines::{
    creating_way, line_base, make_rline, make_wline, points, read_line2s, read_trans, vertices,
};
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkgeomalgo::int_patch::{
    IntPatchALine, IntPatchGLine, IntPatchLine, IntPatchLineConstructor, IntPatchLineTransitions,
    IntPatchPoint, IntPatchPointLine, IntPatchRLine, IntPatchWLine, IntPatchWLineTool,
};
use rustycad_tkgeomalgo::int_start::{TheArc, TheTopolTool, TheVertex};
use rustycad_tkmath::gp::{Point2, Point3};
use std::cell::Cell;

/// GaWtDomain: the domain of the surface whose arcs are given fixture arcs (or its own
/// restrictions), with a 3d representation of tolerance `tol` when `tol >= 0`.
pub struct WtDomain {
    tool: Adaptor3dTopolTool,
    arcs: Option<Vec<TheArc>>,
    idx: Cell<usize>,
    tol: f64,
}

/// `ga_wt_domain`.
pub fn domain(
    r: &mut Reader,
    fx: &Fixture,
    s: &Adaptor3dSurfaceHandle,
) -> Result<WtDomain, Failure> {
    let n = r.integer();
    let arcs: Vec<TheArc> = (0..n.max(0)).map(|_| fx.arc(r.integer())).collect();
    let tol = r.real();
    Ok(WtDomain {
        tool: Adaptor3dTopolTool::with_surface(s)?,
        arcs: (n >= 0).then_some(arcs),
        idx: Cell::new(0),
        tol,
    })
}

impl TheTopolTool for WtDomain {
    fn nb_samples_u(&self) -> Result<i32, Failure> {
        TheTopolTool::nb_samples_u(&self.tool)
    }
    fn nb_samples_v(&self) -> Result<i32, Failure> {
        TheTopolTool::nb_samples_v(&self.tool)
    }
    fn nb_samples(&self) -> Result<i32, Failure> {
        TheTopolTool::nb_samples(&self.tool)
    }
    fn sample_point(&self, index: i32) -> Result<(Point2, Point3), Failure> {
        TheTopolTool::sample_point(&self.tool, index)
    }
    fn classify(&self, point: Point2, tol: f64, reframe: bool) -> Result<State, Failure> {
        TheTopolTool::classify(&self.tool, point, tol, reframe)
    }
    fn init(&self) {
        match &self.arcs {
            None => TheTopolTool::init(&self.tool),
            Some(_) => self.idx.set(0),
        }
    }
    fn more(&self) -> bool {
        match &self.arcs {
            None => TheTopolTool::more(&self.tool),
            Some(arcs) => self.idx.get() < arcs.len(),
        }
    }
    fn value(&self) -> Result<TheArc, Failure> {
        match &self.arcs {
            None => TheTopolTool::value(&self.tool),
            Some(arcs) => Ok(arcs[self.idx.get()].clone()),
        }
    }
    fn next(&self) {
        match &self.arcs {
            None => TheTopolTool::next(&self.tool),
            Some(_) => self.idx.set(self.idx.get() + 1),
        }
    }
    fn initialize_arc(&self, arc: &TheArc) -> Result<(), Failure> {
        self.tool.initialize_arc(arc)
    }
    fn init_vertex_iterator(&self) {
        TheTopolTool::init_vertex_iterator(&self.tool);
    }
    fn more_vertex(&self) -> bool {
        TheTopolTool::more_vertex(&self.tool)
    }
    fn vertex(&self) -> Result<TheVertex, Failure> {
        TheTopolTool::vertex(&self.tool)
    }
    fn next_vertex(&self) {
        TheTopolTool::next_vertex(&self.tool);
    }
    fn identical(&self, first: &TheVertex, second: &TheVertex) -> Result<bool, Failure> {
        TheTopolTool::identical(&self.tool, first, second)
    }
    fn edge(&self) -> Option<Edge> {
        None
    }
    fn is_the_point_on(&self, point: Point2, tol: f64, reframe: bool) -> Result<bool, Failure> {
        TheTopolTool::is_the_point_on(&self.tool, point, tol, reframe)
    }
    fn tol3d(&self, arc: &TheArc) -> Result<f64, Failure> {
        if self.tol < 0.0 {
            return TheTopolTool::tol3d(&self.tool, arc);
        }
        Ok(self.tol)
    }
    fn domain_is_infinite(&self) -> bool {
        TheTopolTool::domain_is_infinite(&self.tool)
    }
    fn sample_pnts(&self, deflection: f64, nu_min: i32, nv_min: i32) -> Result<(), Failure> {
        TheTopolTool::sample_pnts(&self.tool, deflection, nu_min, nv_min)
    }
    fn u_parameters(&self) -> Result<Array1<'static, f64>, Failure> {
        TheTopolTool::u_parameters(&self.tool)
    }
    fn v_parameters(&self) -> Result<Array1<'static, f64>, Failure> {
        TheTopolTool::v_parameters(&self.tool)
    }
    fn is_uniform_sampling(&self) -> Result<bool, Failure> {
        TheTopolTool::is_uniform_sampling(&self.tool)
    }
}

/// Sets the arcs of a point line on both surfaces (none for a rank 0).
type ArcSetter<L> = dyn Fn(&L, Option<TheArc>, Option<TheArc>);

/// The vertices and the arcs (the ranks that are not 0) of a point line.
fn finish_point_line<L: IntPatchPointLine>(
    r: &mut Reader,
    fx: &Fixture,
    l: &L,
    arcs: &ArcSetter<L>,
) {
    for _ in 0..r.integer() {
        l.add_vertex(&read_point(r, fx), false);
    }
    let (a1, a2) = (r.integer(), r.integer());
    let pick = |k: i32| (k != 0).then(|| fx.arc(k));
    arcs(l, pick(a1), pick(a2));
}

fn walking(r: &mut Reader, fx: &Fixture) -> Handle<dyn IntPatchLine> {
    let (tang, t) = read_trans(r);
    let way = r.integer();
    let periods = [r.real(), r.real(), r.real(), r.real()];
    let l = make_wline(&read_line2s(r), tang, t);
    l.set_creating_way_info(creating_way(way));
    l.set_period(periods[0], periods[1], periods[2], periods[3]);
    finish_point_line(r, fx, &l, &|l, a1, a2| {
        if let Some(a1) = a1 {
            l.set_arc_on_s1(&a1);
        }
        if let Some(a2) = a2 {
            l.set_arc_on_s2(&a2);
        }
    });
    Handle::new(l).cast()
}

fn restriction(r: &mut Reader, fx: &Fixture) -> Handle<dyn IntPatchLine> {
    let (tang, t) = read_trans(r);
    let l = make_rline(tang, t);
    if r.flag() {
        l.add(&read_line2s(r));
    }
    finish_point_line(r, fx, &l, &|l, a1, a2| {
        if let Some(a1) = a1 {
            l.set_arc_on_s1(&a1);
        }
        if let Some(a2) = a2 {
            l.set_arc_on_s2(&a2);
        }
    });
    Handle::new(l).cast()
}

fn conic(r: &mut Reader, fx: &Fixture) -> Result<Handle<dyn IntPatchLine>, Failure> {
    let kind = r.integer();
    let trans: (bool, IntPatchLineTransitions) = read_trans(r);
    let l: IntPatchGLine = make_gline(r, kind, trans)?;
    for _ in 0..r.integer() {
        l.add_vertex(&read_point(r, fx))?;
    }
    Ok(Handle::new(l).cast())
}

fn analytic(r: &mut Reader, fx: &Fixture) -> Result<Handle<dyn IntPatchLine>, Failure> {
    let curve = aline_curve(r)?;
    let (tang, t) = read_trans(r);
    let l = IntPatchALine::new(&curve, tang, t);
    for _ in 0..r.integer() {
        l.add_vertex(&read_point(r, fx));
    }
    Ok(Handle::new(l).cast())
}

/// `ga_wt_line`.
pub fn line(r: &mut Reader, fx: &Fixture) -> Result<Handle<dyn IntPatchLine>, Failure> {
    match r.integer() {
        0 => Ok(walking(r, fx)),
        1 => Ok(restriction(r, fx)),
        2 => conic(r, fx),
        _ => analytic(r, fx),
    }
}

/// `ga_wt_report`.
pub fn report(fx: &Fixture, l: &Handle<dyn IntPatchLine>) {
    line_base(l);
    let w: Handle<IntPatchWLine> = l.cast();
    if let Some(w) = w.get() {
        points(w);
        vertices(fx, w);
        int(fx.arc_rank(&w.get_arc_on_s1()));
        int(fx.arc_rank(&w.get_arc_on_s2()));
        int(w.get_creating_way() as i32);
        flag(w.is_purging_allowed());
        real(w.u1_period());
        real(w.v2_period());
    }
    let rl: Handle<IntPatchRLine> = l.cast();
    if let Some(rl) = rl.get() {
        points(rl);
        vertices(fx, rl);
        int(fx.arc_rank(&rl.arc_on_s1()));
        int(fx.arc_rank(&rl.arc_on_s2()));
    }
    let g: Handle<IntPatchGLine> = l.cast();
    if let Some(g) = g.get() {
        vertices(fx, g);
    }
    let a: Handle<IntPatchALine> = l.cast();
    if let Some(a) = a.get() {
        vertices(fx, a);
    }
}

/// `ga_ip_purge_case`.
pub fn purge_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    attempt(|| {
        let (s1, s2) = (surface(&mut r), surface(&mut r));
        let (d1, d2) = (domain(&mut r, &fx, &s1)?, domain(&mut r, &fx, &s2)?);
        let w: Handle<IntPatchWLine> = line(&mut r, &fx)?.cast();
        let p = IntPatchWLineTool::compute_purged_wline(&w, &s1, &s2, &d1, &d2)?;
        flag(p.is_null());
        if p.is_some() {
            report(&fx, &p.cast());
        }
        int(w.nb_pnts()?);
        int(w.nb_vertex());
        Ok(())
    });
    end();
}

/// `ga_ip_join_case`.
pub fn join_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    real(IntPatchWLineTool::MY_MAX_CONCAT_ANGLE);
    attempt(|| {
        let (s1, s2) = (surface(&mut r), surface(&mut r));
        let tol = r.real();
        let mut lines: Vec<Handle<dyn IntPatchLine>> = Vec::new();
        for _ in 0..r.integer() {
            lines.push(line(&mut r, &fx)?);
        }
        let mut pts: Vec<IntPatchPoint> =
            (0..r.integer()).map(|_| read_point(&mut r, &fx)).collect();
        attempt(|| IntPatchWLineTool::join_wlines(&mut lines, &mut pts, &s1, &s2, tol));
        int(lines.len() as i32);
        lines.iter().for_each(|l| report(&fx, l));
        int(pts.len() as i32);
        pts.iter().for_each(|p| point(&fx, p));
        Ok(())
    });
    end();
}

/// `ga_ip_lc_case`.
pub fn lc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    attempt(|| {
        let (s1, s2) = (surface(&mut r), surface(&mut r));
        let (d1, d2) = (domain(&mut r, &fx, &s1)?, domain(&mut r, &fx, &s2)?);
        let tol = r.real();
        let mut refs: Vec<Handle<dyn IntPatchLine>> = Vec::new();
        for _ in 0..r.integer() {
            refs.push(line(&mut r, &fx)?);
        }
        let l = line(&mut r, &fx)?;
        let mut lc = IntPatchLineConstructor::new(2);
        attempt(|| lc.perform(&refs, &l, &s1, &d1, &s2, &d2, tol));
        let n = lc.nb_lines();
        int(n);
        for i in 1..=n {
            report(&fx, &lc.line(i)?);
        }
        attempt(|| lc.line(0).map(|_| ()));
        attempt(|| lc.line(n + 1).map(|_| ()));
        Ok(())
    });
    end();
}
