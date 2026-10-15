// Rust translation of BRepOffsetAPI_ThruSections::CreateRuled
// (ModelingAlgorithms/TKOffset/BRepOffsetAPI/BRepOffsetAPI_ThruSections.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The ruled loft: the shell of `BRepFill_Generator` through the sections, its solid,
//! and the face grown from each section edge.
use super::BRepOffsetAPIThruSections;
use super::solid::{make_solid, oriented_solid};
use rustycad_tkbool::brep_fill::{BRepFillGenerator, ThruSectionErrorStatus};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Shell};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The edge-to-face and vertex-to-face ancestors of the ruled shell.
struct RuledAncestors {
    edge_faces: IndexedDataMapOfShapeListOfShape,
    vertex_faces: IndexedDataMapOfShapeListOfShape,
}

impl BRepOffsetAPIThruSections {
    /// `CreateRuled()`: the generator shell (the status of the generator when it
    /// fails), the solid on request, done, then the history of the section edges.
    pub(super) fn create_ruled(&mut self) -> Result<(), Failure> {
        let mut generator = BRepFillGenerator::new();
        generator.set_mutable_input(self.my_mutable_input);
        for wire in &self.my_wires {
            generator.add_wire(&topods::wire(wire)?);
        }
        let generator = self.my_bf_generator.insert(generator);
        generator.perform()?;
        let status = generator.get_status();
        if status != ThruSectionErrorStatus::Done {
            self.my_status = status;
            return Ok(());
        }
        let shell: Shell = generator.shell().clone();
        self.base.my_shape = self.ruled_result(&shell)?;
        self.base.set_done();
        self.ruled_history(&shell)
    }

    /// The shell, or its solid: closed by its own sections when the first and last are
    /// the same, by caps on the result sections otherwise.
    fn ruled_result(&mut self, shell: &Shell) -> Result<Shape, Failure> {
        if !self.my_is_solid {
            return Ok(shell.as_ref().clone());
        }
        let count = self.my_wires.len();
        if self.my_wires[0].is_same(&self.my_wires[count - 1]) {
            return Ok(oriented_solid(shell)?.into_shape());
        }
        let generator = self.generator()?;
        let wire1 = topods::wire(&generator.result_shape(&self.my_wires[0])?)?;
        let wire2 = topods::wire(&generator.result_shape(&self.my_wires[count - 1])?)?;
        let mut cap_shell = shell.clone();
        let caps = (&mut self.my_first, &mut self.my_last);
        Ok(make_solid(&mut cap_shell, (&wire1, &wire2), self.my_pres3d, caps)?.into_shape())
    }

    /// The face of the shell grown from each edge of each section but the last.
    fn ruled_history(&mut self, shell: &Shell) -> Result<(), Failure> {
        let mut ancestors = RuledAncestors {
            edge_faces: IndexedDataMapOfShapeListOfShape::default(),
            vertex_faces: IndexedDataMapOfShapeListOfShape::default(),
        };
        topexp::map_shapes_and_ancestors(
            shell,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut ancestors.edge_faces,
        )?;
        topexp::map_shapes_and_ancestors(
            shell,
            ShapeEnum::Vertex,
            ShapeEnum::Face,
            &mut ancestors.vertex_faces,
        )?;
        for i in 0..self.my_wires.len().saturating_sub(1) {
            let wire1 = topods::wire(&self.my_wires[i])?;
            let wire2 = topods::wire(&self.my_wires[i + 1])?;
            self.history_between(&wire1, &wire2, &ancestors)?;
        }
        Ok(())
    }

    /// Walks two consecutive sections together (a degenerated edge stays while the
    /// other section advances) and binds each edge of the first to the face shared
    /// with the facing edge of the second.
    fn history_between(
        &mut self,
        wire1: &topods::Wire,
        wire2: &topods::Wire,
        ancestors: &RuledAncestors,
    ) -> Result<(), Failure> {
        let mut exp1 = WireExplorer::with_wire(wire1)?;
        let mut exp2 = WireExplorer::with_wire(wire2)?;
        let mut tantque = exp1.more() && exp2.more();
        while tantque {
            let edge1 = exp1.current().clone();
            let edge2 = exp2.current().clone();
            let degen1 = BRepTool::degenerated(&edge1)?;
            let degen2 = BRepTool::degenerated(&edge2)?;
            let map_faces = self.faces_of_section_edge(&edge2, degen2, ancestors)?;
            if let Some(face) = self.first_shared_face(&edge1, degen1, ancestors, &map_faces)? {
                self.my_edge_face.bind(edge1.as_ref().clone(), face);
            }
            if !degen1 {
                exp1.next()?;
            }
            if !degen2 {
                exp2.next()?;
            }
            tantque = exp1.more() && exp2.more();
            if degen1 {
                tantque = exp2.more();
            }
            if degen2 {
                tantque = exp1.more();
            }
        }
        Ok(())
    }

    /// The faces of the result of a section edge (of its vertex for a degenerated one).
    fn faces_of_section_edge(
        &self,
        edge: &Edge,
        degenerated: bool,
        ancestors: &RuledAncestors,
    ) -> Result<MapOfShape, Failure> {
        let mut map_faces = MapOfShape::default();
        for face in self.ancestor_faces(edge, degenerated, ancestors)?.iter() {
            map_faces.add(face.clone());
        }
        Ok(map_faces)
    }

    /// The first face of the result of a section edge that is also in `map_faces`.
    fn first_shared_face(
        &self,
        edge: &Edge,
        degenerated: bool,
        ancestors: &RuledAncestors,
        map_faces: &MapOfShape,
    ) -> Result<Option<Shape>, Failure> {
        let faces = self.ancestor_faces(edge, degenerated, ancestors)?;
        Ok(faces.iter().find(|face| map_faces.contains(face)).cloned())
    }

    /// The ancestor faces of the result of a section edge, or of the result of the
    /// first vertex of a degenerated one; Standard_NoSuchObject when it has none.
    fn ancestor_faces<'a>(
        &self,
        edge: &Edge,
        degenerated: bool,
        ancestors: &'a RuledAncestors,
    ) -> Result<&'a rustycad_tkernel::ncollection::List<Shape>, Failure> {
        let generator = self.generator()?;
        if degenerated {
            let first = topexp::first_vertex(edge, false);
            let vdegen = topods::vertex(&generator.result_shape(first.as_ref())?)?;
            return ancestors.vertex_faces.find_from_key(vdegen.as_ref());
        }
        let result = generator.result_shape(edge.as_ref())?;
        ancestors.edge_faces.find_from_key(&result)
    }

    /// The generator of the last ruled build.
    pub(super) fn generator(&self) -> Result<&BRepFillGenerator, Failure> {
        self.my_bf_generator.as_ref().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "BRepOffsetAPI_ThruSections: no BRepFill_Generator; expected a ruled build",
            )
        })
    }
}
