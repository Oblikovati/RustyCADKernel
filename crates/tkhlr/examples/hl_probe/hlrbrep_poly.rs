//! HLRBRep_PolyAlgo and HLRBRep_PolyHLRToShape, the Rust twin of
//! tools/oracle/tkhlr_probe_hlrbrep_poly.cxx (with the same deterministic mesher).
use super::common::{Reader, attempt, end, flag, int, real};
use super::hlrbrep_algo::compound;
use super::hlrbrep_fixtures::projector;
use super::hlrbrep_shapes::shape;
#[path = "../../tests/support/grid_mesh.rs"]
mod grid_mesh;
use grid_mesh::mesh;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkhlr::hlr_algo::{BiPointPoints, EdgeStatus, Projector};
use rustycad_tkhlr::hlr_brep::{PolyAlgo, PolyHLRToShape};

fn segment(p: &BiPointPoints, s: &Shape, map: &IndexedMapOfShape, flags: [bool; 4]) {
    for x in [p.pnt1, p.pnt2, p.pnt_p1, p.pnt_p2] {
        real(x.x());
        real(x.y());
        real(x.z());
    }
    int(map.find_index(s) as i32);
    flags.into_iter().for_each(flag);
}

fn walks(a: &PolyAlgo, map: &IndexedMapOfShape) -> Result<(), Failure> {
    let mut st = EdgeStatus::new();
    let mut nh = 0;
    a.init_hide()?;
    while a.more_hide() && nh < 3000 {
        let seg = a.hide(&mut st)?;
        segment(
            &seg.points,
            &seg.shape,
            map,
            [seg.reg1, seg.regn, seg.outl, seg.intl],
        );
        let nv = st.nb_visible_part();
        int(nv);
        for k in 1..=nv {
            let (x, _, y, _) = st.visible_part(k)?;
            real(x);
            real(y);
        }
        a.next_hide()?;
        nh += 1;
    }
    int(nh);
    let mut ns = 0;
    a.init_show()?;
    while a.more_show() && ns < 3000 {
        let seg = a.show()?;
        segment(
            &seg.points,
            &seg.shape,
            map,
            [seg.reg1, seg.regn, seg.outl, seg.intl],
        );
        a.next_show()?;
        ns += 1;
    }
    int(ns);
    Ok(())
}

fn poly_body(r: &mut Reader, p: &Projector) -> Result<(), Failure> {
    let s = shape(r)?;
    let (n, m, tc) = (r.integer(), r.integer(), r.real());
    mesh(&s, n, m)?;
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(&s, ShapeEnum::Edge, &mut map)?;
    topexp::map_shapes_of_type(&s, ShapeEnum::Face, &mut map)?;
    let handle = Handle::new(PolyAlgo::with_shape(&s)?);
    let a = handle.get().ok_or_else(|| Failure::new("no algo"))?;
    a.set_projector(p);
    a.set_tol_coef(tc);
    a.update()?;
    walks(a, &map)?;
    let mut t = PolyHLRToShape::new();
    t.update(&handle)?;
    for mode in 0..2 {
        if mode == 1 {
            t.show();
        }
        attempt(|| compound(&t.v_compound()?));
        attempt(|| compound(&t.h_compound()?));
        attempt(|| compound(&t.out_line_v_compound()?));
        attempt(|| compound(&t.rg1_line_v_compound()?));
        attempt(|| compound(&t.rgn_line_h_compound()?));
        attempt(|| compound(&t.v_compound_of(&s)?));
    }
    attempt(|| compound(&a.out_lined_shape(&s)?));
    Ok(())
}

/// `projector shape n m tolcoef`: see `hb_poly_case`.
pub fn poly_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let p = projector(&mut r)?;
        attempt(|| poly_body(&mut r, &p));
        Ok(())
    });
    end();
}
