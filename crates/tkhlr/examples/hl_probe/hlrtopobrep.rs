//! HLRTopoBRep (OutLiner, DSFiller, FaceIsoLiner, Data), the Rust twin of
//! tools/oracle/tkhlr_probe_hlrtopobrep.cxx.
use super::common::{Reader, attempt, end, flag, int};
use super::hlrbrep_fixtures::projector;
use super::hlrbrep_records::edge_summary;
use super::hlrbrep_shapes::shape;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkhlr::hlr_topo_brep::{Data, MapOfShapeTool, OutLiner};

fn explored(s: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let mut out = Vec::new();
    let mut ex = Explorer::new(s, kind, ShapeEnum::Shape)?;
    while ex.more() {
        out.push(ex.current().clone());
        ex.next()?;
    }
    Ok(out)
}

/// `hb_shape_edges`.
pub fn shape_edges(s: &Shape) -> Result<(), Failure> {
    let edges = explored(s, ShapeEnum::Edge)?;
    int(edges.len() as i32);
    for e in &edges {
        int(e.orientation() as i32);
        attempt(|| edge_summary(&topods::edge(e)?));
    }
    Ok(())
}

/// `hb_shape_topology`.
pub fn shape_topology(s: &Shape) -> Result<(), Failure> {
    for f in explored(s, ShapeEnum::Face)? {
        let wires = explored(&f, ShapeEnum::Wire)?;
        for w in &wires {
            int(explored(w, ShapeEnum::Edge)?.len() as i32);
        }
        int(-(wires.len() as i32));
    }
    Ok(())
}

fn topo_data(ds: &Data, s: &Shape) -> Result<(), Failure> {
    for f in explored(s, ShapeEnum::Face)? {
        let f = topods::face(&f)?;
        flag(ds.face_has_int_l(&f));
        flag(ds.face_has_out_l(&f));
        flag(ds.face_has_iso_l(&f));
        attempt(|| {
            int(ds.face_int_l(&f)?.extent());
            int(ds.face_out_l(&f)?.extent());
            int(ds.face_iso_l(&f)?.extent());
            Ok(())
        });
    }
    for e in explored(s, ShapeEnum::Edge)? {
        let e = topods::edge(&e)?;
        flag(ds.edge_has_spl_e(&e));
        attempt(|| {
            int(ds.edge_spl_e(&e)?.extent());
            Ok(())
        });
        let (v1, v2) = topexp::vertices(&e, false);
        for v in [v1, v2] {
            if !v.is_null() {
                flag(ds.is_out_v(&v));
                flag(ds.is_int_v(&v));
            }
        }
    }
    Ok(())
}

fn outliner_body(r: &mut Reader) -> Result<(), Failure> {
    let p = projector(r)?;
    attempt(|| {
        let s = shape(r)?;
        let nb_iso = r.integer();
        let o = Handle::new(OutLiner::with_original(&s));
        let mut mst = MapOfShapeTool::default();
        o.fill(&p, &mut mst, nb_iso)?;
        let out = o.out_lined_shape();
        flag(out.is_null());
        shape_topology(&out)?;
        shape_edges(&out)?;
        topo_data(&o.data_structure(), &s)
    });
    Ok(())
}

/// `projector shape nbIso`: see `ht_outliner_case`.
pub fn outliner_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| outliner_body(&mut r));
    end();
}
