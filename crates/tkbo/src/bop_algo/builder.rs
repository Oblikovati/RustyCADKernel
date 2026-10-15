//! `BOPAlgo_Builder`: the General Fuse algorithm, building the splits of the arguments from
//! the intersection results of the pave filler.
use super::{BOPAlgoPaveFiller, BuilderShape, Glue};
use crate::int_tools::Context;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List, Map};
use rustycad_tkernel::standard::{Failure, Handle};
use std::ops::{Deref, DerefMut};

#[path = "builder/bop.rs"]
mod bop;
#[path = "builder/containers.rs"]
mod containers;
#[path = "builder/draft_face.rs"]
mod draft_face;
#[path = "builder/faces.rs"]
mod faces;
#[path = "builder/history.rs"]
pub(super) mod history;
#[path = "builder/images.rs"]
mod images;
#[path = "builder/images_edges.rs"]
mod images_edges;
#[path = "builder/internal_shapes.rs"]
mod internal_shapes;
#[path = "builder/perform.rs"]
pub(super) mod perform;
#[path = "builder/same_domain_faces.rs"]
mod same_domain_faces;
#[path = "builder/solids.rs"]
mod solids;
#[path = "builder/split_solids.rs"]
mod split_solids;
pub(crate) use bop::operation_states;
pub use perform::{BuilderHooks, perform_on};
pub use solids::fill_images_solids_with;

/// Shapes and their image, origin or IN-part lists (`myImages`, `myOrigins`, `myInParts`).
pub type ShapeImages = DataMap<Shape, List<Shape>, ShapeMapHasher>;
/// Shapes and their same-domain representative (`myShapesSD`).
pub type SameDomainShapes = DataMap<Shape, Shape, ShapeMapHasher>;

/// The General Fuse algorithm: splits every argument by the others and keeps the images,
/// origins and same-domain links of all sub-shapes.
///
/// ```
/// use rustycad_tkbo::bop_algo::BOPAlgoBuilder;
/// let builder = BOPAlgoBuilder::new();
/// assert!(builder.arguments().is_empty() && builder.shape().is_null());
/// ```
pub struct BOPAlgoBuilder {
    base: BuilderShape,
    pub(super) arguments: List<Shape>,
    argument_fence: Map<Shape, ShapeMapHasher>,
    /// The filler created by `perform` (`myPaveFiller` at entry point 1).
    pub(super) filler: Option<BOPAlgoPaveFiller>,
    pub(super) context: Handle<Context>,
    pub(super) images: ShapeImages,
    pub(super) shapes_sd: SameDomainShapes,
    pub(super) origins: ShapeImages,
    pub(super) in_parts: ShapeImages,
    pub(super) non_destructive: bool,
    pub(super) glue: Glue,
    pub(super) check_inverted: bool,
}

impl Default for BOPAlgoBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BOPAlgoBuilder {
    type Target = BuilderShape;
    fn deref(&self) -> &BuilderShape {
        &self.base
    }
}

impl DerefMut for BOPAlgoBuilder {
    fn deref_mut(&mut self) -> &mut BuilderShape {
        &mut self.base
    }
}

impl BOPAlgoBuilder {
    /// A builder with no arguments and the native defaults (`BOPAlgo_Builder()`).
    /// `BOPAlgoBuilder::new()`.
    pub fn new() -> Self {
        Self {
            base: BuilderShape::new(),
            arguments: List::default(),
            argument_fence: Map::with_hasher(ShapeMapHasher, 100),
            filler: None,
            context: Handle::default(),
            images: DataMap::with_hasher(ShapeMapHasher, 100),
            shapes_sd: DataMap::with_hasher(ShapeMapHasher, 100),
            origins: DataMap::with_hasher(ShapeMapHasher, 100),
            in_parts: DataMap::with_hasher(ShapeMapHasher, 100),
            non_destructive: false,
            glue: Glue::Off,
            check_inverted: true,
        }
    }

    /// Adds a unique shape identity; reversed occurrences are duplicates (`AddArgument`).
    /// `builder.add_argument(&shape)`.
    pub fn add_argument(&mut self, shape: &Shape) {
        if self.argument_fence.add(shape.clone()) {
            self.arguments.append(shape.clone());
        }
    }

    /// Replaces the argument list, the fence kept until `clear` (`SetArguments`).
    /// `builder.set_arguments(&shapes)`.
    pub fn set_arguments(&mut self, shapes: &List<Shape>) {
        self.arguments.clear();
        for shape in shapes.iter() {
            self.add_argument(shape);
        }
    }

    /// The unique arguments (`Arguments`); `builder.arguments().len()`.
    pub fn arguments(&self) -> &List<Shape> {
        &self.arguments
    }

    /// Clears the report, the history, the arguments and the image maps, keeping the
    /// options (`Clear`); `builder.clear()`.
    pub fn clear(&mut self) {
        self.base.clear();
        self.arguments.clear();
        self.argument_fence.clear(false);
        self.images.clear(false);
        self.shapes_sd.clear(false);
        self.origins.clear(false);
        self.in_parts.clear(false);
    }

    /// The filler created by `perform` (`PPaveFiller`); `None` after `perform_with_filler`.
    /// `builder.pave_filler()`.
    pub fn pave_filler(&self) -> Option<&BOPAlgoPaveFiller> {
        self.filler.as_ref()
    }

    /// The geometric context of the last run (`Context`); `builder.context()`.
    pub fn context(&self) -> &Handle<Context> {
        &self.context
    }

    /// Sets the non-destructive mode (`SetNonDestructive`); `builder.set_non_destructive(true)`.
    pub fn set_non_destructive(&mut self, enabled: bool) {
        self.non_destructive = enabled;
    }

    /// The non-destructive mode (`NonDestructive`); `builder.non_destructive()`.
    pub fn non_destructive(&self) -> bool {
        self.non_destructive
    }

    /// Sets the glue mode (`SetGlue`); `builder.set_glue(Glue::Shift)`.
    pub fn set_glue(&mut self, glue: Glue) {
        self.glue = glue;
    }

    /// The glue mode (`Glue`); `builder.glue()`.
    pub fn glue(&self) -> Glue {
        self.glue
    }

    /// Sets the inverted-solid check (`SetCheckInverted`); `builder.set_check_inverted(false)`.
    pub fn set_check_inverted(&mut self, check: bool) {
        self.check_inverted = check;
    }

    /// The inverted-solid check (`CheckInverted`); `builder.check_inverted()`.
    pub fn check_inverted(&self) -> bool {
        self.check_inverted
    }

    /// The splits of the sub-shapes of the arguments (`Images`); `builder.images()`.
    pub fn images(&self) -> &ShapeImages {
        &self.images
    }

    /// The sub-shapes of the arguments each split comes from (`Origins`); `builder.origins()`.
    pub fn origins(&self) -> &ShapeImages {
        &self.origins
    }

    /// The same-domain representative of the splits (`ShapesSD`); `builder.shapes_sd()`.
    pub fn shapes_sd(&self) -> &SameDomainShapes {
        &self.shapes_sd
    }

    /// The solids and the faces inside them (`myInParts`); `builder.in_parts()`.
    pub fn in_parts(&self) -> &ShapeImages {
        &self.in_parts
    }

    /// `myImages`, to write, for the derived algorithms that rebuild the splits
    /// (BRepFeat_Builder); `builder.images_mut()`.
    pub fn images_mut(&mut self) -> &mut ShapeImages {
        &mut self.images
    }

    /// `myOrigins`, to write; `builder.origins_mut()`.
    pub fn origins_mut(&mut self) -> &mut ShapeImages {
        &mut self.origins
    }

    /// `myShapesSD`, to write; `builder.shapes_sd_mut()`.
    pub fn shapes_sd_mut(&mut self) -> &mut SameDomainShapes {
        &mut self.shapes_sd
    }

    /// `myInParts`, to write; `builder.in_parts_mut()`.
    pub fn in_parts_mut(&mut self) -> &mut ShapeImages {
        &mut self.in_parts
    }

    /// Takes the filler kept by `perform` (to work on its data structure while the
    /// images change); give it back with [`Self::restore_pave_filler`];
    /// `builder.take_pave_filler()`.
    pub fn take_pave_filler(&mut self) -> Option<BOPAlgoPaveFiller> {
        self.filler.take()
    }

    /// Gives back a filler taken by [`Self::take_pave_filler`];
    /// `builder.restore_pave_filler(filler)`.
    pub fn restore_pave_filler(&mut self, filler: Option<BOPAlgoPaveFiller>) {
        self.filler = filler;
    }
}

/// The context taken from the pave filler.
fn required_context(handle: &Handle<Context>) -> Result<&Context, Failure> {
    super::builder_area::required_context(handle, "BOPAlgo_Builder")
}
