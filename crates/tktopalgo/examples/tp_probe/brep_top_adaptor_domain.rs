//! BRepTopAdaptor_TopolTool as the domain of IntStart and IntPatch, the Rust twin of
//! tools/oracle/tktopalgo_probe_breptopadaptor_domain.cxx (docs/topalgo-design.md
//! section 8): the searches and IntPatch_Intersection run on faces read by `tp_face`,
//! and every vertex is a BRepTopAdaptor_HVertex reached through `dyn Adaptor3dHVertex`.
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real, xy, xyz};
use super::face::{tp_edge_index, tp_face};
use super::int_patch_report::{Domains, line, point};
use super::int_surf::quadric;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeomalgo::int_patch::{
    IntPatchArcFunction, IntPatchIntersection, IntPatchIntersectionOptions,
    IntPatchThePathPointOfTheSOnBounds, IntPatchTheSOnBounds, IntPatchTheSearchInside,
    IntPatchTheSurfFunction,
};
use rustycad_tkgeomalgo::int_start::{TheArc, TheVertex};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorTopolTool;

/// `TpBtaDomain`: the face, its adaptor (restricted or not) and its tool.
struct Domain {
    face: Face,
    surface: Handle<dyn Adaptor3dSurface>,
    tool: BRepTopAdaptorTopolTool,
}

/// `tp_bta_domain`: `face restr`.
fn domain(r: &mut Reader) -> Result<Domain, Failure> {
    let f = tp_face(r)?;
    let restr = r.flag();
    let surface: Handle<dyn Adaptor3dSurface> =
        Handle::new(BRepAdaptorSurface::with_face(&f.face, restr)?).cast();
    let tool = BRepTopAdaptorTopolTool::with_surface(&surface)?;
    Ok(Domain {
        face: f.face,
        surface,
        tool,
    })
}

/// `tp_bta_arc_edge`: the edge index of an arc (-3 when it is not a BRepAdaptor_Curve2d).
fn arc_edge(dm: &Domain, arc: &TheArc) -> i32 {
    let curve: Handle<BRepAdaptorCurve2d> = arc.cast();
    curve
        .get()
        .map_or(-3, |c| tp_edge_index(&dm.face, &c.edge()))
}

/// `tp_bta_domain_vertex`: the virtual members through the base handle, then the 3d
/// tolerance and point through the tool.
fn domain_vertex(dm: &Domain, v: &TheVertex, arc: &TheArc) {
    pnt2d(v.value());
    int(v.orientation() as i32);
    attempt(|| v.parameter(arc).map(real));
    attempt(|| v.resolution(arc).map(real));
    attempt(|| dm.tool.vertex_tol_3d(v).map(real));
    attempt(|| dm.tool.pnt(v).map(pnt));
}

/// `tp_bta_path_point`.
fn path_point(dm: &Domain, p: &IntPatchThePathPointOfTheSOnBounds) {
    pnt(p.value());
    real(p.tolerance());
    flag(p.is_new());
    real(p.parameter());
    int(arc_edge(dm, &p.arc()));
    attempt(|| p.vertex().map(|v| domain_vertex(dm, &v, &p.arc())));
}

/// `tp_bta_sob_case` after the search: its points and segments.
fn sob_report(dm: &Domain, sob: &IntPatchTheSOnBounds) {
    flag(sob.is_done());
    attempt(|| sob.all_arc_solution().map(flag));
    attempt(|| {
        let n = sob.nb_points()?;
        int(n);
        (1..=n).try_for_each(|i| sob.point(i).map(|p| path_point(dm, p)))
    });
    attempt(|| {
        let n = sob.nb_segments()?;
        int(n);
        (1..=n).try_for_each(|i| {
            let g = sob.segment(i)?;
            int(arc_edge(dm, &g.curve()));
            flag(g.has_first_point());
            attempt(|| g.first_point().map(|p| path_point(dm, p)));
            flag(g.has_last_point());
            attempt(|| g.last_point().map(|p| path_point(dm, p)));
            Ok(())
        })
    });
}

/// `bta_sob`: `face restr quadric tolb tolt recheck`.
pub fn sob_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let dm = domain(&mut r)?;
        let q = quadric(&mut r);
        let (tolb, tolt, recheck) = (r.real(), r.real(), r.flag());
        let mut f = IntPatchArcFunction::new();
        f.set_quadric(&q);
        f.set_surface(&dm.surface);
        let mut sob = IntPatchTheSOnBounds::new();
        attempt(|| sob.perform(&mut f, &dm.tool, tolb, tolt, recheck));
        sob_report(&dm, &sob);
        Ok(())
    });
    end();
}

/// `tp_bta_inside_case` after the search: its interior points.
fn inside_report(w: &IntPatchTheSearchInside) {
    flag(w.is_done());
    attempt(|| {
        let n = w.nb_points()?;
        int(n);
        (1..=n).try_for_each(|i| {
            let p = w.value(i)?;
            pnt(p.value());
            real(p.u_parameter());
            real(p.v_parameter());
            xyz(*p.direction().xyz());
            xy(p.direction2d().xy());
            Ok(())
        })
    });
}

/// `bta_inside`: `face restr quadric tol eps`.
pub fn inside_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let dm = domain(&mut r)?;
        let q = quadric(&mut r);
        let (tol, eps) = (r.real(), r.real());
        let mut f = IntPatchTheSurfFunction::with_surfaces(&dm.surface, &q);
        f.set_tolerance(tol);
        let w = IntPatchTheSearchInside::with_domain(&mut f, &dm.surface, &dm.tool, eps)?;
        inside_report(&w);
        Ok(())
    });
    end();
}

/// `tp_bta_intersection_case` after the intersection: its points and lines.
fn intersection_report(dm: &Domains, inter: &IntPatchIntersection) {
    flag(inter.is_done());
    attempt(|| inter.is_empty().map(flag));
    attempt(|| inter.tangent_faces().map(flag));
    attempt(|| inter.opposite_faces().map(flag));
    attempt(|| {
        let n = inter.nb_pnts()?;
        int(n);
        (1..=n).try_for_each(|i| inter.point(i).map(|p| point(dm, &p)))
    });
    attempt(|| {
        let n = inter.nb_lines()?;
        int(n);
        (1..=n).try_for_each(|i| inter.line(i).map(|l| line(dm, &l)))
    });
}

/// `bta_intersection`: `face1 restr1 face2 restr2 tolarc toltang geomint keeprline postwl`.
pub fn intersection_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let d1 = domain(&mut r)?;
        let d2 = domain(&mut r)?;
        let tols = (r.real(), r.real());
        let options = IntPatchIntersectionOptions {
            is_geom_int: r.flag(),
            is_req_to_keep_rline: r.flag(),
            is_req_to_post_wl_proc: r.flag(),
        };
        let dm = Domains::new(&d1.tool, &d2.tool);
        let mut inter = IntPatchIntersection::new();
        attempt(|| inter.perform(&d1.surface, &d1.tool, &d2.surface, &d2.tool, tols, options));
        intersection_report(&dm, &inter);
        Ok(())
    });
    end();
}
