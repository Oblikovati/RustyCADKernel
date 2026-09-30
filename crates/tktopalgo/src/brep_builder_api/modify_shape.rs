// Rust translation of BRepBuilderAPI_ModifyShape.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_ModifyShape`: the constant-topology modifications, which rebuild a
//! shape through a `BRepTools_Modifier` and trace each sub-shape to its image.
use super::make_shape::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
use rustycad_tkbrep::brep_tools::{BRepToolsModification, Modifier};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use std::ops::{Deref, DerefMut};

/// `BRepBuilderAPI_ModifyShape` (docs/topalgo-design.md section 10): the modifier, the
/// initial shape and the modification of a constant-topology modification. The native
/// constructors and `DoModif` are protected, and the fields too; they are public here
/// because derived classes live in other crates (BRepOffsetAPI_DraftAngle).
///
/// ```
/// use rustycad_tkbrep::brep_tools::{BRepToolsModification, CopyModification};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_builder_api::{
///     BRepBuilderAPIMakeShape, BRepBuilderAPIMakeVertex, BRepBuilderAPIModifyShape,
/// };
/// let vertex = BRepBuilderAPIMakeVertex::new(Point3::new(1.0, 2.0, 3.0))?.vertex()?;
/// let copy: Handle<dyn BRepToolsModification> = Handle::new(CopyModification::new(true, false)).cast();
/// let mut modify = BRepBuilderAPIModifyShape::with_shape_modification(&vertex, &copy)?;
/// assert!(modify.is_done());
/// assert!(!modify.shape()?.is_same(&vertex));
/// assert!(modify.modified_shape(&vertex)?.is_same(modify.shape()?));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct BRepBuilderAPIModifyShape {
    base: BRepBuilderAPIMakeShapeBase,
    /// `myModifier`.
    pub my_modifier: Modifier,
    /// `myInitialShape`.
    pub my_initial_shape: Shape,
    /// `myModification`.
    pub my_modification: Handle<dyn BRepToolsModification>,
}

impl Deref for BRepBuilderAPIModifyShape {
    type Target = BRepBuilderAPIMakeShapeBase;
    fn deref(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }
}

impl DerefMut for BRepBuilderAPIModifyShape {
    fn deref_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }
}

impl BRepBuilderAPIMakeShape for BRepBuilderAPIModifyShape {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `Modified(S)`: the one image of `S` in the modifier; Standard_NoSuchObject (after
    /// the list was cleared) for a shape that is not a sub-shape of the initial shape;
    /// `modify.modified(&face)?`.
    fn modified(&mut self, shape: &Shape) -> Result<&List<Shape>, Failure> {
        self.base.my_generated.clear();
        let image = self.my_modifier.modified_shape(shape)?;
        self.base.my_generated.append(image);
        Ok(&self.base.my_generated)
    }
}

impl BRepBuilderAPIModifyShape {
    /// `BRepBuilderAPI_ModifyShape()` (protected): no shape and no modification;
    /// `BRepBuilderAPIModifyShape::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepBuilderAPI_ModifyShape(S)` (protected): the modifier initialized on `S`;
    /// `BRepBuilderAPIModifyShape::with_shape(&shape)?`.
    pub fn with_shape(shape: &Shape) -> Result<Self, Failure> {
        Ok(Self {
            my_modifier: Modifier::with_shape(shape)?,
            my_initial_shape: shape.clone(),
            ..Self::default()
        })
    }

    /// `BRepBuilderAPI_ModifyShape(M)` (protected): the modification only;
    /// `BRepBuilderAPIModifyShape::with_modification(&m)`.
    pub fn with_modification(modification: &Handle<dyn BRepToolsModification>) -> Self {
        Self {
            my_modification: modification.clone(),
            ..Self::default()
        }
    }

    /// `BRepBuilderAPI_ModifyShape(S, M)` (protected): the modifier initialized on `S`,
    /// then the modification performed;
    /// `BRepBuilderAPIModifyShape::with_shape_modification(&shape, &m)?`.
    pub fn with_shape_modification(
        shape: &Shape,
        modification: &Handle<dyn BRepToolsModification>,
    ) -> Result<Self, Failure> {
        let mut modify = Self::with_shape(shape)?;
        modify.my_modification = modification.clone();
        modify.do_modif()?;
        Ok(modify)
    }

    /// The private `DoModif()`: performs the modification on the initial shape;
    /// Standard_NullObject without a shape or a modification. Done with the image of the
    /// initial shape when the modifier is done, else not done.
    fn do_modif(&mut self) -> Result<(), Failure> {
        if self.my_initial_shape.is_null() || self.my_modification.is_null() {
            return Err(Failure::with_message(
                "Standard_NullObject",
                "BRepBuilderAPI_ModifyShape::DoModif: null initial shape or modification",
            ));
        }
        self.my_modifier
            .perform(&self.my_modification, &ProgressRange::default())?;
        if !self.my_modifier.is_done() {
            self.base.set_not_done();
            return Ok(());
        }
        self.base.set_done();
        self.base.my_shape = self.my_modifier.modified_shape(&self.my_initial_shape)?;
        Ok(())
    }

    /// `DoModif(S)` (protected): performs the modification on `S`, unless `S` is the
    /// initial shape (IsEqual) and the command is done; `modify.do_modif_shape(&s)?`.
    pub fn do_modif_shape(&mut self, shape: &Shape) -> Result<(), Failure> {
        // No class derived from ModifyShape overrides the virtual IsDone(), so the
        // command flag is what the native call reads (docs/topalgo-design.md section 10).
        if shape.is_equal(&self.my_initial_shape) && self.base.is_done() {
            return Ok(());
        }
        self.my_initial_shape = shape.clone();
        self.my_modifier.init(shape)?;
        self.do_modif()
    }

    /// `DoModif(M)` (protected): performs `M` on the initial shape;
    /// `modify.do_modif_modification(&m)?`.
    pub fn do_modif_modification(
        &mut self,
        modification: &Handle<dyn BRepToolsModification>,
    ) -> Result<(), Failure> {
        self.my_modification = modification.clone();
        self.do_modif()
    }

    /// `DoModif(S, M)` (protected): performs `M` on `S`;
    /// `modify.do_modif_shape_modification(&s, &m)?`.
    pub fn do_modif_shape_modification(
        &mut self,
        shape: &Shape,
        modification: &Handle<dyn BRepToolsModification>,
    ) -> Result<(), Failure> {
        self.my_initial_shape = shape.clone();
        self.my_modifier.init(shape)?;
        self.my_modification = modification.clone();
        self.do_modif()
    }

    /// `ModifiedShape(S)` (virtual; the derived classes shadow it): the image of `S` in
    /// the modifier; Standard_NoSuchObject for a shape that is not a sub-shape of the
    /// initial shape; `modify.modified_shape(&face)?`.
    pub fn modified_shape(&self, shape: &Shape) -> Result<Shape, Failure> {
        self.my_modifier.modified_shape(shape)
    }
}
