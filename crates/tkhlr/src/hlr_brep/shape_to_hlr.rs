//! HLRBRep_ShapeToHLR.hxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::Data;
use crate::hlr_algo::Projector;
use crate::hlr_topo_brep::{Data as TopoData, MapOfShapeTool, OutLiner};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::standard::real::{epsilon, real_first, real_last};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// HLRBRep_ShapeToHLR: builds the data structure of the hidden line algorithm from
/// an outlined shape: one edge record per edge, one face record per face with its
/// wires; `ShapeToHLR::load(&outliner, &projector, &mut tools, 0)?`.
pub struct ShapeToHLR;

// The vertex data of an edge end: index, outline and internal flags, parameter and
// tolerance (the infinite parameter of an end without vertex).
struct EdgeEnd {
    index: i32,
    out: bool,
    cut: bool,
    param: f64,
    tol: f32,
}

fn edge_end(
    v: &Vertex,
    param: f64,
    infinite: f64,
    vertices: &IndexedDataMapOfShapeListOfShape,
    top_ds: &TopoData,
) -> Result<EdgeEnd, Failure> {
    if v.is_null() {
        return Ok(EdgeEnd {
            index: 0,
            out: false,
            cut: false,
            param: infinite,
            tol: epsilon(infinite) as f32,
        });
    }
    Ok(EdgeEnd {
        index: vertices.find_index(&Shape::from(v.clone())) as i32,
        out: top_ds.is_out_v(v),
        cut: top_ds.is_int_v(v),
        param,
        tol: BRepTool::vertex_tolerance(v)? as f32,
    })
}

// The faces of the shells, then the faces outside shells, once each.
fn face_map(s: &Shape) -> Result<IndexedMapOfShape, Failure> {
    let mut fm = IndexedMapOfShape::default();
    let mut exshell = Explorer::new(s, ShapeEnum::Shell, ShapeEnum::Shape)?;
    while exshell.more() {
        let mut exface = Explorer::new(exshell.current(), ShapeEnum::Face, ShapeEnum::Shape)?;
        while exface.more() {
            if !fm.contains(exface.current()) {
                fm.add(exface.current().clone());
            }
            exface.next()?;
        }
        exshell.next()?;
    }
    let mut exface = Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shell)?;
    while exface.more() {
        if !fm.contains(exface.current()) {
            fm.add(exface.current().clone());
        }
        exface.next()?;
    }
    Ok(fm)
}

// Whether the edge is G1 (and G2) between its two faces.
fn regularity(
    edg: &Edge,
    edges_to_faces: &IndexedDataMapOfShapeListOfShape,
) -> Result<(bool, bool), Failure> {
    let inde = edges_to_faces.find_index(&Shape::from(edg.clone()));
    if inde == 0 {
        return Ok((false, false));
    }
    let faces = edges_to_faces.find_from_index(inde)?;
    if faces.extent() != 2 {
        return Ok((false, false));
    }
    let mut it = faces.iter();
    let (f1, f2) = match (it.next(), it.next()) {
        (Some(a), Some(b)) => (topods::face(a)?, topods::face(b)?),
        _ => return Ok((false, false)),
    };
    let rg = BRepTool::continuity_on_faces(edg, &f1, &f2)?;
    Ok((rg >= Continuity::G1, rg >= Continuity::G2))
}

impl ShapeToHLR {
    /// Load(S, P, MST, nbIso): fills the outliner, then creates the data structure
    /// with every edge of the outlined shape (its regularity, end vertices with
    /// their outline flags, range and tolerances) and every face;
    /// `ShapeToHLR::load(&outliner, &projector, &mut tools, 0)?`.
    pub fn load(
        s: &Handle<OutLiner>,
        p: &Projector,
        mst: &mut MapOfShapeTool,
        nb_iso: i32,
    ) -> Result<Handle<Data>, Failure> {
        let outliner = s.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "HLRBRep_ShapeToHLR::Load: no outliner; expected a HLRTopoBRep_OutLiner handle",
            )
        })?;
        outliner.fill(p, mst, nb_iso)?;
        let out_s = outliner.out_lined_shape();
        let fm = face_map(&out_s)?;
        let mut em = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(&out_s, ShapeEnum::Edge, &mut em)?;
        let nb_edge = em.extent() as i32;
        let mut vertices_to_edges = IndexedDataMapOfShapeListOfShape::default();
        for i in 1..=nb_edge {
            topexp::map_shapes_and_ancestors(
                em.find_key(i as usize)?,
                ShapeEnum::Vertex,
                ShapeEnum::Edge,
                &mut vertices_to_edges,
            )?;
        }
        let nb_vert = vertices_to_edges.extent() as i32;
        let nb_face = fm.extent() as i32;
        let ds = Handle::new(Data::new(nb_vert, nb_edge, nb_face)?);
        let mut edges_to_faces = IndexedDataMapOfShapeListOfShape::default();
        for i in 1..=nb_face {
            topexp::map_shapes_and_ancestors(
                fm.find_key(i as usize)?,
                ShapeEnum::Edge,
                ShapeEnum::Face,
                &mut edges_to_faces,
            )?;
        }
        let data = ds
            .get()
            .ok_or_else(|| Failure::new("HLRBRep_ShapeToHLR::Load: no data structure"))?;
        {
            let top_ds = outliner.data_structure();
            for i in 1..=nb_edge {
                let edg = topods::edge(em.find_key(i as usize)?)?;
                let (vf, vl) = topexp::vertices(&edg, false);
                let (pf, pl) = BRepTool::range(&edg)?;
                let (reg1, regn) = regularity(&edg, &edges_to_faces)?;
                let a = edge_end(&vf, pf, real_first(), &vertices_to_edges, &top_ds)?;
                let b = edge_end(&vl, pl, real_last(), &vertices_to_edges, &top_ds)?;
                let mut ed = data.e_data_array();
                let ed = ed
                    .change_value(i)
                    .map_err(|e| e.into_failure("HLRBRep_ShapeToHLR::Load"))?;
                ed.set(
                    reg1, regn, &edg, a.index, b.index, a.out, b.out, a.cut, b.cut, a.param, a.tol,
                    b.param, b.tol,
                )?;
                data.edge_map().add(Shape::from(edg));
            }
        }
        Self::explore_shape(s, &ds, &fm, &em)?;
        Ok(ds)
    }

    /// ExploreFace(S, DS, FM, EM, i, F, closed): the next face record: the face `i`
    /// of `fm` (FORWARD, with its orientation kept), its wires and their edges that
    /// are not degenerated, with the outline, internal, double and isoline flags of
    /// the original face `f`; `ShapeToHLR::explore_face(&s, &ds, &fm, &em, &mut i, &f, true)?`.
    fn explore_face(
        s: &Handle<OutLiner>,
        ds: &Handle<Data>,
        fm: &IndexedMapOfShape,
        em: &IndexedMapOfShape,
        i: &mut i32,
        f: &Face,
        closed: bool,
    ) -> Result<(), Failure> {
        *i += 1;
        let outliner = s.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "HLRBRep_ShapeToHLR::ExploreFace: no outliner",
            )
        })?;
        let data = ds.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "HLRBRep_ShapeToHLR::ExploreFace: no data structure",
            )
        })?;
        let top_ds = outliner.data_structure();
        let fi = fm.find_key(*i as usize)?;
        let orient = fi.orientation();
        let mut the_face = topods::face(fi)?;
        the_face.set_orientation(Orientation::Forward);
        let face_shape = Shape::from(the_face.clone());
        let wires: Vec<Shape> =
            Explorer::new(&face_shape, ShapeEnum::Wire, ShapeEnum::Shape)?.collect();
        let mut fdata = data.f_data_array();
        let fd = fdata
            .change_value(*i)
            .map_err(|e| e.into_failure("HLRBRep_ShapeToHLR::ExploreFace"))?;
        fd.set(&the_face, orient, closed, wires.len() as i32)?;
        for (nw, wire) in wires.iter().enumerate() {
            let nw = nw as i32 + 1;
            let mut edges = Vec::new();
            for e in Explorer::new(wire, ShapeEnum::Edge, ShapeEnum::Shape)? {
                let e = topods::edge(&e)?;
                if !BRepTool::degenerated(&e)? {
                    edges.push(e);
                }
            }
            fd.set_wire(nw, edges.len() as i32)?;
            for (ne, e) in edges.iter().enumerate() {
                let ie = em.find_index(&Shape::from(e.clone())) as i32;
                let int = top_ds.is_int_l_face_edge(f, e)?;
                let iso = top_ds.is_iso_l_face_edge(f, e)?;
                let out = top_ds.is_out_l_face_edge(f, e)?;
                let dbl = brep_tools::is_really_closed(e, &the_face)?;
                fd.set_w_edge(nw, ne as i32 + 1, ie, e.orientation(), out, int, dbl, iso)?;
            }
        }
        drop(fdata);
        data.face_map().add(face_shape);
        Ok(())
    }

    /// ExploreShape(S, DS, FM, EM): the face records of the faces of the original
    /// shells (closed when every edge is used once in each direction), then of the
    /// faces outside shells; `ShapeToHLR::explore_shape(&s, &ds, &fm, &em)?`.
    fn explore_shape(
        s: &Handle<OutLiner>,
        ds: &Handle<Data>,
        fm: &IndexedMapOfShape,
        em: &IndexedMapOfShape,
    ) -> Result<(), Failure> {
        let outliner = s.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "HLRBRep_ShapeToHLR::ExploreShape: no outliner",
            )
        })?;
        let original = outliner.original_shape();
        let mut shape_map = MapOfShape::default();
        let mut i = 0;
        let mut exshell = Explorer::new(&original, ShapeEnum::Shell, ShapeEnum::Shape)?;
        while exshell.more() {
            let shell = exshell.current().clone();
            let closed = shell.closed()? || Self::edges_balanced(&shell, em)?;
            for face in Explorer::new(&shell, ShapeEnum::Face, ShapeEnum::Shape)? {
                if shape_map.add(face.clone()) {
                    Self::explore_face(s, ds, fm, em, &mut i, &topods::face(&face)?, closed)?;
                }
            }
            exshell.next()?;
        }
        for face in Explorer::new(&original, ShapeEnum::Face, ShapeEnum::Shell)? {
            if shape_map.add(face.clone()) {
                Self::explore_face(s, ds, fm, em, &mut i, &topods::face(&face)?, false)?;
            }
        }
        Ok(())
    }

    // An open shell is closed when each edge of `em` is used as many times FORWARD
    // as REVERSED (the edges outside `em` count at index 0, which is not tested).
    fn edges_balanced(shell: &Shape, em: &IndexedMapOfShape) -> Result<bool, Failure> {
        let mut flag = vec![0i32; em.extent() + 1];
        for e in Explorer::new(shell, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edge = topods::edge(&e)?;
            let ie = em.find_index(&e);
            if BRepTool::degenerated(&edge)? {
                continue;
            }
            match edge.orientation() {
                Orientation::Forward => flag[ie] += 1,
                Orientation::Reversed => flag[ie] -= 1,
                _ => {}
            }
        }
        Ok(flag.iter().skip(1).all(|f| *f == 0))
    }
}
