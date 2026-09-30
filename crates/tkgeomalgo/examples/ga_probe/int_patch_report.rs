//! The printers of the IntPatch analytic intersections, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_intpatch_report.cxx: the lines, their vertices and the arcs
//! of both domains.
use super::common::{attempt, dir, flag, int, pnt, pnt2d, real, xyz};
use super::int_patch::trans;
use super::int_walk::pnt2s;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::int_patch::{
    IntPatchALine, IntPatchGLine, IntPatchIType, IntPatchLine, IntPatchPoint, IntPatchPointLine,
    IntPatchRLine, IntPatchWLine,
};
use rustycad_tkgeomalgo::int_start::{TheArc, TheTopolTool, TheVertex};
use rustycad_tkmath::gp::{Axis2, Point3, Vector3};

/// GaIplDomains: the arcs of both domains, collected before the intersection runs.
pub struct Domains {
    a1: Vec<TheArc>,
    a2: Vec<TheArc>,
}

// Any domain (the TKTopAlgo probe passes BRepTopAdaptor_TopolTool ones), through the
// virtual members, as the native `occ::handle<Adaptor3d_TopolTool>`.
fn collect(d: &dyn TheTopolTool) -> Vec<TheArc> {
    let mut v = Vec::new();
    d.init();
    while d.more() {
        v.push(d.value().expect("probe domain arc"));
        d.next();
    }
    v
}

impl Domains {
    pub fn new(d1: &dyn TheTopolTool, d2: &dyn TheTopolTool) -> Self {
        Self {
            a1: collect(d1),
            a2: collect(d2),
        }
    }
    /// The rank of an arc: 1.. on the first domain, 101.. on the second one; 0 null, -1
    /// another arc.
    pub fn rank(&self, a: &TheArc) -> i32 {
        if a.is_null() {
            return 0;
        }
        let same = |x: &TheArc| x.object_address() == a.object_address();
        if let Some(k) = self.a1.iter().position(same) {
            return k as i32 + 1;
        }
        self.a2.iter().position(same).map_or(-1, |k| k as i32 + 101)
    }
}

/// `ga_ipl_hvertex`.
fn hvertex(v: &TheVertex) {
    flag(v.is_null());
    if let Some(x) = v.get() {
        pnt2d(x.value());
        int(x.orientation() as i32);
    }
}

/// `ga_ipl_point`.
pub fn point(dm: &Domains, p: &IntPatchPoint) {
    pnt(p.value());
    real(p.parameter_on_line());
    real(p.tolerance());
    flag(p.is_tangency_point());
    flag(p.is_multiple());
    pnt2s(&p.pnt_on_2s());
    flag(p.is_on_dom_s1());
    flag(p.is_vertex_on_s1());
    attempt(|| p.vertex_on_s1().map(|v| hvertex(&v)));
    attempt(|| p.arc_on_s1().map(|a| int(dm.rank(&a))));
    attempt(|| p.transition_line_arc1().map(|t| trans(&t)));
    attempt(|| p.transition_on_s1().map(|t| trans(&t)));
    attempt(|| p.parameter_on_arc1().map(real));
    flag(p.is_on_dom_s2());
    flag(p.is_vertex_on_s2());
    attempt(|| p.vertex_on_s2().map(|v| hvertex(&v)));
    attempt(|| p.arc_on_s2().map(|a| int(dm.rank(&a))));
    attempt(|| p.transition_line_arc2().map(|t| trans(&t)));
    attempt(|| p.transition_on_s2().map(|t| trans(&t)));
    attempt(|| p.parameter_on_arc2().map(real));
}

fn ax2(a: Axis2) {
    pnt(a.location());
    dir(a.direction());
    dir(a.x_direction());
}

/// The vertices `NbVertex`, `Vertex(0..n+1)` and the end flags of a line.
fn vertices(
    dm: &Domains,
    n: i32,
    vertex: impl Fn(i32) -> Result<IntPatchPoint, Failure>,
    ends: (bool, bool),
) {
    int(n);
    for i in 0..=n + 1 {
        attempt(|| vertex(i).map(|v| point(dm, &v)));
    }
    flag(ends.0);
    flag(ends.1);
}

fn gline(dm: &Domains, g: &IntPatchGLine) {
    attempt(|| {
        match g.arc_type() {
            IntPatchIType::Lin => {
                let l = g.line()?;
                pnt(l.location());
                dir(l.direction());
            }
            IntPatchIType::Circle => {
                let c = g.circle()?;
                ax2(c.position());
                real(c.radius());
            }
            IntPatchIType::Ellipse => {
                let e = g.ellipse()?;
                ax2(e.position());
                real(e.major_radius());
                real(e.minor_radius());
            }
            IntPatchIType::Parabola => {
                let p = g.parabola()?;
                ax2(p.position());
                real(p.focal());
            }
            _ => {
                let h = g.hyperbola()?;
                ax2(h.position());
                real(h.major_radius());
                real(h.minor_radius());
            }
        }
        Ok(())
    });
    vertices(
        dm,
        g.nb_vertex(),
        |i| g.vertex(i),
        (g.has_first_point(), g.has_last_point()),
    );
}

/// `ga_ipl_aline`.
fn aline(dm: &Domains, a: &IntPatchALine) {
    let (mut f, mut l) = (0.0, 0.0);
    attempt(|| {
        let (x, inc) = a.first_parameter()?;
        f = x;
        real(f);
        flag(inc);
        Ok(())
    });
    attempt(|| {
        let (x, inc) = a.last_parameter()?;
        l = x;
        real(l);
        flag(inc);
        Ok(())
    });
    for k in 0..=4 {
        attempt(|| a.value(f + (l - f) * f64::from(k) / 4.0).map(pnt));
    }
    attempt(|| {
        let (mut p, mut v) = (Point3::default(), Vector3::default());
        let ok = a.d1(0.5 * (f + l), &mut p, &mut v)?;
        flag(ok);
        pnt(p);
        if ok {
            xyz(*v.xyz());
        }
        Ok(())
    });
    vertices(
        dm,
        a.nb_vertex(),
        |i| a.vertex(i),
        (a.has_first_point(), a.has_last_point()),
    );
}

fn wline(dm: &Domains, w: &IntPatchWLine) {
    let n = w.nb_pnts().expect("walking line points");
    int(n);
    for i in 1..=n {
        pnt2s(&w.point(i).expect("walking line point"));
    }
    real(w.u1_period());
    real(w.v1_period());
    real(w.u2_period());
    real(w.v2_period());
    int(w.get_creating_way() as i32);
    flag(w.is_purging_allowed());
    flag(w.has_arc_on_s1());
    int(dm.rank(&w.get_arc_on_s1()));
    flag(w.has_arc_on_s2());
    int(dm.rank(&w.get_arc_on_s2()));
    vertices(
        dm,
        w.nb_vertex(),
        |i| w.vertex(i),
        (w.has_first_point(), w.has_last_point()),
    );
}

fn rline(dm: &Domains, r: &IntPatchRLine) {
    flag(r.is_arc_on_s1());
    int(dm.rank(&r.arc_on_s1()));
    flag(r.is_arc_on_s2());
    int(dm.rank(&r.arc_on_s2()));
    flag(r.has_polygon());
    attempt(|| {
        let n = r.nb_pnts()?;
        int(n);
        for i in 1..=n {
            pnt2s(&r.point(i)?);
        }
        Ok(())
    });
    vertices(
        dm,
        r.nb_vertex(),
        |i| r.vertex(i),
        (r.has_first_point(), r.has_last_point()),
    );
}

/// `ga_ipl_line`.
pub fn line(dm: &Domains, l: &Handle<dyn IntPatchLine>) {
    int(l.arc_type() as i32);
    flag(l.is_tangent());
    int(l.transition_on_s1() as i32);
    int(l.transition_on_s2() as i32);
    attempt(|| l.situation_s1().map(|s| int(s as i32)));
    attempt(|| l.situation_s2().map(|s| int(s as i32)));
    flag(l.is_u_iso_on_s1());
    flag(l.is_v_iso_on_s1());
    flag(l.is_u_iso_on_s2());
    flag(l.is_v_iso_on_s2());
    match l.arc_type() {
        IntPatchIType::Analytic => {
            aline(dm, l.cast::<IntPatchALine>().get().expect("analytic line"))
        }
        IntPatchIType::Walking => wline(dm, l.cast::<IntPatchWLine>().get().expect("walking line")),
        IntPatchIType::Restriction => rline(
            dm,
            l.cast::<IntPatchRLine>().get().expect("restriction line"),
        ),
        _ => gline(dm, l.cast::<IntPatchGLine>().get().expect("conic line")),
    }
}
