// Rust translation of BRepBuilderAPI_GTransform.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_GTransform`: a shape deformed by a general (affine) transformation:
//! its geometry is first converted to B-splines, then transformed by
//! `BRepTools_GTrsfModification`.
use super::collect::BRepBuilderAPICollect;
use super::make_shape::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
use super::modify_shape::BRepBuilderAPIModifyShape;
use super::nurbs_convert::BRepBuilderAPINurbsConvert;
use rustycad_tkbrep::brep_tools::{BRepToolsModification, GTrsfModification};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::GeneralTransform3;
use std::ops::{Deref, DerefMut};

/// `BRepBuilderAPI_GTransform` (docs/topalgo-design.md section 10): the history of the
/// NURBS conversion is kept in a `BRepBuilderAPI_Collect`, so `Modified` and
/// `ModifiedShape` take an initial sub-shape through both steps.
///
/// `Modified(S)` returns the image of the last conversion image of `S` only (natively
/// too: each image overwrites the list), and an empty list for a shape the conversion
/// did not record.
///
/// ```
/// use rustycad_tkmath::gp::{GeneralTransform3, Point3};
/// use rustycad_tktopalgo::brep_builder_api::{
///     BRepBuilderAPIGTransform, BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape,
/// };
/// let edge = BRepBuilderAPIMakeEdge::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0))?.edge()?;
/// let mut deformed = BRepBuilderAPIGTransform::with_shape(&edge, GeneralTransform3::default(), false)?;
/// assert!(deformed.is_done());
/// assert!(!deformed.modified_shape(&edge)?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepBuilderAPIGTransform {
    modify: BRepBuilderAPIModifyShape,
    my_gtrsf: GeneralTransform3,
    my_hist: BRepBuilderAPICollect,
}

impl Deref for BRepBuilderAPIGTransform {
    type Target = BRepBuilderAPIModifyShape;
    fn deref(&self) -> &BRepBuilderAPIModifyShape {
        &self.modify
    }
}

impl DerefMut for BRepBuilderAPIGTransform {
    fn deref_mut(&mut self) -> &mut BRepBuilderAPIModifyShape {
        &mut self.modify
    }
}

impl BRepBuilderAPIMakeShape for BRepBuilderAPIGTransform {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        self.modify.make_shape_base()
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        self.modify.make_shape_base_mut()
    }

    /// `Modified(S)`: for each conversion image of `S`, the list becomes its image in
    /// the modifier (the base `Modified`); empty when the conversion did not record `S`;
    /// `gtransform.modified(&face)?`.
    fn modified(&mut self, shape: &Shape) -> Result<&List<Shape>, Failure> {
        self.modify.make_shape_base_mut().my_generated.clear();
        let converted: Vec<Shape> = match self.my_hist.modification().seek(shape) {
            Some(images) => images.iter().cloned().collect(),
            None => Vec::new(),
        };
        for image in &converted {
            // The native copy into a local list (Li.Assign) is discarded.
            self.modify.modified(image)?;
        }
        Ok(&self.modify.make_shape_base().my_generated)
    }
}

impl BRepBuilderAPIGTransform {
    /// `BRepBuilderAPI_GTransform(T)`: the transformation and its modification, no shape
    /// yet; `BRepBuilderAPIGTransform::new(gtrsf)`.
    pub fn new(gtrsf: GeneralTransform3) -> Self {
        let modification: Handle<dyn BRepToolsModification> =
            Handle::new(GTrsfModification::new(gtrsf)).cast();
        Self {
            modify: BRepBuilderAPIModifyShape::with_modification(&modification),
            my_gtrsf: gtrsf,
            my_hist: BRepBuilderAPICollect::new(),
        }
    }

    /// `BRepBuilderAPI_GTransform(S, T, Copy)`: then [`Self::perform`];
    /// `BRepBuilderAPIGTransform::with_shape(&s, gtrsf, false)?`.
    pub fn with_shape(
        shape: &Shape,
        gtrsf: GeneralTransform3,
        copy: bool,
    ) -> Result<Self, Failure> {
        let mut gtransform = Self::new(gtrsf);
        gtransform.perform(shape, copy)?;
        Ok(gtransform)
    }

    /// `Perform(S, Copy)`: converts `S` to B-splines (recording that history), then
    /// runs the transformation on the converted shape; `gtransform.perform(&s, false)?`.
    pub fn perform(&mut self, shape: &Shape, copy: bool) -> Result<(), Failure> {
        let mut nc = BRepBuilderAPINurbsConvert::new();
        nc.perform(shape, copy)?;
        self.my_hist.add(shape, &mut nc)?;
        let s_local = nc.shape()?.clone();
        let modification: Handle<GTrsfModification> = self.modify.my_modification.cast();
        if let Some(the_modif) = modification.get() {
            the_modif.set_gtrsf(self.my_gtrsf);
        }
        let modification = self.modify.my_modification.clone();
        self.modify
            .do_modif_shape_modification(&s_local, &modification)
        //  myHist.Filter (Shape());
    }

    /// `ModifiedShape(S)`: the image in the modifier of the first conversion image of
    /// `S` (of `S` itself when the conversion did not record it);
    /// `gtransform.modified_shape(&face)?`.
    pub fn modified_shape(&self, shape: &Shape) -> Result<Shape, Failure> {
        let converted: Option<&Shape> = self
            .my_hist
            .modification()
            .seek(shape)
            .and_then(|images| images.first());
        self.modify.modified_shape(converted.unwrap_or(shape))
    }
}
