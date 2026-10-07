//! HLRTopoBRep_OutLiner.cxx: ProcessFace, the copy of a face with its split edges
//! and its outlines and isolines as internal wires.
use super::OutLiner;
use crate::hlr_topo_brep::{Data, MapOfShapeTool};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Wire};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::extrema::ExtPC;
use rustycad_tkmath::geom_abs::CurveType;

impl OutLiner {
    // ProcessFace(F, S, MST): the face rebuilt in `s` with the split edges, its
    // internal outlines (an outline that is an edge of the face becomes an outline on
    // restriction) and its isolines.
    pub(super) fn process_face(
        &self,
        f: &Face,
        s: &mut Shape,
        mst: &mut MapOfShapeTool,
    ) -> Result<(), Failure> {
        let b = BRepBuilder::default();
        let mut a_ve_map = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(f, ShapeEnum::Vertex, ShapeEnum::Edge, &mut a_ve_map)?;
        let mut nf = f.empty_copied()?;
        let mut ds = self.data_structure();
        let mut exw = Explorer::new(f, ShapeEnum::Wire, ShapeEnum::Shape)?;
        while exw.more() {
            let w = rebuilt_wire(exw.current(), &mut ds)?;
            b.add(&mut nf, &w)?;
            exw.next()?;
        }
        ds.add_int_l(f);
        ds.add_out_l(f);
        if ds.face_has_int_l(f) {
            let int_l: Vec<Shape> = ds.face_int_l(f)?.iter().cloned().collect();
            let w = internal_outlines(&int_l, f, &a_ve_map, &mut ds)?;
            if !w.is_null() {
                b.add(&mut nf, &w)?;
            }
        }
        if ds.face_has_iso_l(f) {
            let iso_l: Vec<Shape> = ds.face_iso_l(f)?.iter().cloned().collect();
            let mut w = Wire::default();
            for e in &iso_l {
                internal_wire_edge(&topods::edge(e)?, f, &mut w, &mut ds)?;
            }
            if !w.is_null() {
                b.add(&mut nf, &w)?;
            }
        }
        ds.add_old_s(&nf, f);
        let tool = mst.find(f)?.clone();
        mst.bind(nf.clone(), tool);
        b.add(s, &nf)
    }
}

// A wire of the face with each split edge replaced by its parts (oriented as the
// edge and recorded with it as original).
fn rebuilt_wire(wire: &Shape, ds: &mut Data) -> Result<Wire, Failure> {
    let b = BRepBuilder::default();
    let mut w = Wire::default();
    b.make_wire(&mut w);
    let mut exe = Explorer::new(wire, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while exe.more() {
        let e = topods::edge(exe.current())?;
        if ds.edge_has_spl_e(&e) {
            let parts: Vec<Shape> = ds.edge_spl_e(&e)?.iter().cloned().collect();
            for part in parts {
                let mut new_e = topods::edge(&part)?;
                new_e.set_orientation(e.orientation());
                ds.add_old_s(&new_e, &e);
                b.add(&mut w, &new_e)?;
            }
        } else {
            b.add(&mut w, &e)?;
        }
        exe.next()?;
    }
    Ok(w)
}

// The internal outlines of the face as one internal wire (null when every outline
// is an edge of the face, which becomes an outline on restriction instead).
fn internal_outlines(
    int_l: &[Shape],
    f: &Face,
    a_ve_map: &IndexedDataMapOfShapeListOfShape,
    ds: &mut Data,
) -> Result<Wire, Failure> {
    let mut w = Wire::default();
    for s in int_l {
        let mut e = topods::edge(s)?;
        e.set_orientation(Orientation::Internal);
        // Check, if outline edge coincides real edge
        if same_as_face_edge(&e, a_ve_map)? {
            ds.add_out_l(f).append(Shape::from(e));
            continue;
        }
        internal_wire_edge(&e, f, &mut w, ds)?;
    }
    Ok(w)
}

// The parts of `e` (or `e`) as INTERNAL edges of `w` (made when null), each recorded
// with the face as original.
fn internal_wire_edge(e: &Edge, f: &Face, w: &mut Wire, ds: &mut Data) -> Result<(), Failure> {
    let b = BRepBuilder::default();
    let mut e = e.clone();
    e.set_orientation(Orientation::Internal);
    let parts: Vec<Shape> = if ds.edge_has_spl_e(&e) {
        ds.edge_spl_e(&e)?.iter().cloned().collect()
    } else {
        vec![Shape::from(e)]
    };
    for mut new_e in parts {
        new_e.set_orientation(Orientation::Internal);
        if w.is_null() {
            b.make_wire(w);
        }
        ds.add_old_s(&new_e, f);
        b.add(w, &new_e)?;
    }
    Ok(())
}

// Whether the outline joins the same vertices as an edge of the face and is that
// edge: both lines, or a point at 0.34/0.66 of the outline within 1e-7 of it.
fn same_as_face_edge(
    e: &Edge,
    a_ve_map: &IndexedDataMapOfShapeListOfShape,
) -> Result<bool, Failure> {
    let c = BRepAdaptorCurve::with_edge(e)?;
    let par = 0.34 * c.first_parameter()? + 0.66 * c.last_parameter()?;
    let p = c.value(par)?;
    let (v1, v2) = topexp::vertices(e, false);
    if v1.is_null() || !a_ve_map.contains(&v1) {
        return Ok(false);
    }
    for a_e in a_ve_map.find_from_key(&v1)?.iter() {
        let a_e = topods::edge(a_e)?;
        let (a_v1, a_v2) = topexp::vertices(&a_e, false);
        if !((v1.is_same(&a_v1) && v2.is_same(&a_v2)) || (v1.is_same(&a_v2) && v2.is_same(&a_v1))) {
            continue;
        }
        let a_c = BRepAdaptorCurve::with_edge(&a_e)?;
        if c.get_type()? == CurveType::Line && a_c.get_type()? == CurveType::Line {
            return Ok(true);
        }
        // Try to project one point
        let an_ext = ExtPC::with_curve(p, &a_c, 1.0e-10)?;
        if an_ext.is_done() && an_ext.nb_ext()? > 0 {
            let mut dist = real_last();
            for ec in 1..=an_ext.nb_ext()? {
                dist = std_min(dist, an_ext.square_distance(ec)?);
            }
            if dist <= 1.0e-14 {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
