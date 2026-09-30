//! IntStart probe operations, the Rust twin of tools/oracle/tkgeomalgo_probe_intstart.cxx:
//! the two generics instantiated with the probe ports of the IntPatch classes
//! (`int_start_tools`), `HSurfaceTool`, `IntImpZerImpFunc` and `Adaptor3dTopolTool`.
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real, xy, xyz};
use super::ics_geometry::surface;
use super::int_start_tools::{
    ProbeArcFunction, ProbeDomain, ProbeHInterTool, ProbePathPoint, ProbeSeenTool, ProbeSegment,
    SEEN, SeenTable,
};
use super::int_surf::quadric;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkgeomalgo::int_imp::{HSurfaceTool, IntImpZerImpFunc};
use rustycad_tkgeomalgo::int_start::{
    SearchInside, SearchInsideTypes, SearchOnBoundaries, SearchOnBoundariesTypes, TheArc,
    ThePathPoint, TheTopolTool,
};
use rustycad_tkgeomalgo::int_surf::IntSurfQuadricTool;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis3, Direction3};
use rustycad_tkmath::toploc::Location;

/// The IntPatch_TheSOnBounds instance on the probe domain.
struct PatchTypes;
impl SearchOnBoundariesTypes for PatchTypes {
    type TopolTool = ProbeDomain;
    type SOBTool = ProbeHInterTool;
    type Function = ProbeArcFunction;
    type PathPoint = ProbePathPoint;
    type Segment = ProbeSegment;
}
/// The instance on the tool whose arcs have been seen.
struct SeenTypes;
impl SearchOnBoundariesTypes for SeenTypes {
    type TopolTool = ProbeDomain;
    type SOBTool = ProbeSeenTool;
    type Function = ProbeArcFunction;
    type PathPoint = ProbePathPoint;
    type Segment = ProbeSegment;
}
/// The IntPatch_TheSearchInside instance.
struct InsideTypes;
impl SearchInsideTypes for InsideTypes {
    type SurfaceTool = HSurfaceTool;
    type TopolTool = Adaptor3dTopolTool;
    type Function = IntImpZerImpFunc<HSurfaceTool, IntSurfQuadricTool>;
}

/// `ga_is_edge`: no edge, or a straight edge with its flags and regularity.
fn edge(r: &mut Reader) -> Result<Option<Edge>, Failure> {
    if !r.flag() {
        return Ok(None);
    }
    let (p, d) = (r.pnt(), r.dir());
    let (f, l) = (r.real(), r.real());
    let (deg, reg) = (r.flag(), r.flag());
    let b = BRepBuilder::default();
    let mut e = Edge::default();
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(p, d)).cast();
    b.make_edge_with_curve(&mut e, &line, 1.0e-7)?;
    b.range(&e, f, l, false)?;
    b.degenerated(&e, deg)?;
    if reg {
        let z = Direction3::new(0.0, 0.0, 1.0)?;
        let y = Direction3::new(0.0, 1.0, 0.0)?;
        let s1: Handle<dyn GeomSurface> =
            Handle::new(GeomPlane::new(Axis3::from_normal(p, z)?)).cast();
        let s2: Handle<dyn GeomSurface> =
            Handle::new(GeomPlane::new(Axis3::from_normal(p, y)?)).cast();
        let loc = Location::default();
        b.continuity(&e, (&s1, &s2), (&loc, &loc), Shape::C1)?;
    }
    Ok(Some(e))
}

/// `ga_is_arc_rank`: the rank of the arc among the restrictions of the domain.
fn arc_rank(d: &ProbeDomain, arc: &TheArc) -> i32 {
    let mut rank = 0;
    d.init();
    while d.more() {
        rank += 1;
        if d.value().is_ok_and(|a| &a == arc) {
            return rank;
        }
        d.next();
    }
    0
}

fn path_point(d: &ProbeDomain, p: &ProbePathPoint) {
    pnt(p.value());
    real(p.tolerance());
    flag(p.is_new());
    real(p.parameter());
    int(arc_rank(d, &p.arc()));
    attempt(|| {
        let v = p.vertex()?;
        pnt2d(v.value());
        real(v.resolution(&p.arc()));
        Ok(())
    });
}

fn segment(d: &ProbeDomain, g: &ProbeSegment) {
    int(arc_rank(d, &g.arc));
    let domain_error = || print!("Standard_DomainError ");
    flag(g.first.is_some());
    g.first
        .as_ref()
        .map_or_else(domain_error, |p| path_point(d, p));
    flag(g.last.is_some());
    g.last
        .as_ref()
        .map_or_else(domain_error, |p| path_point(d, p));
}

fn sob_report<T: SearchOnBoundariesTypes<PathPoint = ProbePathPoint, Segment = ProbeSegment>>(
    s: &SearchOnBoundaries<T>,
    d: &ProbeDomain,
) {
    flag(s.is_done());
    attempt(|| {
        flag(s.all_arc_solution()?);
        Ok(())
    });
    attempt(|| {
        let n = s.nb_points()?;
        int(n);
        (1..=n).try_for_each(|i| {
            path_point(d, s.point(i)?);
            Ok(())
        })
    });
    attempt(|| {
        let n = s.nb_segments()?;
        int(n);
        (1..=n).try_for_each(|i| {
            segment(d, s.segment(i)?);
            Ok(())
        })
    });
    attempt(|| s.point(0).map(|_| ()));
    attempt(|| s.point(s.nb_points()? + 1).map(|_| ()));
    attempt(|| s.segment(0).map(|_| ()));
    attempt(|| s.segment(s.nb_segments()? + 1).map(|_| ()));
}

fn seen_table(r: &mut Reader) -> SeenTable {
    let mut t = SeenTable::default();
    for _ in 0..r.integer() {
        t.pts.push(std::array::from_fn(|_| r.real()));
    }
    for _ in 0..r.integer() {
        let f = r.integer();
        t.segs.push((f, r.integer()));
    }
    t.all = r.flag();
    t
}

fn run_sob<T>(f: &mut ProbeArcFunction, d: &ProbeDomain, tols: (f64, f64, bool), nperf: i32)
where
    T: SearchOnBoundariesTypes<
            TopolTool = ProbeDomain,
            Function = ProbeArcFunction,
            PathPoint = ProbePathPoint,
            Segment = ProbeSegment,
        >,
{
    let mut sob = SearchOnBoundaries::<T>::new();
    for _ in 0..nperf {
        attempt(|| sob.perform(f, d, tols.0, tols.1, tols.2));
    }
    sob_report(&sob, d);
}

/// `ga_is_sob_case`.
pub fn sob_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let seen = r.flag();
    let s = surface(&mut r);
    let q = quadric(&mut r);
    let (tolb, tolt, recheck) = (r.real(), r.real(), r.flag());
    let mode = r.integer();
    let e = edge(&mut r);
    if seen {
        let table = seen_table(&mut r);
        SEEN.with(|t| *t.borrow_mut() = table);
    }
    let nperf = match mode {
        0 => 1,
        1 => 2,
        _ => 0,
    };
    attempt(|| {
        let d = ProbeDomain {
            tool: Adaptor3dTopolTool::with_surface(&s)?,
            edge: e?,
        };
        let mut f = ProbeArcFunction::default();
        f.set_quadric(&q);
        f.set_surface(&s);
        if seen {
            run_sob::<SeenTypes>(&mut f, &d, (tolb, tolt, recheck), nperf);
        } else {
            run_sob::<PatchTypes>(&mut f, &d, (tolb, tolt, recheck), nperf);
        }
        Ok(())
    });
    end();
}

fn inside_report(s: &SearchInside<InsideTypes>) {
    flag(s.is_done());
    attempt(|| {
        let n = s.nb_points()?;
        int(n);
        (1..=n).try_for_each(|i| {
            let p = s.value(i)?;
            pnt(p.value());
            real(p.u_parameter());
            real(p.v_parameter());
            xyz(*p.direction().xyz());
            xy(p.direction2d().xy());
            Ok(())
        })
    });
    attempt(|| s.value(0).map(|_| ()));
    attempt(|| s.value(s.nb_points()? + 1).map(|_| ()));
}

/// `ga_is_inside_case`.
pub fn inside_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = surface(&mut r);
    let q = quadric(&mut r);
    let (tol, eps, mode) = (r.real(), r.real(), r.integer());
    attempt(|| {
        let mut f = IntImpZerImpFunc::<HSurfaceTool, IntSurfQuadricTool>::with_surfaces(&s, &q);
        f.set_tolerance(tol);
        let d = Adaptor3dTopolTool::with_surface(&s)?;
        match mode {
            0 => attempt(|| {
                let w = SearchInside::<InsideTypes>::with_domain(&mut f, &s, &d, eps)?;
                inside_report(&w);
                Ok(())
            }),
            1 => {
                let (u, v) = (r.real(), r.real());
                let mut w = SearchInside::<InsideTypes>::new();
                attempt(|| w.perform_from(&mut f, &s, u, v));
                inside_report(&w);
            }
            _ => inside_report(&SearchInside::<InsideTypes>::new()),
        }
        Ok(())
    });
    end();
}
