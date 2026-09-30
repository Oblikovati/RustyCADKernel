// Rust translation of BRepBuilderAPI_Transform.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_Transform`: a shape moved by a gp_Trsf, through a location when the
//! transformation keeps the geometry, else through `BRepTools_TrsfModification`.
use super::make_shape::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
use super::modify_shape::BRepBuilderAPIModifyShape;
use rustycad_tkbrep::brep_tools::{BRepToolsModification, TrsfModification};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Transform3;
use rustycad_tkmath::toploc::Location;
use std::ops::{Deref, DerefMut};

/// `BRepBuilderAPI_Transform` (docs/topalgo-design.md section 10): with a copy request,
/// a negative transformation or a scale factor away from 1 (beyond
/// `TopLoc_Location::ScalePrec()`), the geometry is rebuilt by the modification; else
/// the result is the shape moved by the location of the transformation, which shares
/// the geometry.
///
/// `myUseModif` is uninitialized natively until the first Perform; it starts false
/// here, so `ModifiedShape` and `Modified` before a Perform move by the identity.
///
/// ```
/// use rustycad_tkmath::gp::{Point3, Transform3};
/// use rustycad_tktopalgo::brep_builder_api::{
///     BRepBuilderAPIMakeShape, BRepBuilderAPIMakeVertex, BRepBuilderAPITransform,
/// };
/// let vertex = BRepBuilderAPIMakeVertex::new(Point3::new(1.0, 2.0, 3.0))?.vertex()?;
/// let mut moved = BRepBuilderAPITransform::with_shape(&vertex, Transform3::translation([1.0, 0.0, 0.0]), false, false)?;
/// assert!(moved.shape()?.is_partner(&vertex));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepBuilderAPITransform {
    modify: BRepBuilderAPIModifyShape,
    my_trsf: Transform3,
    my_location: Location,
    my_use_modif: bool,
}

impl Deref for BRepBuilderAPITransform {
    type Target = BRepBuilderAPIModifyShape;
    fn deref(&self) -> &BRepBuilderAPIModifyShape {
        &self.modify
    }
}

impl DerefMut for BRepBuilderAPITransform {
    fn deref_mut(&mut self) -> &mut BRepBuilderAPIModifyShape {
        &mut self.modify
    }
}

impl BRepBuilderAPIMakeShape for BRepBuilderAPITransform {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        self.modify.make_shape_base()
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        self.modify.make_shape_base_mut()
    }

    /// `Modified(S)`: `S` moved by the location, or the image of the modification;
    /// `transform.modified(&face)?`.
    fn modified(&mut self, shape: &Shape) -> Result<&List<Shape>, Failure> {
        if self.my_use_modif {
            return self.modify.modified(shape);
        }
        let generated = &mut self.modify.make_shape_base_mut().my_generated;
        generated.clear();
        generated.append(shape.moved(&self.my_location, false)?);
        Ok(generated)
    }
}

impl BRepBuilderAPITransform {
    /// `BRepBuilderAPI_Transform(T)`: the transformation and its modification, no shape
    /// yet; `BRepBuilderAPITransform::new(trsf)`.
    pub fn new(trsf: Transform3) -> Self {
        let modification: Handle<dyn BRepToolsModification> =
            Handle::new(TrsfModification::new(trsf)).cast();
        Self {
            modify: BRepBuilderAPIModifyShape::with_modification(&modification),
            my_trsf: trsf,
            my_location: Location::default(),
            my_use_modif: false,
        }
    }

    /// `BRepBuilderAPI_Transform(theShape, theTrsf, theCopyGeom, theCopyMesh)`: then
    /// [`Self::perform`]; `BRepBuilderAPITransform::with_shape(&s, trsf, false, false)?`.
    pub fn with_shape(
        shape: &Shape,
        trsf: Transform3,
        copy_geom: bool,
        copy_mesh: bool,
    ) -> Result<Self, Failure> {
        let mut transform = Self::new(trsf);
        transform.perform(shape, copy_geom, copy_mesh)?;
        Ok(transform)
    }

    /// `Perform(theShape, theCopyGeom, theCopyMesh)`: runs the modification (with the
    /// mesh copied on request) when the geometry must be rebuilt, else moves the shape
    /// by the location of the transformation; `transform.perform(&s, false, false)?`.
    pub fn perform(
        &mut self,
        shape: &Shape,
        copy_geom: bool,
        copy_mesh: bool,
    ) -> Result<(), Failure> {
        self.my_use_modif = copy_geom
            || self.my_trsf.is_negative()
            || ((self.my_trsf.scale_factor().abs() - 1.0).abs() > Location::scale_prec());
        if !self.my_use_modif {
            self.my_location = Location::new(self.my_trsf);
            let base = self.modify.make_shape_base_mut();
            base.my_shape = shape.moved(&self.my_location, false)?;
            base.set_done();
            return Ok(());
        }
        let modification: Handle<TrsfModification> = self.modify.my_modification.cast();
        if let Some(the_modif) = modification.get() {
            the_modif.set_trsf(self.my_trsf);
            the_modif.set_copy_mesh(copy_mesh);
        }
        let modification = self.modify.my_modification.clone();
        self.modify
            .do_modif_shape_modification(shape, &modification)
    }

    /// `ModifiedShape(S)`: `S` moved by the location, or its image in the modifier
    /// (Standard_NoSuchObject for a foreign shape); `transform.modified_shape(&face)?`.
    pub fn modified_shape(&self, shape: &Shape) -> Result<Shape, Failure> {
        if self.my_use_modif {
            return self.modify.modified_shape(shape);
        }
        shape.moved(&self.my_location, false)
    }
}
