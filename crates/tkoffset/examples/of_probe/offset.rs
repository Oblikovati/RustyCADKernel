//! BRepOffset_Offset probe operations, the Rust twin of
//! tools/oracle/tkoffset_probe_brepoffset_offset.cxx (the records are documented there).
use super::common::{Reader, flag, int};
use super::shape::shape;
use super::tool::{finish, primitive};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::{
    IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, ShapeMapHasher,
};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::{JoinType, Shape as Continuity};
use rustycad_tkoffset::brep_offset::{BRepOffsetOffset, CreatedShapes, PipeOptions};

/// The status and face of an offset, with the image of each edge of its initial face.
fn print_offset(o: &BRepOffsetOffset, source: &Shape) -> Result<(), Failure> {
    int(o.status() as i32);
    shape(o.face().as_ref())?;
    if source.is_null() {
        return Ok(());
    }
    for e in Explorer::new(source, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let g = o.generated(&e);
        flag(g.is_null());
        if !g.is_null() {
            int(g.orientation() as i32);
        }
    }
    Ok(())
}

/// Prints the exception type of a failed request part.
fn caught(result: Result<(), Failure>) {
    if let Err(error) = result {
        print!("{} ", error.exception_type());
    }
}

/// Binds the edge images of an offset face not yet bound.
fn share(o: &BRepOffsetOffset, f: &Shape, created: &mut CreatedShapes) -> Result<(), Failure> {
    for e in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let g = o.generated(&e);
        if !g.is_null() && created.seek(&e).is_none() {
            created.bind(e, g);
        }
    }
    Ok(())
}

/// The continuity of its enumeration value.
fn continuity_of(value: i32) -> Continuity {
    match value {
        0 => Continuity::C0,
        1 => Continuity::G1,
        2 => Continuity::C1,
        3 => Continuity::G2,
        4 => Continuity::C2,
        5 => Continuity::C3,
        _ => Continuity::CN,
    }
}

/// `oo_faces`.
pub fn faces_case(values: &[f64]) {
    let mut r = Reader::new(values);
    finish((|| {
        let p = primitive(&mut r)?;
        let off = r.real();
        let outside = r.flag();
        let join = if r.flag() {
            JoinType::Intersection
        } else {
            JoinType::Arc
        };
        let shared = r.flag();
        let mut created = CreatedShapes::default();
        for fs in Explorer::new(&p, ShapeEnum::Face, ShapeEnum::Shape)? {
            caught((|| {
                let f = topods::face(&fs)?;
                let o = BRepOffsetOffset::from_face_created(&f, off, &created, outside, join)?;
                print_offset(&o, &fs)?;
                if shared {
                    share(&o, &fs, &mut created)?;
                }
                Ok(())
            })());
        }
        Ok(())
    })());
}

/// `oo_pipes`.
pub fn pipes_case(values: &[f64]) {
    let mut r = Reader::new(values);
    finish((|| {
        let p = primitive(&mut r)?;
        let off = r.real();
        let polynomial = r.flag();
        let tol = r.real();
        let conti = continuity_of(r.integer());
        let connectors = r.flag();
        let options = PipeOptions {
            polynomial,
            tol,
            conti,
        };
        let created = CreatedShapes::default();
        let mut faces: DataMap<Shape, BRepOffsetOffset, ShapeMapHasher> = DataMap::default();
        for fs in Explorer::new(&p, ShapeEnum::Face, ShapeEnum::Shape)? {
            let f = topods::face(&fs)?;
            let o = BRepOffsetOffset::from_face_created(&f, off, &created, true, JoinType::Arc)?;
            faces.bind(fs, o);
        }
        let mut ef = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(&p, ShapeEnum::Edge, ShapeEnum::Face, &mut ef)?;
        let mut vconn: DataMap<Shape, List<Shape>, ShapeMapHasher> = DataMap::default();
        for i in 1..=ef.extent() {
            let e = topods::edge(ef.find_key(i)?)?;
            let lf = ef.find_from_index(i)?;
            let (Some(first), Some(last)) = (lf.first(), lf.last()) else {
                int(-1);
                continue;
            };
            if lf.len() != 2 || first.is_same(last) || BRepTool::degenerated(&e)? {
                int(-1);
                continue;
            }
            caught((|| {
                let image = |f: &Shape| -> Result<topods::Edge, Failure> {
                    let o = faces
                        .seek(f)
                        .ok_or_else(|| Failure::with_message("Standard_NoSuchObject", "face"))?;
                    topods::edge(&o.generated(e.as_ref()))
                };
                let (r1, r2) = (image(first)?, image(last)?);
                let o = BRepOffsetOffset::from_edges(&e, &r1, &r2, off, options)?;
                print_offset(&o, &Shape::default())?;
                let (v1, v2) = topexp::vertices(&e, false);
                let g1 = o.generated(v1.as_ref());
                let g2 = o.generated(v2.as_ref());
                shape(&g1)?;
                shape(&g2)?;
                if !g1.is_null() {
                    vconn
                        .try_emplaced(v1.as_ref().clone(), List::default)
                        .append(g1.clone());
                }
                if !g2.is_null() && !v2.is_same(&v1) {
                    vconn
                        .try_emplaced(v2.as_ref().clone(), List::default)
                        .append(g2.clone());
                }
                if connectors {
                    let o2 = BRepOffsetOffset::from_edges_with_connectors(
                        &e,
                        &r1,
                        &r2,
                        off,
                        &topods::edge(&g1)?,
                        &topods::edge(&g2)?,
                        options,
                    )?;
                    print_offset(&o2, &Shape::default())?;
                }
                Ok(())
            })());
        }
        let mut vs = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(&p, ShapeEnum::Vertex, &mut vs)?;
        for i in 1..=vs.extent() {
            let v = vs.find_key(i)?;
            let Some(l) = vconn.seek(v).filter(|l| l.len() >= 3) else {
                int(-1);
                continue;
            };
            caught((|| {
                let o = BRepOffsetOffset::from_vertex(&topods::vertex(v)?, l, off, options)?;
                int(o.status() as i32);
                shape(o.face().as_ref())
            })());
        }
        Ok(())
    })());
}

/// `oo_tubes`.
pub fn tubes_case(values: &[f64]) {
    let mut r = Reader::new(values);
    finish((|| {
        let p = primitive(&mut r)?;
        let off = r.real();
        let mut es = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(&p, ShapeEnum::Edge, &mut es)?;
        for i in 1..=es.extent() {
            let e = topods::edge(es.find_key(i)?)?;
            if BRepTool::degenerated(&e)? {
                int(-1);
                continue;
            }
            caught((|| {
                let mut o = BRepOffsetOffset::new();
                o.init_edge(&e, off)?;
                shape(o.face().as_ref())?;
                let (v1, v2) = topexp::vertices(&e, false);
                shape(&o.generated(v1.as_ref()))?;
                shape(&o.generated(v2.as_ref()))
            })());
        }
        Ok(())
    })());
}
