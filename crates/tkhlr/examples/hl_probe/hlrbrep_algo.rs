//! HLRBRep_Algo, HLRBRep_InternalAlgo, HLRBRep_HLRToShape and HLRAppli_ReflectLines,
//! the Rust twin of tools/oracle/tkhlr_probe_hlrbrep_algo.cxx.
use super::common::{Reader, attempt, end, flag, int};
use super::hlrbrep_fixtures::projector;
use super::hlrbrep_records::edge_summary;
use super::hlrbrep_shapes::shape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkhlr::hlr_algo::{MinMaxIndices, Projector};
use rustycad_tkhlr::hlr_appli::ReflectLines;
use rustycad_tkhlr::hlr_brep::{Algo, HLRToShape, TypeOfResultingEdge};

fn minmax(m: &MinMaxIndices) {
    m.min.iter().chain(m.max.iter()).for_each(|v| int(*v));
}

/// `hb_compound`: the edges of a result compound.
pub fn compound(c: &Shape) -> Result<(), Failure> {
    flag(c.is_null());
    if c.is_null() {
        return Ok(());
    }
    let edges: Vec<Shape> = Explorer::new(c, ShapeEnum::Edge, ShapeEnum::Shape)?.collect();
    int(edges.len() as i32);
    for e in &edges {
        attempt(|| edge_summary(&topods::edge(e)?));
    }
    Ok(())
}

const TYPES: [TypeOfResultingEdge; 5] = [
    TypeOfResultingEdge::IsoLine,
    TypeOfResultingEdge::OutLine,
    TypeOfResultingEdge::Rg1Line,
    TypeOfResultingEdge::RgNLine,
    TypeOfResultingEdge::Sharp,
];

fn to_shape(a: &Handle<Algo>, first: &Shape) {
    let t = HLRToShape::new(a);
    for ty in TYPES {
        for v in [true, false] {
            attempt(|| compound(&t.compound_of_edges(ty, v, false)?));
        }
    }
    attempt(|| compound(&t.out_line_v_compound_3d()?));
    attempt(|| compound(&t.v_compound_of(first)?));
    attempt(|| compound(&t.h_compound_of(first)?));
}

fn algo_body(r: &mut Reader, p: &Projector) -> Result<(), Failure> {
    let n = r.integer();
    let mut shapes = Vec::new();
    for _ in 0..n {
        shapes.push(shape(r)?);
    }
    let (nb_iso, mode) = (r.integer(), r.integer());
    let handle = Handle::new(Algo::new()?);
    let a = handle.get().ok_or_else(|| Failure::new("no algo"))?;
    for s in &shapes {
        a.add(s, nb_iso);
    }
    a.set_projector(p);
    a.update()?;
    int(a.nb_shapes());
    let ds = a.data_structure();
    flag(ds.is_null());
    if let Some(d) = ds.get() {
        int(d.nb_edges());
        int(d.nb_faces());
    }
    for k in 1..=a.nb_shapes() {
        let sb = a.shape_bounds(k)?;
        let (v1, v2, e1, e2, f1, f2) = sb.bounds();
        [v1, v2, e1, e2, f1, f2].into_iter().for_each(int);
        minmax(sb.min_max());
        int(a.index(&shapes[k as usize - 1]));
    }
    match mode {
        0 => attempt(|| a.hide()),
        1 => attempt(|| a.partial_hide()),
        _ => {
            attempt(|| a.hide_shape(1));
            if n > 1 {
                attempt(|| a.hide_shapes(1, 2));
            }
            a.select()?;
        }
    }
    to_shape(&handle, &shapes[0]);
    Ok(())
}

/// `projector nshapes shape* nbIso mode`: see `hb_algo_case`.
pub fn algo_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let p = projector(&mut r)?;
        attempt(|| algo_body(&mut r, &p));
        Ok(())
    });
    end();
}

fn reflect_body(r: &mut Reader) -> Result<(), Failure> {
    let s = shape(r)?;
    let mut v = [0.; 9];
    v.iter_mut().for_each(|x| *x = r.real());
    let mut l = ReflectLines::new(&s)?;
    l.set_axes(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8])?;
    l.perform()?;
    attempt(|| compound(&l.get_result()?));
    for vis in [true, false] {
        attempt(|| compound(&l.get_compound_of_3d_edges(TypeOfResultingEdge::Sharp, vis, true)?));
    }
    Ok(())
}

/// `shape N At Up`: see `hb_reflect_case`.
pub fn reflect_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| reflect_body(&mut r));
    end();
}
