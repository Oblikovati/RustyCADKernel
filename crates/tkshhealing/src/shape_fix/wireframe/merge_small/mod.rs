// Rust translation of ShapeFix_Wireframe::MergeSmallEdges of ShapeFix_Wireframe.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MergeSmallEdges`: each small edge of the wires of the faces with small edges, then of
//! the free wires, is joined with a neighbour (or dropped in drop mode); the vertices of
//! the joined edges are replaced by copies, the context is applied and SameParameter run.
mod choose;
mod drop;
mod join;

use super::{ShapeFixWireframe, ShapeFixWireframeShapeLists};
use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus, ShapeExtendWireData};
use crate::shape_fix::{FixRoot, ShapeFix, ShapeFixWire};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape, ShapeIterator, Wire};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::message::{Msg, ProgressRange};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The maps built by CheckSmallEdges, updated as the edges are merged.
pub(super) struct SmallEdgeMaps<'a> {
    pub small: &'a mut MapOfShape,
    pub edge_faces: &'a mut ShapeFixWireframeShapeLists,
    pub face_small: &'a mut ShapeFixWireframeShapeLists,
    pub multiple: &'a MapOfShape,
}

/// One run of MergeSmallEdges: the tool, its context, the maps, the shared wire fixer
/// (`SFW`), the angle limit (when `aModLimitAngle`), the drop mode and `theNewVertices`.
pub(super) struct MergeRun<'a, 'm> {
    fixer: &'a ShapeFixWireframe,
    context: Handle<ShapeBuildReShape>,
    maps: SmallEdgeMaps<'m>,
    wire_fixer: ShapeFixWire,
    limit_angle: Option<f64>,
    drop_mode: bool,
    new_vertices: DataMap<Shape, Shape, ShapeMapHasher>,
}

/// The wire under merge: `anExpf2.Current()` (the shape compared with the faces of a
/// small edge; the whole shape for free wires, as the exhausted explorer returns it)
/// and the face given to the analysis of the resulting edge (none for free wires).
pub(super) struct WireScope {
    pub current: Shape,
    pub face: Option<Face>,
}

impl ShapeFixWireframe {
    /// `MergeSmallEdges(small, edgeToFaces, faceWithSmall, multyEdges, modeDrop, limitAngle)`:
    /// merges the small edges found by [`Self::check_small_edges`]; false when no edge is
    /// small. Example: `tool.merge_small_edges(&mut small, &mut ef, &mut fs, &mut multi, false, -1.)?`.
    pub fn merge_small_edges(
        &self,
        small: &mut MapOfShape,
        edge_faces: &mut ShapeFixWireframeShapeLists,
        face_small: &mut ShapeFixWireframeShapeLists,
        multiple: &mut MapOfShape,
        mode_drop: bool,
        limit_angle: f64,
    ) -> Result<bool, Failure> {
        if small.is_empty() {
            return Ok(false);
        }
        let configured = self.limit_angle();
        let modified = limit_angle > -1.0 || configured > -1.0;
        let maps = SmallEdgeMaps {
            small,
            edge_faces,
            face_small,
            multiple,
        };
        let mut run = MergeRun::new(
            self,
            maps,
            modified.then(|| limit_angle.max(configured)),
            mode_drop || self.mode_drop_small_edges(),
        );
        run.merge_faces()?;
        run.merge_free_wires()?;
        run.finish()
    }
}

impl<'a, 'm> MergeRun<'a, 'm> {
    fn new(
        fixer: &'a ShapeFixWireframe,
        maps: SmallEdgeMaps<'m>,
        limit_angle: Option<f64>,
        drop_mode: bool,
    ) -> Self {
        let context = fixer.context();
        let wire_fixer = ShapeFixWire::new();
        wire_fixer.set_context(&context);
        Self {
            fixer,
            context,
            maps,
            wire_fixer,
            limit_angle,
            drop_mode,
            new_vertices: DataMap::default(),
        }
    }

    fn precision(&self) -> f64 {
        self.fixer.precision()
    }

    fn add_status(&self, status: ShapeExtendStatus) {
        self.fixer.state.write().small |= ShapeExtend::encode_status(status);
    }

    fn warn(&self, shape: &Shape, key: &str) {
        self.fixer
            .send_warning(shape, &Msg::from_key(&AsciiString::from(key)));
    }

    fn wire_data(&self) -> Result<Handle<ShapeExtendWireData>, Failure> {
        self.wire_fixer.wire_data()
    }

    /// The faces of the shape having small edges.
    fn merge_faces(&mut self) -> Result<(), Failure> {
        for current in Explorer::new(&self.fixer.shape(), ShapeEnum::Face, ShapeEnum::Shape)? {
            let Ok(edges) = self.maps.face_small.find(&current) else {
                continue;
            };
            if edges.extent() == 0 {
                continue;
            }
            self.merge_face(&current)?;
        }
        Ok(())
    }

    fn merge_face(&mut self, current: &Shape) -> Result<(), Failure> {
        let facet = topods::face(&ReShape::apply(&*self.context, current, ShapeEnum::Shape)?)?;
        if !facet.is_same(current) {
            self.remap_face_edges(current)?;
        }
        let mut face = facet.clone();
        for child in ShapeIterator::new(&facet, true, true) {
            if child.shape_type()? != ShapeEnum::Wire {
                continue;
            }
            if face.orientation() == rustycad_tkg3d::top_abs::Orientation::Reversed {
                face =
                    topods::face(&facet.oriented(rustycad_tkg3d::top_abs::Orientation::Forward))?;
            }
            let wire = topods::wire(&child)?;
            let data = Handle::new(ShapeExtendWireData::from_wire(&wire, true, false)?);
            self.wire_fixer.load_wire_data(&data)?;
            self.wire_fixer.fix_reorder(false)?;
            let scope = WireScope {
                current: current.clone(),
                face: Some(face.clone()),
            };
            self.merge_wire(&wire, &scope)?;
        }
        self.remove_empty_face(face, &facet)
    }

    /// The edges of a face the context modified are renamed in the small-edge and
    /// edge-to-faces maps (`// gka`).
    fn remap_face_edges(&mut self, current: &Shape) -> Result<(), Failure> {
        for edge in Explorer::new(current, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let (status, new_edge) = ReShape::status(&*self.context, &edge, true)?;
            if status <= 0 {
                continue;
            }
            if self.maps.small.contains(&edge) {
                self.maps.small.remove(&edge);
                self.maps.small.add(new_edge.clone());
            }
            if self.maps.edge_faces.is_bound(&edge) {
                let faces = self.maps.edge_faces.find(&edge)?.clone();
                self.maps.edge_faces.unbind(&edge);
                self.maps.edge_faces.bind(new_edge, faces);
            }
        }
        Ok(())
    }

    fn remove_empty_face(&self, mut face: Face, facet: &Face) -> Result<(), Failure> {
        face.set_orientation(facet.orientation());
        let new_shape = ReShape::apply(&*self.context, &face, ShapeEnum::Shape)?;
        if ShapeIterator::new(&new_shape, true, true).more() {
            return Ok(());
        }
        ReShape::remove(&*self.context, &new_shape)?;
        self.warn(&face, "FixWireframe.FixSmallEdges.MSG2");
        Ok(())
    }

    /// The wires not lying on faces (`enk block`).
    fn merge_free_wires(&mut self) -> Result<(), Failure> {
        let shape = self.fixer.shape();
        for child in Explorer::new(&shape, ShapeEnum::Wire, ShapeEnum::Face)? {
            let wire = topods::wire(&child)?;
            self.wire_fixer.load(&wire)?;
            self.wire_fixer.fix_reorder(false)?;
            let scope = WireScope {
                current: shape.clone(),
                face: None,
            };
            self.merge_wire(&wire, &scope)?;
        }
        Ok(())
    }

    /// The loop over the edges of the loaded wire, then the drop of a last small edge or
    /// the reconnection of the wire and its replacement in the context.
    fn merge_wire(&mut self, wire: &Wire, scope: &WireScope) -> Result<(), Failure> {
        let mut index = 1;
        while index <= self.wire_data()?.nb_edges() && self.wire_data()?.nb_edges() > 1 {
            index = self.merge_at(index, scope)?;
        }
        if self.wire_data()?.nb_edges() == 1 && self.drop_mode {
            return self.drop_last_edge(wire);
        }
        self.wire_fixer.fix_connected_all(-1.0)?;
        ReShape::replace(&*self.context, wire, &self.wire_fixer.wire()?.into_shape())
    }

    fn drop_last_edge(&mut self, wire: &Wire) -> Result<(), Failure> {
        let edge = self.wire_data()?.edge(1)?;
        if !self.maps.small.contains(&edge) {
            return Ok(());
        }
        self.wire_data()?.remove(1)?;
        ReShape::remove(&*self.context, &edge)?;
        self.warn(&edge, "FixWireframe.FixSmallEdges.MSG0");
        self.maps.small.remove(&edge);
        self.maps.edge_faces.unbind(&edge);
        ReShape::remove(&*self.context, wire)?;
        self.warn(wire, "FixWireframe.FixSmallEdges.MSG1");
        self.add_status(ShapeExtendStatus::Done2);
        Ok(())
    }

    /// The vertices of the joined edges are replaced by their copies, the context is
    /// applied to the shape and SameParameter run on the result.
    fn finish(self) -> Result<bool, Failure> {
        for (vertex, copy) in self.new_vertices.items() {
            ReShape::replace(&*self.context, vertex, copy)?;
        }
        let shape = self.fixer.shape();
        self.fixer.state.write().shape = Shape::default();
        let applied = ReShape::apply(&*self.context, &shape, ShapeEnum::Shape)?;
        self.fixer.state.write().shape = applied.clone();
        ShapeFix::same_parameter(
            &applied,
            false,
            0.0,
            &ProgressRange::default(),
            &Handle::default(),
        )?;
        Ok(self.fixer.status_small_edges(ShapeExtendStatus::Done))
    }
}
