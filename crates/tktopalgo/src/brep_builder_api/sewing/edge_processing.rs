// Rust translation of the static GetSeqEdges, IsDegeneratedWire and DegeneratedSection
// and of BRepBuilderAPI_Sewing::GetFreeWires, EdgeProcessing and EdgeRegularity
// (BRepBuilderAPI_Sewing.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The free edges left after the merge: chained into wires, the wires shorter than
//! their tolerances made of degenerated edges; and the regularity of the merged edges.
use super::evaluate::global_curve;
use super::replace_edge::replace_edge;
use super::{BRepBuilderAPISewing, SewingShapeListMap, SewingShapeMap, live_reshape};
use crate::brep_lib;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::{List, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::gcpnts::AbscissaPoint;

/// The native default `TolAng` of `BRepLib::EncodeRegularity(E, F1, F2)`.
const ENCODE_TOL_ANG: f64 = 1.0e-10;

/// The children of a shape without orientation composition, as
/// `TopoDS_Iterator(S, false)` gives them.
fn stored_children(edge: &Shape) -> Vec<Shape> {
    let mut vertices = Vec::new();
    let mut it = ShapeIterator::new(edge, false, true);
    while let Ok(v) = it.value() {
        vertices.push(v.clone());
        it.next();
    }
    vertices
}

/// The static `GetSeqEdges(edge, seqEdges, VertEdge)`: the free edges connected to
/// `edge` through its vertices, inserted before it (first vertex) or after it (other
/// vertices), recursively.
fn get_seq_edges(
    edge: &Shape,
    seq_edges: &mut Sequence<Shape>,
    vert_edge: &SewingShapeListMap,
) -> Result<(), Failure> {
    for (num_v, v1) in stored_children(edge).iter().enumerate() {
        let Ok(list_edges) = vert_edge.find(v1) else {
            continue;
        };
        for edge1 in list_edges.iter() {
            if edge1.is_same(edge) {
                continue;
            }
            let (mut is_contained, mut index) = (false, 1);
            let mut i = 1;
            while i <= seq_edges.len() && !is_contained {
                is_contained = seq_edges.value(i)?.is_same(edge1);
                if !is_contained && seq_edges.value(i)?.is_same(edge) {
                    index = i;
                }
                i += 1;
            }
            if is_contained {
                continue;
            }
            if num_v == 0 {
                seq_edges.insert_before(index, edge1.clone())?;
            } else {
                seq_edges.insert_after(index, edge1.clone())?;
            }
            get_seq_edges(edge1, seq_edges, vert_edge)?;
        }
    }
    Ok(())
}

/// The length of one edge of a wire (the chord when the curve is not bent back, else
/// its arc length) and whether it is within its vertices' tolerances; None without 3d
/// curve.
fn wire_edge_length(
    edge: &Edge,
    (ve1, ve2): (&Vertex, &Vertex),
) -> Result<Option<(f64, bool)>, Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    let Some(curve) = found.curve.get() else {
        return Ok(None);
    };
    let c3d: Handle<dyn GeomCurve> = curve.copy()?.cast();
    let c = c3d
        .get()
        .ok_or_else(|| super::failures::null_object("IsDegeneratedWire: null copy"))?;
    if !found.location.is_identity() {
        c.transform(found.location.transformation())?;
    }
    let (first, last) = (found.first, found.last);
    let (pfirst, plast, pmid) = (
        c.value(first)?,
        c.value(last)?,
        c.value((first + last) * 0.5)?,
    );
    let length = if pfirst.distance(&plast) > pfirst.distance(&pmid) {
        pfirst.distance(&plast)
    } else {
        AbscissaPoint::length_between(&GeomAdaptorCurve::with_curve(&c3d)?, first, last)?
    };
    let tole = BRepTool::vertex_tolerance(ve1)? + BRepTool::vertex_tolerance(ve2)?;
    Ok(Some((length, length <= tole)))
}

/// The static `IsDegeneratedWire(wire)`: every edge is within its vertices'
/// tolerances, or the whole wire is within the tolerances of its two ends.
fn is_degenerated_wire(wire: &Shape) -> Result<bool, Failure> {
    if wire.shape_type()? != ShapeEnum::Wire {
        return Ok(false);
    }
    let (mut v1, mut v2) = (Vertex::default(), Vertex::default());
    let (mut wire_length, mut nume, mut is_small) = (0.0, 0, 0);
    let mut it = ShapeIterator::new(wire, false, true);
    while let Ok(child) = it.value() {
        nume += 1;
        let edge = topods::edge(child)?;
        let (ve1, ve2) = topexp::vertices(&edge, false);
        if nume == 1 {
            (v1, v2) = (ve1.clone(), ve2.clone());
        } else {
            if ve1.is_same(&v1) {
                v1 = ve2.clone();
            } else if ve1.is_same(&v2) {
                v2 = ve2.clone();
            }
            if ve2.is_same(&v1) {
                v1 = ve1.clone();
            } else if ve2.is_same(&v2) {
                v2 = ve1.clone();
            }
        }
        if let Some((length, small)) = wire_edge_length(&edge, (&ve1, &ve2))? {
            is_small += i32::from(small);
            wire_length += length;
        }
        it.next();
    }
    if is_small == nume {
        return Ok(true);
    }
    let tol = BRepTool::vertex_tolerance(&v1)? + BRepTool::vertex_tolerance(&v2)?;
    Ok(wire_length <= tol)
}

/// The vertex of a degenerated section: the one vertex of a closed section, a vertex
/// whose tolerance covers the other, or a new vertex at the middle point.
fn degenerated_vertex(
    v1: &Vertex,
    v2: &Vertex,
    [p1, p2, p3]: [rustycad_tkmath::gp::Point3; 3],
) -> Result<Vertex, Failure> {
    if p1.distance(&p3) < BRepTool::vertex_tolerance(v1)? {
        return Ok(v1.clone());
    }
    if p1.distance(&p3) < BRepTool::vertex_tolerance(v2)? {
        return Ok(v2.clone());
    }
    let d1 = BRepTool::vertex_tolerance(v1)? + p2.distance(&p1);
    let d2 = BRepTool::vertex_tolerance(v2)? + p2.distance(&p3);
    let mut new_vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut new_vertex, p2, std_max(d1, d2))?;
    Ok(new_vertex)
}

/// The static `DegeneratedSection(section, face)`: a degenerated copy of the section
/// (one vertex at both ends, no 3d curve, the p-curve on the face kept); a section
/// without 3d curve is flagged degenerated in place.
fn degenerated_section(section: &Edge, face: &Shape) -> Result<Edge, Failure> {
    if BRepTool::degenerated(section)? {
        return Ok(section.clone());
    }
    let b = BRepBuilder::default();
    let (c3d, first, last) = global_curve(section)?;
    let Some(c) = c3d.get() else {
        b.degenerated(section, true)?;
        return Ok(section.clone());
    };
    let (v1, v2) = topexp::vertices(section, false);
    let (p1, p3) = (BRepTool::pnt(&v1)?, BRepTool::pnt(&v2)?);
    let p2 = c.d0(0.5 * (first + last))?;
    let mut edge = section.clone();
    edge.empty_copy()?;
    let (vf, vr) = if v1.is_same(&v2) {
        (v1.clone(), v2.clone())
    } else {
        let v = degenerated_vertex(&v1, &v2, [p1, p2, p3])?;
        (v.clone(), v)
    };
    let mut an_edge = edge.oriented(Orientation::Forward);
    b.add(&mut an_edge, &vf.oriented(Orientation::Forward))?;
    b.add(&mut an_edge, &vr.oriented(Orientation::Reversed))?;
    let (first, last) = BRepTool::range(section)?;
    b.range(
        &Edge::unchecked(edge.oriented(Orientation::Forward)),
        first,
        last,
        false,
    )?;
    b.degenerated(&edge, true)?;
    if !face.is_null() {
        let f = topods::face(face)?;
        let c2dt = BRepTool::curve_on_face(section, &f)?.pcurve;
        b.update_edge_curve(&edge, &Handle::default(), 0.0)?;
        if BRepTool::curve_on_face(&edge, &f)?.pcurve.is_null() {
            b.update_edge_pcurve_on_face(&edge, &c2dt, &f, 0.0)?;
        }
    }
    Ok(edge)
}

impl BRepBuilderAPISewing {
    /// `GetFreeWires(MapFreeEdges, seqWires)`: the free edges chained through their
    /// vertices into wires (each edge once), removed from the map.
    pub(crate) fn get_free_wires(
        &self,
        map_free_edges: &mut IndexedMapOfShape,
        seq_wires: &mut Sequence<Shape>,
    ) -> Result<(), Failure> {
        let mut vert_edge = SewingShapeListMap::default();
        let mut seq_free_edges: Vec<Shape> = Vec::new();
        for i in 1..=map_free_edges.extent() {
            let edge = map_free_edges.find_key(i)?.clone();
            seq_free_edges.push(edge.clone());
            for v1 in stored_children(&edge) {
                match vert_edge.change_find(&v1) {
                    Ok(list) => {
                        list.append(edge.clone());
                    }
                    Err(_) => {
                        let mut list = List::default();
                        list.append(edge.clone());
                        vert_edge.bind(v1, list);
                    }
                }
            }
        }
        let b = BRepBuilder::default();
        for edge in &seq_free_edges {
            if !map_free_edges.contains(edge) {
                continue;
            }
            let mut seq_edges: Sequence<Shape> = Sequence::default();
            seq_edges.append(edge.clone());
            get_seq_edges(edge, &mut seq_edges, &vert_edge)?;
            let mut wire = Wire::default();
            b.make_wire(&mut wire);
            for e in seq_edges.iter() {
                b.add(&mut wire, e)?;
                map_free_edges.remove_key(e);
            }
            seq_wires.append(wire.into_shape());
            if map_free_edges.is_empty() {
                break;
            }
        }
        Ok(())
    }

    /// The images of the free boundaries (or of their unmerged sections) with their
    /// face.
    fn free_edges_with_faces(
        &self,
        progress: &ProgressRange<'_>,
    ) -> Result<(IndexedMapOfShape, SewingShapeMap), Failure> {
        let reshape = live_reshape(&self.my_re_shape)?;
        let (mut map_free_edges, mut edge_face) =
            (IndexedMapOfShape::default(), SewingShapeMap::default());
        let scope = ProgressScope::with_literal(
            progress,
            "Edge processing",
            self.my_bound_faces.extent() as f64,
            false,
        );
        let mut i = 1;
        while i <= self.my_bound_faces.extent() && scope.more() {
            let bound = self.my_bound_faces.find_key(i)?;
            let list_faces = self.my_bound_faces.find_from_index(i)?;
            i += 1;
            scope.next(1.0);
            if list_faces.extent() != 1 {
                continue;
            }
            let pieces: Vec<Shape> = match self.my_bound_sections.find(bound) {
                Ok(sections) => sections.iter().cloned().collect(),
                Err(_) => vec![bound.clone()],
            };
            for piece in pieces.iter().filter(|p| !self.my_merged_edges.contains(p)) {
                let edge = reshape.apply(piece, ShapeEnum::Shape)?;
                if !map_free_edges.contains(&edge) {
                    edge_face.bind(
                        edge.clone(),
                        list_faces.first().cloned().unwrap_or_default(),
                    );
                    map_free_edges.add(edge);
                }
            }
        }
        Ok((map_free_edges, edge_face))
    }

    /// `EdgeProcessing(theProgress)`: the free edges are chained into wires; the edges
    /// of a wire within its tolerances become degenerated sections.
    pub(crate) fn edge_processing(&mut self, progress: &ProgressRange<'_>) -> Result<(), Failure> {
        let (mut map_free_edges, edge_face) = self.free_edges_with_faces(progress)?;
        if map_free_edges.is_empty() {
            return Ok(());
        }
        let mut seq_wires = Sequence::default();
        self.get_free_wires(&mut map_free_edges, &mut seq_wires)?;
        let reshape_handle = self.my_re_shape.clone();
        let reshape = live_reshape(&reshape_handle)?;
        for wire in seq_wires.iter() {
            if !is_degenerated_wire(wire)? {
                continue;
            }
            for child in stored_children(wire) {
                let edge = topods::edge(&reshape.apply(&child, ShapeEnum::Shape)?)?;
                let face = edge_face.find(&edge).cloned().unwrap_or_default();
                let degedge = degenerated_section(&edge, &face)?;
                if degedge.is_null() {
                    continue;
                }
                if !degedge.is_same(&edge) {
                    replace_edge(&edge, &degedge, reshape)?;
                }
                if BRepTool::degenerated(&degedge)? {
                    self.my_degenerated.add(degedge.into_shape());
                }
            }
        }
        Ok(())
    }

    /// `EdgeRegularity(theProgress)`: each merged edge shared by two faces of the sewn
    /// shape gets its continuity between them; the merged edges are then forgotten.
    pub(crate) fn edge_regularity(&mut self, progress: &ProgressRange<'_>) -> Result<(), Failure> {
        let mut map_ef = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            &self.my_sewed_shape,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut map_ef,
        )?;
        let reshape = live_reshape(&self.my_re_shape)?;
        let scope = ProgressScope::with_literal(
            progress,
            "Encode edge regularity",
            self.my_merged_edges.extent() as f64,
            false,
        );
        // The merged edges are visited in hash order, natively as here; each edge is
        // encoded on its own, so the order does not change the result.
        for merged in self.my_merged_edges.iter() {
            if !scope.more() {
                break;
            }
            let edge = topods::edge(&reshape.apply(merged, ShapeEnum::Shape)?)?;
            if let Some(faces) = map_ef.seek(&edge)
                && faces.extent() == 2
                && let (Some(f1), Some(f2)) = (faces.first(), faces.last())
            {
                brep_lib::encode_regularity_of_edge(
                    &edge,
                    &topods::face(f1)?,
                    &topods::face(f2)?,
                    ENCODE_TOL_ANG,
                )?;
            }
            scope.next(1.0);
        }
        self.my_merged_edges.clear(true);
        Ok(())
    }
}
