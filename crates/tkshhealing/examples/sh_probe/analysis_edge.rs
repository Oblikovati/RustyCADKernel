//! ShapeAnalysis_Edge probe operations, the Rust twin of
//! tools/oracle/tkshhealing_probe_analysis_edge.cxx (which documents the requests).
use super::common::{Reader, attempt, end, flag, int, pnt2d, real};
use super::healing_common::{curve2d, curve3d, flags, vec2d};
use super::shapes::{TpShapes, tp_same_index, tp_shapes};
use super::wire_data::{sh_edge, sh_face};
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::gp::Point2;
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;

/// `sh_ea_vertex`.
fn vertex(t: &TpShapes, v: &Vertex) {
    int(tp_same_index(t, v));
    int(if v.is_null() {
        -1
    } else {
        v.orientation() as i32
    });
}

fn status_flags(sae: &ShapeAnalysisEdge) {
    flags(|s| sae.status_flag(s));
}

/// `sh_ea_curves`.
fn curves(
    r: &mut Reader<'_>,
    t: &TpShapes,
    sae: &ShapeAnalysisEdge,
    op: i32,
) -> Result<(), Failure> {
    let e = sh_edge(t, r.integer())?;
    if op == 0 {
        let (c, f, l) = sae.curve_3d(&e, r.flag())?;
        flag(!c.is_null());
        curve3d(&c, f, l);
        flag(sae.has_curve_3d(&e)?);
        flag(sae.is_closed_3d(&e)?);
        return Ok(());
    }
    if op == 3 {
        vertex(t, &sae.first_vertex(&e));
        vertex(t, &sae.last_vertex(&e));
        return Ok(());
    }
    if op == 1 {
        let face = sh_face(t, r.integer())?;
        let (c, f, l) = sae.pcurve_on_face(&e, &face, r.flag())?;
        print_pcurve(&c, f, l);
        flag(sae.has_pcurve_on_face(&e, &face)?);
        print_bound(sae.bound_uv_on_face(&e, &face)?);
        flag(sae.is_seam_on_face(&e, &face)?);
        return Ok(());
    }
    let s = t.surfaces[r.integer() as usize].clone();
    let loc = t.locations[r.integer() as usize].clone();
    let (c, f, l) = sae.pcurve_on_surface(&e, &s, &loc, r.flag())?;
    print_pcurve(&c, f, l);
    flag(sae.has_pcurve_on_surface(&e, &s, &loc)?);
    print_bound(sae.bound_uv_on_surface(&e, &s, &loc)?);
    flag(sae.is_seam_on_surface(&e, &s, &loc)?);
    Ok(())
}

fn print_pcurve(c: &Handle<dyn Geom2dCurve>, f: f64, l: f64) {
    flag(!c.is_null());
    curve2d(c, f, l);
}

fn print_bound(bound: Option<(Point2, Point2)>) {
    flag(bound.is_some());
    if let Some((a, b)) = bound {
        pnt2d(a);
        pnt2d(b);
    }
}

/// `sh_ea_tangent`.
fn tangent(
    r: &mut Reader<'_>,
    t: &TpShapes,
    sae: &ShapeAnalysisEdge,
    op: i32,
) -> Result<(), Failure> {
    let e = sh_edge(t, r.integer())?;
    let (ok, p, v) = if op == 4 {
        let face = sh_face(t, r.integer())?;
        let at_end = r.flag();
        sae.get_end_tangent_2d_on_face(&e, &face, at_end, r.real())?
    } else {
        let s = t.surfaces[r.integer() as usize].clone();
        let loc = t.locations[r.integer() as usize].clone();
        let at_end = r.flag();
        sae.get_end_tangent_2d_on_surface(&e, &s, &loc, at_end, r.real())?
    };
    flag(ok);
    pnt2d(p);
    vec2d(v);
    Ok(())
}

/// `sh_ea_vertices`.
fn vertices(
    r: &mut Reader<'_>,
    t: &TpShapes,
    sae: &mut ShapeAnalysisEdge,
    op: i32,
) -> Result<(), Failure> {
    let e = sh_edge(t, r.integer())?;
    match op {
        6 => {
            let p = r.real();
            flag(sae.check_vertices_with_curve_3d(&e, p, r.integer())?);
        }
        7 => {
            let face = sh_face(t, r.integer())?;
            let p = r.real();
            flag(sae.check_vertices_with_pcurve_on_face(&e, &face, p, r.integer())?);
        }
        8 => {
            let s = t.surfaces[r.integer() as usize].clone();
            let loc = t.locations[r.integer() as usize].clone();
            let p = r.real();
            flag(sae.check_vertices_with_pcurve_on_surface(&e, &s, &loc, p, r.integer())?);
        }
        _ => {
            let (ok, t1, t2) = if op == 9 {
                sae.check_vertex_tolerance_on_face(&e, &sh_face(t, r.integer())?)?
            } else {
                sae.check_vertex_tolerance(&e)?
            };
            flag(ok);
            if !sae.status_flag(ShapeExtendStatus::Fail1) {
                real(t1);
                real(t2);
            }
        }
    }
    status_flags(sae);
    Ok(())
}

/// `sh_ea_checks`.
fn checks(
    r: &mut Reader<'_>,
    t: &TpShapes,
    sae: &mut ShapeAnalysisEdge,
    op: i32,
) -> Result<(), Failure> {
    if op == 15 {
        let (f, l) = (r.real(), r.real());
        flag(sae.check_pcurve_range(f, l, &t.pcurves[r.integer() as usize])?);
        return Ok(());
    }
    let e = sh_edge(t, r.integer())?;
    match op {
        11 => flag(sae.check_curve_3d_with_pcurve_on_face(&e, &sh_face(t, r.integer())?)?),
        12 => {
            let s = t.surfaces[r.integer() as usize].clone();
            let loc = t.locations[r.integer() as usize].clone();
            flag(sae.check_curve_3d_with_pcurve_on_surface(&e, &s, &loc)?);
        }
        13 => {
            let (ok, d) = sae.check_same_parameter(&e, r.integer())?;
            flag(ok);
            real(d);
        }
        14 => {
            let face = sh_face(t, r.integer())?;
            let (ok, d) = sae.check_same_parameter_on_face(&e, &face, r.integer())?;
            flag(ok);
            real(d);
        }
        _ => {
            let e2 = sh_edge(t, r.integer())?;
            let mut tol = r.real();
            flag(sae.check_overlapping(&e, &e2, &mut tol, r.real())?);
            real(tol);
        }
    }
    status_flags(sae);
    Ok(())
}

/// `sh_edgeana program op ...`.
pub fn edgeana_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = tp_shapes(&mut r).expect("probe shapes");
    let op = r.integer();
    let mut sae = ShapeAnalysisEdge::new();
    match op {
        0..=3 => attempt(|| curves(&mut r, &t, &sae, op)),
        4 | 5 => attempt(|| tangent(&mut r, &t, &sae, op)),
        6..=10 => attempt(|| vertices(&mut r, &t, &mut sae, op)),
        _ => attempt(|| checks(&mut r, &t, &mut sae, op)),
    }
    end();
}
