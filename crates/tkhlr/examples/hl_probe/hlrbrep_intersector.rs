//! HLRBRep_Intersector, the Rust twin of tools/oracle/tkhlr_probe_hlrbrep_intersector.cxx.
use super::common::{Reader, attempt, end, flag, int};
use super::hlrbrep_faces::{face, ics_point};
use super::hlrbrep_fixtures::projector;
use super::hlrbrep_records::edge_v;
use super::int_res2d::{ipoint, iseg};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkhlr::hlr_algo::{Projector, init_min_max};
use rustycad_tkhlr::hlr_brep::{EdgeData, Intersector, Surface};
use std::sync::Arc;

/// `hb_set_edge`.
fn set_edge(e: &Edge, p: &Arc<Projector>) -> Result<EdgeData, Failure> {
    let bc = BRepAdaptorCurve::with_edge(e)?;
    let (a, b) = (bc.first_parameter()?, bc.last_parameter()?);
    let mut d = EdgeData::new();
    d.set(
        false, false, e, 1, 2, false, false, false, false, a, 1e-7, b, 1e-7,
    )?;
    let c = d.change_geometry();
    c.set_projector(Some(p.clone()));
    let (mut mn, mut mx) = ([0.; 16], [0.; 16]);
    init_min_max(1e100, &mut mn, &mut mx);
    c.update(&mut mn, &mut mx)?;
    Ok(d)
}

fn inter_print(i: &Intersector) -> Result<(), Failure> {
    flag(i.is_done()?);
    let n = i.nb_points()?;
    int(n);
    for k in 1..=n {
        attempt(|| ipoint(&i.point(k)?));
    }
    let ns = i.nb_segments()?;
    int(ns);
    for k in 1..=ns {
        attempt(|| iseg(&i.segment(k)?));
    }
    Ok(())
}

fn cs_print(i: &Intersector) -> Result<(), Failure> {
    flag(i.is_done()?);
    let n = i.nb_points()?;
    int(n);
    for k in 1..=n {
        attempt(|| {
            ics_point(&i.cs_point(k)?);
            Ok(())
        });
    }
    int(i.nb_segments()?);
    Ok(())
}

fn intersector_body(r: &mut Reader, p: &Arc<Projector>) -> Result<(), Failure> {
    let (e1, e2) = (edge_v(r)?, edge_v(r)?);
    let (da1, db1, da2, db2) = (r.real(), r.real(), r.real(), r.real());
    let enbout = r.flag();
    let (u, v) = (r.real(), r.real());
    let f = face(r)?;
    let (d1, d2) = (set_edge(&e1, p)?, set_edge(&e2, p)?);
    let mut i = Intersector::new();
    attempt(|| {
        i.perform_self(&d1, da1, db1)?;
        inter_print(&i)
    });
    attempt(|| {
        i.perform_edges(1, &d1, da1, db1, 2, &d2, da2, db2, enbout)?;
        inter_print(&i)
    });
    attempt(|| {
        i.simulate_one_point(&d1, u, &d2, v)?;
        inter_print(&i)
    });
    let mut s = Surface::new();
    s.set_projector(Some(p.clone()));
    s.set_surface(&f)?;
    i.load();
    for _ in 0..3 {
        let (x, y, pp) = (r.real(), r.real(), r.real());
        attempt(|| {
            i.perform_line(&s, &p.shoot(x, y)?, pp)?;
            cs_print(&i)
        });
    }
    Ok(())
}

/// `projector edge1 edge2 da1 db1 da2 db2 enbout u v face (x y P)*3`: see
/// `hb_intersector_case`.
pub fn intersector_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let p = Arc::new(projector(&mut r)?);
        attempt(|| intersector_body(&mut r, &p));
        Ok(())
    });
    end();
}
