//! IntPatch line probe operations, the Rust twin of the walking and restriction lines of
//! tools/oracle/tkgeomalgo_probe_intpatch_lines.cxx (the conic and analytic lines are in
//! `int_patch_curves`): one line per request, edited by a command script and printed.
use super::common::{Reader, attempt, captured, end, flag, int, real, text};
use super::int_patch::{Fixture, point, read_point};
use super::int_walk::pnt2s;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkgeomalgo::int_patch::{
    IntPatchALine, IntPatchGLine, IntPatchLine, IntPatchLineTransitions, IntPatchPoint,
    IntPatchPointLine, IntPatchRLine, IntPatchWLType, IntPatchWLine,
};
use rustycad_tkgeomalgo::int_surf::{
    IntSurfLineOn2S, IntSurfPntOn2S, IntSurfSituation, IntSurfTypeTrans,
};
use rustycad_tkmath::gp::{Point2, Point3};

fn type_trans(k: i32) -> IntSurfTypeTrans {
    match k {
        0 => IntSurfTypeTrans::In,
        1 => IntSurfTypeTrans::Out,
        2 => IntSurfTypeTrans::Touch,
        _ => IntSurfTypeTrans::Undecided,
    }
}

fn situation(k: i32) -> IntSurfSituation {
    match k {
        0 => IntSurfSituation::Inside,
        1 => IntSurfSituation::Outside,
        _ => IntSurfSituation::Unknown,
    }
}

/// `ga_ip_read_trans`: the tangency flag and the transitions.
pub fn read_trans(r: &mut Reader) -> (bool, IntPatchLineTransitions) {
    let (kind, tang) = (r.integer(), r.flag());
    if kind == 0 {
        return (tang, IntPatchLineTransitions::Undecided);
    }
    let (a, b) = (r.integer(), r.integer());
    if kind == 1 {
        return (
            tang,
            IntPatchLineTransitions::Types(type_trans(a), type_trans(b)),
        );
    }
    (
        tang,
        IntPatchLineTransitions::Touch(situation(a), situation(b)),
    )
}

/// `ga_ip_line_base`, on the line viewed as an `IntPatch_Line` handle.
pub fn line_base<L: IntPatchLine + RuntimeTyped + ?Sized>(line: &Handle<L>) {
    let h: Handle<dyn IntPatchLine> = line.cast();
    let l: &dyn IntPatchLine = h.get().expect("probe line");
    text(l.dynamic_type().name().as_bytes());
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
    flag(h.is_kind_name("IntPatch_PointLine"));
}

/// The vertex queries of `ga_ip_vertices`, common to the four lines.
pub trait VertexLine {
    fn count(&self) -> i32;
    fn at(&self, index: i32) -> Result<IntPatchPoint, Failure>;
    fn has_first(&self) -> bool;
    fn has_last(&self) -> bool;
    fn first(&self) -> Result<IntPatchPoint, Failure>;
    fn last(&self) -> Result<IntPatchPoint, Failure>;
}

macro_rules! vertex_line {
    ($line:ty) => {
        impl VertexLine for $line {
            fn count(&self) -> i32 {
                self.nb_vertex()
            }
            fn at(&self, index: i32) -> Result<IntPatchPoint, Failure> {
                self.vertex(index)
            }
            fn has_first(&self) -> bool {
                self.has_first_point()
            }
            fn has_last(&self) -> bool {
                self.has_last_point()
            }
            fn first(&self) -> Result<IntPatchPoint, Failure> {
                self.first_point()
            }
            fn last(&self) -> Result<IntPatchPoint, Failure> {
                self.last_point()
            }
        }
    };
}
vertex_line!(IntPatchWLine);
vertex_line!(IntPatchRLine);
vertex_line!(IntPatchGLine);
vertex_line!(IntPatchALine);

/// `ga_ip_vertices`.
pub fn vertices(fx: &Fixture, l: &dyn VertexLine) {
    let n = l.count();
    int(n);
    for i in 1..=n {
        point(fx, &l.at(i).expect("probe vertex"));
    }
    flag(l.has_first());
    flag(l.has_last());
    attempt(|| l.first().map(|p| point(fx, &p)));
    attempt(|| l.last().map(|p| point(fx, &p)));
    attempt(|| l.at(0).map(|_| ()));
    attempt(|| l.at(n + 1).map(|_| ()));
}

/// `ga_ip_points`.
pub fn points(l: &dyn IntPatchPointLine) {
    attempt(|| {
        let n = l.nb_pnts()?;
        int(n);
        for i in 1..=n {
            pnt2s(&l.point(i)?);
        }
        attempt(|| l.point(0).map(|_| ()));
        attempt(|| l.point(n + 1).map(|_| ()));
        flag(l.is_out_surf1_box(Point2::new(0.5, 0.5))?);
        flag(l.is_out_surf2_box(Point2::new(100.0, 100.0))?);
        flag(l.is_out_box(Point3::new(0.0, 0.0, 0.0))?);
        flag(l.is_out_box(Point3::new(100.0, 100.0, 100.0))?);
        Ok(())
    });
    flag(l.curve().is_null());
}

/// `ga_ip_read_line2s`.
pub fn read_line2s(r: &mut Reader) -> Handle<IntSurfLineOn2S> {
    let c = IntSurfLineOn2S::new(None);
    for _ in 0..r.integer() {
        let x = r.pnt();
        let (a, b, e, d) = (r.real(), r.real(), r.real(), r.real());
        let mut q = IntSurfPntOn2S::new();
        q.set_value_all(x, a, b, e, d);
        c.add(&q);
    }
    Handle::new(c)
}

/// The point-line commands of `ga_ip_point_line_command`; `Ok(false)` for another one.
fn point_line_command<L: IntPatchPointLine>(
    r: &mut Reader,
    fx: &Fixture,
    l: &L,
    c: i32,
    replace: &dyn Fn(&L, i32, &IntPatchPoint) -> Result<(), Failure>,
    first_last: &dyn Fn(&L, bool, i32),
    compute: &dyn Fn(&L, f64) -> Result<(), Failure>,
) -> Result<bool, Failure> {
    match c {
        1 => {
            let pre = r.flag();
            l.add_vertex(&read_point(r, fx), pre);
        }
        3 => {
            let i = r.integer();
            replace(l, i, &read_point(r, fx))?;
        }
        4 => first_last(l, true, r.integer()),
        5 => first_last(l, false, r.integer()),
        6 => compute(l, r.real())?,
        7 => l.remove_vertex(r.integer())?,
        9 => l.clear_vertexes(),
        12 => {
            let (a, b, e) = (r.flag(), r.flag(), r.flag());
            l.set_value(a, b, e, r.flag());
        }
        14 => {
            let i = r.integer();
            let prm = r.real();
            l.change_vertex(i, &mut |p| p.set_parameter(prm))?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn wline_command(r: &mut Reader, fx: &Fixture, l: &IntPatchWLine, c: i32) -> Result<(), Failure> {
    let done = point_line_command(
        r,
        fx,
        l,
        c,
        &|l, i, p| l.replace(i, p),
        &|l, first, i| {
            if first {
                l.set_first_point(i)
            } else {
                l.set_last_point(i)
            }
        },
        &|l, tol| l.compute_vertex_parameters(tol),
    )?;
    if done {
        return Ok(());
    }
    match c {
        2 => {
            let i = r.integer();
            l.set_point(i, &read_point(r, fx))
        }
        8 => {
            let i = r.integer();
            l.insert_vertex_before(i, &read_point(r, fx))
        }
        10 => {
            l.set_arc_on_s1(&fx.arc(r.integer()));
            Ok(())
        }
        11 => {
            l.set_arc_on_s2(&fx.arc(r.integer()));
            Ok(())
        }
        _ => {
            l.enable_purging(r.flag());
            Ok(())
        }
    }
}

pub fn creating_way(k: i32) -> IntPatchWLType {
    match k {
        1 => IntPatchWLType::ImpImp,
        2 => IntPatchWLType::ImpPrm,
        3 => IntPatchWLType::PrmPrm,
        _ => IntPatchWLType::Unknown,
    }
}

pub fn make_wline(
    c: &Handle<IntSurfLineOn2S>,
    tang: bool,
    t: IntPatchLineTransitions,
) -> IntPatchWLine {
    match t {
        IntPatchLineTransitions::Types(a, b) => IntPatchWLine::new(c, tang, a, b),
        IntPatchLineTransitions::Touch(a, b) => IntPatchWLine::new_touch(c, tang, a, b),
        IntPatchLineTransitions::Undecided => IntPatchWLine::new_undecided(c, tang),
    }
}

/// `ga_ip_wline_case`.
pub fn wline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    let dump = r.integer();
    let (tang, t) = read_trans(&mut r);
    let way = r.integer();
    let periods = [r.real(), r.real(), r.real(), r.real()];
    let c = read_line2s(&mut r);
    let l = Handle::new(make_wline(&c, tang, t));
    l.set_creating_way_info(creating_way(way));
    l.set_period(periods[0], periods[1], periods[2], periods[3]);
    for _ in 0..r.integer() {
        l.add_vertex(&read_point(&mut r, &fx), false);
    }
    for _ in 0..r.integer() {
        let cmd = r.integer();
        attempt(|| wline_command(&mut r, &fx, &l, cmd));
    }
    line_base(&l);
    points(&*l);
    vertices(&fx, &*l);
    attempt(|| l.first_point_index().map(|(_, i)| int(i)));
    attempt(|| l.last_point_index().map(|(_, i)| int(i)));
    [l.u1_period(), l.v1_period(), l.u2_period(), l.v2_period()]
        .into_iter()
        .for_each(real);
    flag(l.has_arc_on_s1());
    int(fx.arc_rank(&l.get_arc_on_s1()));
    flag(l.has_arc_on_s2());
    int(fx.arc_rank(&l.get_arc_on_s2()));
    flag(l.is_purging_allowed());
    int(l.get_creating_way() as i32);
    if dump >= 0 {
        captured(|out| l.dump(dump, out));
    }
    end();
}

fn rline_command(r: &mut Reader, fx: &Fixture, l: &IntPatchRLine, c: i32) -> Result<(), Failure> {
    let done = point_line_command(
        r,
        fx,
        l,
        c,
        &|l, i, p| l.replace(i, p),
        &|l, first, i| {
            if first {
                l.set_first_point(i)
            } else {
                l.set_last_point(i)
            }
        },
        &|l, tol| l.compute_vertex_parameters(tol),
    )?;
    if done {
        return Ok(());
    }
    match c {
        2 => {
            let i = r.integer();
            l.set_point(i, &read_point(r, fx))
        }
        10 => {
            l.set_arc_on_s1(&fx.arc(r.integer()));
            Ok(())
        }
        11 => {
            l.set_arc_on_s2(&fx.arc(r.integer()));
            Ok(())
        }
        13 => {
            l.set_curve(&read_line2s(r));
            Ok(())
        }
        _ => {
            l.set_curve(&Handle::default());
            Ok(())
        }
    }
}

pub fn make_rline(tang: bool, t: IntPatchLineTransitions) -> IntPatchRLine {
    match t {
        IntPatchLineTransitions::Types(a, b) => IntPatchRLine::new(tang, a, b),
        IntPatchLineTransitions::Touch(a, b) => IntPatchRLine::new_touch(tang, a, b),
        IntPatchLineTransitions::Undecided => IntPatchRLine::new_undecided(tang),
    }
}

/// `ga_ip_rline_case`.
pub fn rline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    let dump = r.integer();
    let (tang, t) = read_trans(&mut r);
    let l = Handle::new(make_rline(tang, t));
    if r.flag() {
        l.add(&read_line2s(&mut r));
    }
    for _ in 0..r.integer() {
        l.add_vertex(&read_point(&mut r, &fx), false);
    }
    for _ in 0..r.integer() {
        let cmd = r.integer();
        attempt(|| rline_command(&mut r, &fx, &l, cmd));
    }
    line_base(&l);
    points(&*l);
    vertices(&fx, &*l);
    flag(l.has_polygon());
    flag(l.is_arc_on_s1());
    int(fx.arc_rank(&l.arc_on_s1()));
    flag(l.is_arc_on_s2());
    int(fx.arc_rank(&l.arc_on_s2()));
    let ((a, b), (c, d)) = (l.param_on_s1(), l.param_on_s2());
    [a, b, c, d].into_iter().for_each(real);
    if dump >= 0 {
        captured(|out| l.dump(dump, out));
    }
    end();
}
