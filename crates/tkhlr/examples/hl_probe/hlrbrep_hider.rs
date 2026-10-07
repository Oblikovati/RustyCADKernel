//! HLRBRep_Hider, the Rust twin of `hb_hider_case` in
//! tools/oracle/tkhlr_probe_hlrbrep_data.cxx.
use super::common::{Reader, attempt, end, flag, int, real};
use super::hlrbrep_data::shape_box;
use super::hlrbrep_fixtures::projector;
use super::hlrbrep_shapes::shape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkhlr::hlr_algo::Projector;
use rustycad_tkhlr::hlr_brep::{Data, Hider, ShapeToHLR};
use rustycad_tkhlr::hlr_topo_brep::{MapOfShapeTool, OutLiner};

/// `hb_edge_status`: the visible parts of every edge.
pub fn edge_status(ds: &Data) -> Result<(), Failure> {
    let ed = ds.e_data_array();
    for e in 1..=ds.nb_edges() {
        let st = ed.value(e).map_err(|x| x.into_failure("probe"))?.status();
        flag(st.all_hidden());
        flag(st.all_visible());
        let n = st.nb_visible_part();
        int(n);
        for k in 1..=n {
            let (a, ta, b, tb) = st.visible_part(k)?;
            real(a);
            real(f64::from(ta));
            real(b);
            real(f64::from(tb));
        }
    }
    Ok(())
}

fn hider_body(r: &mut Reader, p: &Projector) -> Result<(), Failure> {
    let s = shape(r)?;
    let nb_iso = r.integer();
    let o = Handle::new(OutLiner::with_original(&s));
    let mut mst = MapOfShapeTool::default();
    let handle = ShapeToHLR::load(&o, p, &mut mst, nb_iso)?;
    let ds = handle.get().ok_or_else(|| Failure::new("no data"))?;
    ds.update(p)?;
    let (ne, nf) = (ds.nb_edges(), ds.nb_faces());
    if ne == 0 {
        return Ok(());
    }
    ds.init_bound_sort(&shape_box(ds, ne, nf)?, 1, ne)?;
    let h = Hider::new(&handle);
    for f in 1..=nf {
        if ds.f_data_array()[f].hiding() {
            attempt(|| h.hide(f, &mut mst));
        }
    }
    edge_status(ds)
}

/// `projector shape nbIso`: see `hb_hider_case`.
pub fn hider_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let p = projector(&mut r)?;
        attempt(|| hider_body(&mut r, &p));
        Ok(())
    });
    end();
}
