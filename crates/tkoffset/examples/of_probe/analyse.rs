//! BRepOffset_Analyse probe operations, the Rust twin of
//! tools/oracle/tkoffset_probe_brepoffset_analyse.cxx (the records are documented there).
use super::common::{Reader, attempt, end, flag, int, real};
use super::make_offset::failed;
use super::shape::shape;
use super::tool::{primitive, sub};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkoffset::brep_offset::BRepOffsetAnalyse;

/// The map index and orientation of a shape (0 when it is not in the map).
fn reference(all: &IndexedMapOfShape, s: &Shape) {
    int(all.find_index(s) as i32);
    int(s.orientation() as i32);
}

/// The concavity of its enumeration value.
fn concavity(t: i32) -> TypeOfConcavity {
    match t {
        0 => TypeOfConcavity::Concave,
        1 => TypeOfConcavity::Convex,
        2 => TypeOfConcavity::Tangential,
        3 => TypeOfConcavity::FreeBound,
        4 => TypeOfConcavity::Other,
        _ => TypeOfConcavity::Mixed,
    }
}

/// `oan_analyse prim angle offset n (face offset)...`.
pub fn analyse_case(values: &[f64]) {
    let mut r = Reader::new(values);
    if let Err(error) = analyse_body(&mut r) {
        failed(error);
        return;
    }
    end();
}

fn analyse_body(r: &mut Reader<'_>) -> Result<(), Failure> {
    let p = primitive(r)?;
    let (angle, off) = (r.real(), r.real());
    let n = r.integer();
    let mut offsets: DataMap<Shape, f64, ShapeMapHasher> = DataMap::default();
    for _ in 0..n {
        let f = sub(&p, ShapeEnum::Face, r.integer())?;
        offsets.bind(f, r.real());
    }
    let mut a = BRepOffsetAnalyse::new();
    a.set_offset_value(off);
    a.set_face_offset_map(&offsets);
    a.perform(&p, angle)?;
    flag(a.is_done());
    let mut all = IndexedMapOfShape::default();
    topexp::map_shapes(&p, &mut all, true, true);
    int(a.new_faces().len() as i32);
    for nf in a.new_faces() {
        topexp::map_shapes(nf, &mut all, true, true);
        reference(&all, nf);
        shape(nf)?;
    }
    for i in 1..=all.extent() {
        let s = all.find_key(i)?.clone();
        if s.shape_type()? == ShapeEnum::Edge {
            attempt(|| {
                let l = a.type_of(&topods::edge(&s)?)?;
                int(l.len() as i32);
                for iv in l {
                    real(iv.first()?);
                    real(iv.last()?);
                    int(iv.get_type()? as i32);
                }
                Ok(())
            });
        }
        attempt(|| {
            let l = a.ancestors(&s)?;
            int(l.len() as i32);
            for x in l {
                reference(&all, x);
            }
            Ok(())
        });
        let g = a.generated(&s);
        if g.is_null() {
            int(-1);
        } else {
            reference(&all, &g);
        }
        match a.descendants(&s, false) {
            None => int(-1),
            Some(d) => {
                int(d.len() as i32);
                for x in d {
                    reference(&all, x);
                }
            }
        }
    }
    for f in Explorer::new(&p, ShapeEnum::Face, ShapeEnum::Shape)? {
        let f = topods::face(&f)?;
        for e in Explorer::new(&f, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let e = topods::edge(&e)?;
            reference(&all, a.edge_replacement(&f, &e));
        }
    }
    for t1 in 0..=5 {
        for t2 in t1..=5 {
            attempt(|| {
                let mut groups = Vec::new();
                if t1 == t2 {
                    a.explode(concavity(t1), &mut groups)?;
                } else {
                    a.explode_two_types(concavity(t1), concavity(t2), &mut groups)?;
                }
                int(groups.len() as i32);
                for g in &groups {
                    let mut fm = IndexedMapOfShape::default();
                    topexp::map_shapes_of_type(g, ShapeEnum::Face, &mut fm)?;
                    int(fm.extent() as i32);
                    for k in 1..=fm.extent() {
                        reference(&all, fm.find_key(k)?);
                    }
                }
                Ok(())
            });
        }
    }
    for i in 1..=all.extent() {
        let s = all.find_key(i)?;
        flag(a.has_ancestor(s));
        flag(a.has_generated(s));
    }
    let refs = |l: &[Shape]| {
        int(l.len() as i32);
        for x in l {
            reference(&all, x);
        }
    };
    for t in 0..=5 {
        for v in Explorer::new(&p, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            attempt(|| {
                let mut l = Vec::new();
                a.edges_on_vertex(&topods::vertex(&v)?, concavity(t), &mut l)?;
                refs(&l);
                Ok(())
            });
        }
        for f in Explorer::new(&p, ShapeEnum::Face, ShapeEnum::Shape)? {
            attempt(|| {
                let mut l = Vec::new();
                a.edges_on_face(&topods::face(&f)?, concavity(t), &mut l)?;
                refs(&l);
                Ok(())
            });
        }
    }
    for e in Explorer::new(&p, ShapeEnum::Edge, ShapeEnum::Shape)? {
        for v in Explorer::new(&e, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            attempt(|| {
                let mut l = Vec::new();
                a.tangent_edges(&topods::edge(&e)?, &topods::vertex(&v)?, &mut l)?;
                refs(&l);
                Ok(())
            });
        }
    }
    Ok(())
}
