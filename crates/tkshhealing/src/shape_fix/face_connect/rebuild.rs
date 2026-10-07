//! `rebuildShell` of ShapeFix_FaceConnect: each original free edge replaced by the wire
//! of its resulting edges, the wires of the faces reordered and fixed, and the vertices of
//! the replaced edges merged.
use super::vertices::{VertexReplacements, merge_vertices};
use super::{ShapeFixFaceConnect, ShapeListMap};
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWireOrder};
use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::{ShapeExtendStatus, ShapeExtendWireData};
use crate::shape_fix::{FixRoot, ShapeFixFace, ShapeFixFaceWireMap, ShapeFixWire};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Shell, Vertex, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::FlatDataMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The wire of the resulting free then shared edges different from the edge; None when
/// there is none.
fn replacement_wire(
    edge: &Shape,
    free: &ShapeListMap,
    shared: &ShapeListMap,
) -> Result<Option<Wire>, Failure> {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    let mut empty = true;
    for list in [free.find(edge)?, shared.find(edge)?] {
        for result in list.iter().filter(|r| !r.is_same(edge)) {
            builder.add(&mut wire, result)?;
            empty = false;
        }
    }
    Ok((!empty).then_some(wire))
}

/// The vertex of the wire nearest to `point` (the first of equal distances).
fn nearest_vertex(wire: &Wire, point: rustycad_tkmath::gp::Point3) -> Result<Vertex, Failure> {
    let mut best: Option<(f64, Vertex)> = None;
    for shape in Explorer::new(wire, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        let vertex = topods::vertex(&shape)?;
        let distance = point.distance(&BRepTool::pnt(&vertex)?);
        if best.as_ref().is_none_or(|(d, _)| distance < *d) {
            best = Some((distance, vertex));
        }
    }
    Ok(best.map(|(_, v)| v).unwrap_or_default())
}

/// The new vertex replacing `old` when it is a different one.
fn record_vertex(replacements: &mut VertexReplacements, old: &Vertex, new: &Vertex) {
    if old.is_same(new) {
        return;
    }
    let list = replacements.try_bound(old.clone().into_shape(), Default::default());
    if !list.iter().any(|stored| stored.is_same(new.as_ref())) {
        list.append(new.clone().into_shape());
    }
}

impl ShapeFixFaceConnect {
    /// `rebuildShell(theShell, theSewingTolerance, theFixingTolerance)`.
    pub(super) fn rebuild_shell(
        &self,
        shell: &Shell,
        sewing_tolerance: f64,
        fixing_tolerance: f64,
    ) -> Result<Shell, Failure> {
        if self.ori_free_edges.is_empty() {
            return Ok(shell.clone());
        }
        let mut rep_edges = FlatDataMap::<Shape, Shape, ShapeMapHasher>::default();
        let mut rep_vertices = VertexReplacements::default();
        for edges in self.ori_free_edges.values() {
            for edge in edges.iter() {
                self.replace_edge(edge, &mut rep_edges, &mut rep_vertices)?;
            }
        }
        if rep_edges.is_empty() {
            return Ok(shell.clone());
        }
        let reshape = Handle::new(ShapeBuildReShape::new());
        let context = reshape.get().ok_or_else(null_reshape)?;
        for (old, wire) in rep_edges.items() {
            context.replace(old, wire)?;
        }
        let result = topods::shell(&context.apply(shell.as_ref(), ShapeEnum::Shape)?)?;
        if context.status_flag(ShapeExtendStatus::Ok)
            || context.status_flag(ShapeExtendStatus::Fail1)
        {
            return Ok(result);
        }
        let fixed = fix_faces(&result, (sewing_tolerance, fixing_tolerance))?;
        if rep_vertices.is_empty() {
            return Ok(fixed);
        }
        merge_vertices(&fixed, &rep_vertices, context)
    }

    /// The wire replacing one original free edge and the vertices replacing its ends.
    fn replace_edge(
        &self,
        edge: &Shape,
        rep_edges: &mut FlatDataMap<Shape, Shape, ShapeMapHasher>,
        rep_vertices: &mut VertexReplacements,
    ) -> Result<(), Failure> {
        let old = topods::edge(edge)?;
        let Some(wire) = replacement_wire(edge, &self.res_free_edges, &self.res_shar_edges)? else {
            return Ok(());
        };
        let (v1, v2) = topexp::vertices(&old, false);
        let (p1, p2) = (BRepTool::pnt(&v1)?, BRepTool::pnt(&v2)?);
        let new1 = nearest_vertex(&wire, p1)?;
        let new2 = nearest_vertex(&wire, p2)?;
        record_vertex(rep_vertices, &v1, &new1);
        record_vertex(rep_vertices, &v2, &new2);
        rep_edges.try_bind(edge.clone(), wire.into_shape());
        Ok(())
    }
}

/// "Perform necessary fixes on subshapes": the wires of each face reordered (by their
/// 2D ends, then in 3D) and their edge curves and self-intersections fixed in an empty
/// copy of the face, whose orientation is then fixed.
fn fix_faces(
    result: &Shell,
    (sewing_tolerance, fixing_tolerance): (f64, f64),
) -> Result<Shell, Failure> {
    let builder = BRepBuilder::default();
    let wire_fixer = ShapeFixWire::new();
    let face_fixer = ShapeFixFace::default();
    let mut shell = topods::shell(&result.as_ref().empty_copied()?)?;
    for shape in ShapeIterator::new(result, true, true) {
        let face = topods::face(&shape)?;
        let mut empty_face = topods::face(&face.as_ref().empty_copied()?)?;
        for child in ShapeIterator::new(&face, true, true) {
            if child.shape_type()? != ShapeEnum::Wire {
                continue;
            }
            let wire = fix_wire(
                &topods::wire(&child)?,
                &empty_face,
                &wire_fixer,
                (sewing_tolerance, fixing_tolerance),
            )?;
            builder.add(&mut empty_face, &wire)?;
        }
        face_fixer.init(empty_face.clone().into_shape());
        let mut wires = ShapeFixFaceWireMap::default();
        if face_fixer.fix_orientation_with_map(&mut wires)? {
            empty_face = face_fixer.face()?;
        }
        builder.add(&mut shell, &empty_face)?;
    }
    shell.set_closed(BRepTool::is_closed(shell.as_ref())?)?;
    Ok(shell)
}

/// One wire reordered and fixed on the face.
fn fix_wire(
    wire: &Wire,
    face: &Face,
    fixer: &ShapeFixWire,
    (sewing_tolerance, fixing_tolerance): (f64, f64),
) -> Result<Wire, Failure> {
    let sewd = Handle::new(ShapeExtendWireData::from_wire(wire, true, true)?);
    let data = sewd.get().ok_or_else(null_reshape)?;
    let analysis = ShapeAnalysisEdge::new();
    let mut order = ShapeAnalysisWireOrder::with_mode(false, 0., false);
    let forward = topods::face(&face.oriented(Orientation::Forward))?;
    for i in 1..=data.nb_edges() {
        let edge: Edge = data.edge(i)?;
        let (pcurve, first, last) = analysis.pcurve_on_face(&edge, &forward, true)?;
        let Some(curve) = pcurve.get() else {
            continue;
        };
        order.add_2d(curve.value(first)?.xy(), curve.value(last)?.xy());
    }
    order.perform(true)?;
    fixer.load_wire_data(&sewd)?;
    fixer.fix_reorder_with(&order)?;
    fixer.fix_reorder(false)?;
    fixer.set_face(face)?;
    fixer.set_precision(fixing_tolerance);
    fixer.set_max_tolerance(sewing_tolerance);
    fixer.fix_edge_curves()?;
    fixer.fix_self_intersection()?;
    fixer.wire()
}

fn null_reshape() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeFix_FaceConnect: null handle; expected a reshape or wire data",
        "",
    )
}
