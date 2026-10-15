// Rust translation of ModelingAlgorithms/TKOffset/BRepOffsetAPI/BRepOffsetAPI_DraftAngle.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871 (CorrectWires is in the sibling module).

//! `BRepOffsetAPI_DraftAngle`: the taper-adding transformation of faces of a shape, a
//! `Draft_Modification` performed through `BRepTools_Modifier`.
mod correct_wires;

use crate::draft::{DraftErrorStatus, DraftModification};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{BRepToolsModification, BRepToolsReShape, NewSurface, ReShape};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Direction3, Plane3};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase, BRepBuilderAPIModifyShape,
};

/// `Epsilon(x)`: the gap from `|x|` to the next representable value.
fn epsilon(x: f64) -> f64 {
    let a = x.abs();
    f64::from_bits(a.to_bits() + 1) - a
}

/// `BRepOffsetAPI_DraftAngle`: adds draft angles to faces of a shape about neutral
/// planes, keeping the topology.
///
/// ```
/// use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIDraftAngle;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let draft = BRepOffsetAPIDraftAngle::new();
/// assert!(!draft.is_done());
/// assert!(draft.add_done().is_err());
/// ```
#[derive(Default)]
pub struct BRepOffsetAPIDraftAngle {
    modify: BRepBuilderAPIModifyShape,
    draft: Handle<DraftModification>,
    my_vtx_to_replace: DataMap<Shape, Shape, ShapeMapHasher>,
    my_subs: BRepToolsReShape,
}

impl BRepOffsetAPIDraftAngle {
    /// `BRepOffsetAPI_DraftAngle()`: no shape and no modification;
    /// `BRepOffsetAPIDraftAngle::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepOffsetAPI_DraftAngle(S)`: the initial shape and its draft modification;
    /// `BRepOffsetAPIDraftAngle::with_shape(&shape)?`.
    pub fn with_shape(s: &Shape) -> Result<Self, Failure> {
        let mut draft = Self::new();
        draft.modify.my_initial_shape = s.clone();
        draft.set_modification(Handle::new(DraftModification::new(s)?));
        Ok(draft)
    }

    fn set_modification(&mut self, modification: Handle<DraftModification>) {
        self.modify.my_modification = modification.clone().cast::<dyn BRepToolsModification>();
        self.draft = modification;
    }

    /// The draft modification (Standard_NullObject before a shape is given, a null
    /// dereference natively).
    fn modification(&self) -> Result<&DraftModification, Failure> {
        self.draft.get().ok_or_else(|| {
            Failure::with_message("Standard_NullObject", "BRepOffsetAPI_DraftAngle: no modification")
        })
    }

    /// `Standard_NullObject_Raise_if(myInitialShape.IsNull(), ...)`.
    fn check_initial_shape(&self, member: &str) -> Result<(), Failure> {
        if self.modify.my_initial_shape.is_null() {
            return Err(Failure::with_message(
                "Standard_NullObject",
                &format!("BRepOffsetAPI_DraftAngle::{member}() - initial shape is not set"),
            ));
        }
        Ok(())
    }

    /// `Clear()`: resets the modification on the same shape; `draft.clear()`.
    pub fn clear(&mut self) {
        if let Some(draft) = self.draft.get() {
            draft.clear();
        }
    }

    /// `Init(S)`: a new initial shape; not done; `draft.init(&shape)?`.
    pub fn init(&mut self, s: &Shape) -> Result<(), Failure> {
        self.modify.my_initial_shape = s.clone();
        self.modify.set_not_done();
        if let Some(draft) = self.draft.get() {
            draft.init(s)
        } else {
            self.set_modification(Handle::new(DraftModification::new(s)?));
            Ok(())
        }
    }

    /// `Add(F, Direction, Angle, NeutralPlane, Flag)`: the face to draft (nothing for an
    /// angle up to 1e-4); Standard_NullObject without an initial shape;
    /// `draft.add(&face, dir, 0.1, &plane, true)?`.
    pub fn add(
        &mut self,
        f: &Face,
        d: Direction3,
        angle: f64,
        plane: &Plane3,
        flag: bool,
    ) -> Result<(), Failure> {
        // POP-DPF : protection
        if angle.abs() <= 1.0e-04 {
            return Ok(());
        }
        self.check_initial_shape("Add")?;
        self.modification()?.add(f, d, angle, plane, flag)?;
        Ok(())
    }

    /// `AddDone()`: no problematic shape; `draft.add_done()?`.
    pub fn add_done(&self) -> Result<bool, Failure> {
        self.check_initial_shape("AddDone")?;
        Ok(self.modification()?.problematic_shape().is_null())
    }

    /// `Remove(F)`: removes the face and the faces added with it; `draft.remove(&face)?`.
    pub fn remove(&mut self, f: &Face) -> Result<(), Failure> {
        self.check_initial_shape("Remove")?;
        self.modification()?.remove(f)
    }

    /// `ProblematicShape()`; `draft.problematic_shape()?`.
    pub fn problematic_shape(&self) -> Result<Shape, Failure> {
        self.check_initial_shape("ProblematicShape")?;
        Ok(self.modification()?.problematic_shape())
    }

    /// `Status()`; `draft.status()?`.
    pub fn status(&self) -> Result<DraftErrorStatus, Failure> {
        self.check_initial_shape("Status")?;
        Ok(self.modification()?.error())
    }

    /// `ConnectedFaces(F)`; `draft.connected_faces(&face)?`.
    pub fn connected_faces(&self, f: &Face) -> Result<List<Shape>, Failure> {
        self.check_initial_shape("ConnectedFaces")?;
        self.modification()?.connected_faces(f)
    }

    /// `ModifiedFaces()`; `draft.modified_faces()?`.
    pub fn modified_faces(&self) -> Result<List<Shape>, Failure> {
        self.check_initial_shape("ModifiedFaces")?;
        self.modification()?.modified_faces()
    }

    /// `ModifiedShape(S)`: the replaced vertex, else the image in the modifier (through
    /// the vertex substitution); `draft.modified_shape(&shape)?`.
    pub fn modified_shape(&self, s: &Shape) -> Result<Shape, Failure> {
        if s.shape_type()? == ShapeEnum::Vertex
            && let Some(v) = self.my_vtx_to_replace.seek(s)
        {
            return Ok(v.clone());
        }
        if self.my_vtx_to_replace.is_empty() {
            self.modify.my_modifier.modified_shape(s)
        } else {
            let a_ns = self.modify.my_modifier.modified_shape(s)?;
            self.my_subs.value(&a_ns)
        }
    }

    /// Whether the modification gives the face a new surface.
    fn has_new_surface(&self, s: &Shape) -> Result<bool, Failure> {
        let mut result = NewSurface::default();
        self.modification()?.new_surface(&topods::face(s)?, &mut result)
    }

    /// `CorrectVertexTol()`: the vertices of the new edges get the edge tolerance (the
    /// original vertices are replaced by new ones, through `mySubs`).
    fn correct_vertex_tol(&mut self) -> Result<(), Failure> {
        let mut an_init_vertices = MapOfShape::default();
        let mut an_init_edges = MapOfShape::default();
        let mut a_new_edges = MapOfShape::default();
        for e in Explorer::new(&self.modify.my_initial_shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
            for v in ShapeIterator::new(&e, true, true) {
                an_init_vertices.add(v);
            }
            an_init_edges.add(e);
        }

        let a_bb = BRepBuilder::default();
        self.my_vtx_to_replace.clear(true);
        let shape = self.modify.my_shape.clone();
        for an_e in Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
            // Skip old (not modified) edges
            if an_init_edges.contains(&an_e) {
                continue;
            }
            // Skip processed edges
            if a_new_edges.contains(&an_e) {
                continue;
            }
            a_new_edges.add(an_e.clone());

            let an_e_tol = BRepTool::edge_tolerance(&topods::edge(&an_e)?)?;
            for a_vtx in ShapeIterator::new(&an_e, true, true) {
                let a_vtx = topods::vertex(&a_vtx)?;
                if an_init_vertices.contains(&a_vtx) {
                    if let Some(replacement) = self.my_vtx_to_replace.seek(&a_vtx) {
                        a_bb.update_vertex_tolerance(&topods::vertex(replacement)?, an_e_tol + epsilon(an_e_tol))?;
                    } else {
                        let a_v_tol = BRepTool::vertex_tolerance(&a_vtx)?;
                        if a_v_tol < an_e_tol {
                            let mut a_new_vtx = Vertex::default();
                            let a_v_pnt = BRepTool::pnt(&a_vtx)?;
                            a_bb.make_vertex_at(&mut a_new_vtx, a_v_pnt, an_e_tol + epsilon(an_e_tol))?;
                            a_new_vtx.set_orientation(a_vtx.orientation());
                            self.my_vtx_to_replace.bind(a_vtx.into_shape(), a_new_vtx.into_shape());
                        }
                    }
                } else {
                    a_bb.update_vertex_tolerance(&a_vtx, an_e_tol + epsilon(an_e_tol))?;
                }
            }
        }

        if self.my_vtx_to_replace.is_empty() {
            return Ok(());
        }

        self.my_subs.clear();
        for (k, v) in self.my_vtx_to_replace.items() {
            self.my_subs.replace(k, v)?;
        }
        self.my_subs.apply(&shape, ShapeEnum::Shape)?;
        self.modify.my_shape = self.my_subs.value(&shape)?;
        Ok(())
    }
}

impl BRepBuilderAPIMakeShape for BRepOffsetAPIDraftAngle {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        self.modify.make_shape_base()
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        self.modify.make_shape_base_mut()
    }

    /// `Build(theRange)`: performs the draft modification, then the modifier on the
    /// initial shape, corrects the wires crossing seams and the vertex tolerances; not
    /// done when the draft fails.
    fn build(&mut self, _range: &ProgressRange<'_>) -> Result<(), Failure> {
        self.modification()?.perform()?;
        if !self.modification()?.is_done() {
            self.modify.set_not_done();
        } else {
            let initial = self.modify.my_initial_shape.clone();
            self.modify.do_modif_shape(&initial)?;
            self.correct_wires()?;
            self.correct_vertex_tol()?;
        }
        Ok(())
    }

    /// `Generated(S)`: the image of a face given a new surface.
    fn generated(&mut self, s: &Shape) -> Result<&List<Shape>, Failure> {
        self.modify.my_generated.clear();
        self.check_initial_shape("Generated")?;
        if s.shape_type()? == ShapeEnum::Face && self.has_new_surface(s)? {
            let image = if self.my_vtx_to_replace.is_empty() {
                self.modified_shape(s)?
            } else {
                self.my_subs.value(&self.modified_shape(s)?)?
            };
            self.modify.my_generated.append(image);
        }
        Ok(&self.modify.my_generated)
    }

    /// `Modified(S)`: the image of a face without a new surface (empty when it is the
    /// face itself).
    fn modified(&mut self, s: &Shape) -> Result<&List<Shape>, Failure> {
        self.modify.my_generated.clear();
        self.check_initial_shape("Modified")?;
        if s.shape_type()? == ShapeEnum::Face && !self.has_new_surface(s)? {
            // Ce n est pas une generation => peut etre une  modif
            let image = if self.my_vtx_to_replace.is_empty() {
                self.modified_shape(s)?
            } else {
                self.my_subs.value(&self.modified_shape(s)?)?
            };
            self.modify.my_generated.append(image);
            if self.modify.my_generated.len() == 1
                && self.modify.my_generated.first().is_some_and(|f| f.is_same(s))
            {
                self.modify.my_generated.clear();
            }
        }
        Ok(&self.modify.my_generated)
    }
}
