//! HLRBRep_EdgeData, HLRBRep_FaceData, HLRBRep_FaceIterator, HLRBRep_EdgeFaceTool and the
//! HLRBRep package functions, the Rust twin of tools/oracle/tkhlr_probe_hlrbrep_records.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt, real, show};
use super::hlrbrep_faces::face;
use super::hlrbrep_fixtures::{geometry, projector};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkhlr::hlr_algo::{Projector, init_min_max};
use rustycad_tkhlr::hlr_brep::{
    BCurveTool, Curve, EdgeData, EdgeFaceTool, FaceData, FaceIterator, Surface, make_edge,
    make_edge_3d, poly_hlr_angle_and_deflection,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
use std::sync::Arc;

/// `hb_edge_v`.
pub fn edge_v(r: &mut Reader) -> Result<Edge, Failure> {
    let c = geometry(r)?;
    let (u1, u2) = (r.real(), r.real());
    BRepBuilderAPIMakeEdge::from_curve_params(&c, u1, u2)?.edge()
}

pub fn edge_summary(e: &Edge) -> Result<(), Failure> {
    flag(e.is_null());
    if e.is_null() {
        return Ok(());
    }
    let (v1, v2) = topexp::vertices(e, false);
    for v in [v1, v2] {
        flag(v.is_null());
        if !v.is_null() {
            pnt(BRepTool::pnt(&v)?);
        }
    }
    attempt(|| {
        let c = BRepAdaptorCurve::with_edge(e)?;
        let (a, b) = (c.first_parameter()?, c.last_parameter()?);
        real(a);
        real(b);
        int(c.get_type()? as i32);
        for t in [a, (a + b) / 2., b] {
            pnt(c.value(t)?);
        }
        Ok(())
    });
    Ok(())
}

// The vertex indices, tolerance and status of a default EdgeData are uninitialized
// natively, so `full` is false before Set.
pub fn edge_flags(d: &EdgeData, full: bool) {
    for v in [
        d.selected(),
        d.rg1_line(),
        d.rgn_line(),
        d.vertical(),
        d.simple(),
        d.out_lv_sta(),
        d.out_lv_end(),
        d.cut_at_sta(),
        d.cut_at_end(),
        d.ver_at_sta(),
        d.ver_at_end(),
        d.auto_intersection_done(),
        d.used(),
    ] {
        flag(v);
    }
    int(d.hide_count());
    if !full {
        return;
    }
    int(d.v_sta());
    int(d.v_end());
    real(f64::from(d.tolerance()));
    let (s, ts, e, te) = d.status().bounds();
    real(s);
    real(f64::from(ts));
    real(e);
    real(f64::from(te));
}

fn edge_conversions(c: &Curve) -> Result<(), Failure> {
    let (a, b) = (
        BCurveTool::first_parameter(c.curve())?,
        BCurveTool::last_parameter(c.curve())?,
    );
    attempt(|| edge_summary(&make_edge(c, a, b)?));
    attempt(|| edge_summary(&make_edge(c, a + (b - a) / 4., b - (b - a) / 3.)?));
    attempt(|| edge_summary(&make_edge_3d(c, a, b)?));
    attempt(|| edge_summary(&make_edge_3d(c, a + (b - a) / 4., b)?));
    Ok(())
}

fn edgedata_body(r: &mut Reader, projector: Arc<Projector>) -> Result<(), Failure> {
    let edge = edge_v(r)?;
    let mut f = [false; 9];
    f.iter_mut().for_each(|b| *b = r.flag());
    let (s, ts, e, te) = (r.real(), r.real() as f32, r.real(), r.real() as f32);
    let mut d = EdgeData::new();
    edge_flags(&d, false);
    d.set(
        f[0], f[1], &edge, 3, 7, f[2], f[3], f[4], f[5], s, ts, e, te,
    )?;
    edge_flags(&d, true);
    d.set_vertical(f[6]);
    d.set_simple(f[7]);
    d.set_auto_intersection_done(f[8]);
    d.set_used(true);
    d.set_hide_count(4);
    d.set_selected(false);
    d.set_ver_at_sta(true);
    d.set_ver_at_end(f[8]);
    edge_flags(&d, true);
    let c = d.change_geometry();
    c.set_projector(Some(projector));
    let (mut mn, mut mx) = ([0.; 16], [0.; 16]);
    init_min_max(1e100, &mut mn, &mut mx);
    attempt(|| {
        c.update(&mut mn, &mut mx)?;
        edge_conversions(c)
    });
    Ok(())
}

/// `projector edge flags start tolstart end tolend`: see `hb_edgedata_case`.
pub fn edgedata_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let p = Arc::new(projector(&mut r)?);
        attempt(|| edgedata_body(&mut r, p));
        Ok(())
    });
    end();
}

fn read_w_edges(r: &mut Reader, fd: &mut FaceData) -> Result<(), Failure> {
    let mut wi = r.integer();
    while wi > 0 {
        let (ewi, ei, eo) = (r.integer(), r.integer(), r.integer());
        let (a, b, c, dd) = (r.flag(), r.flag(), r.flag(), r.flag());
        fd.set_w_edge(wi, ewi, ei, Orientation::ALL[eo as usize], a, b, c, dd)?;
        wi = r.integer();
    }
    Ok(())
}

fn set_face_flags(fd: &mut FaceData, o: i32, cl: bool) {
    fd.set_back(true);
    fd.set_side(o == 1);
    fd.set_hiding(cl);
    fd.set_simple(true);
    fd.set_cut(false);
    fd.set_with_out_l(cl);
    fd.set_plane(true);
    fd.set_cylinder(o == 2);
    fd.set_cone(false);
    fd.set_sphere(true);
    fd.set_torus(cl);
    fd.set_size(2.5);
    fd.set_selected(false);
}

fn walk(fd: &FaceData) -> Result<(), Failure> {
    let mut it = FaceIterator::new();
    it.init_edge(fd)?;
    while it.more_edge() {
        int(it.edge()?);
        int(it.orientation()? as i32);
        for v in [it.out_line()?, it.internal()?, it.double()?, it.iso_line()?] {
            flag(v);
        }
        flag(it.beginning_of_wire());
        flag(it.end_of_wire());
        int(it.wire()?.get().map_or(0, |w| w.nb_edges()));
        it.next_edge()?;
    }
    Ok(())
}

fn facedata_body(r: &mut Reader) -> Result<(), Failure> {
    let f = face(r)?;
    let (o, cl, nw) = (r.integer(), r.flag(), r.integer());
    let mut fd = FaceData::new();
    flag(fd.selected());
    real(fd.size());
    fd.set(&f, Orientation::ALL[o as usize], cl, nw)?;
    for w in 1..=nw {
        fd.set_wire(w, r.integer())?;
    }
    read_w_edges(r, &mut fd)?;
    set_face_flags(&mut fd, o, cl);
    for v in [
        fd.selected(),
        fd.back(),
        fd.side(),
        fd.closed(),
        fd.hiding(),
        fd.simple(),
        fd.cut(),
        fd.with_out_l(),
        fd.plane(),
        fd.cylinder(),
        fd.cone(),
        fd.sphere(),
        fd.torus(),
    ] {
        flag(v);
    }
    real(fd.size());
    int(fd.orientation() as i32);
    real(f64::from(fd.tolerance()));
    int(fd.geometry().get_type() as i32);
    walk(&fd)
}

/// `face or closed nw (ne)* (wi ewi ei or outl inte dble isol)* -1`: see `hb_facedata_case`.
pub fn facedata_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| facedata_body(&mut r));
    end();
}

fn uv_of(e: &Edge, s: &Surface, projector: &Arc<Projector>, par: f64) -> Result<(), Failure> {
    let mut c = Curve::new();
    c.set_projector(Some(projector.clone()));
    c.set_curve(e)?;
    let (a, b) = (
        BCurveTool::first_parameter(c.curve())?,
        BCurveTool::last_parameter(c.curve())?,
    );
    attempt(|| {
        let uv = EdgeFaceTool::uv_point(a + (b - a) * par, &c, s)?;
        flag(uv.is_some());
        let (uu, vv) = uv.unwrap_or((-1., -1.));
        real(uu);
        real(vv);
        Ok(())
    });
    Ok(())
}

fn edgeface_body(r: &mut Reader, projector: Arc<Projector>) -> Result<(), Failure> {
    let f = face(r)?;
    let (u, v, d, par) = (r.real(), r.real(), r.dir(), r.real());
    let free = edge_v(r)?;
    let mut s = Surface::new();
    s.set_projector(Some(projector.clone()));
    s.set_surface(&f)?;
    show(EdgeFaceTool::curvature_value(&s, u, v, d), real);
    let ex = Explorer::new(&f, ShapeEnum::Edge, ShapeEnum::Shape)?;
    let first = topods::edge(ex.current())?;
    for e in [first, free] {
        uv_of(&e, &s, &projector, par)?;
    }
    let (oa, od) = poly_hlr_angle_and_deflection(par);
    real(oa);
    real(od);
    Ok(())
}

/// `projector face u v dir par edge`: see `hb_edgeface_case`.
pub fn edgeface_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let p = Arc::new(projector(&mut r)?);
        attempt(|| edgeface_body(&mut r, p));
        Ok(())
    });
    end();
}
