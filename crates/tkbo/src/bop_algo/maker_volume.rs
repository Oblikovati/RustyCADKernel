//! `BOPAlgo_MakerVolume`: the solids bounded by the faces of the arguments.
#[path = "maker_volume/volumes.rs"]
mod volumes;
use super::alerts::alert;
use super::builder::perform::{adopt_filler, no_structure, perform_on};
use super::{
    AlertBuilderFailed, AlertTooFewArguments, BOPAlgoBuilder, BOPAlgoPaveFiller, BuilderHooks,
};
use crate::bop_ds::DataStructure;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Shape, Solid};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox3;
use std::ops::{Deref, DerefMut};

/// Builds the volumes the faces of the arguments bound: the arguments are intersected (or
/// not), their faces in both orientations and the faces of an enlarged bounding box are
/// split into solids, and the solid touching the box is removed.
///
/// ```
/// use rustycad_tkbo::bop_algo::BOPAlgoMakerVolume;
/// let mut maker = BOPAlgoMakerVolume::new();
/// maker.perform();
/// // No argument to make a volume of.
/// assert!(maker.options().has_errors());
/// ```
pub struct BOPAlgoMakerVolume {
    builder: BOPAlgoBuilder,
    intersect: bool,
    bounds: BoundingBox3,
    bounding_solid: Solid,
    faces: List<Shape>,
    avoid_internal_shapes: bool,
}

impl Default for BOPAlgoMakerVolume {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BOPAlgoMakerVolume {
    type Target = BOPAlgoBuilder;
    fn deref(&self) -> &BOPAlgoBuilder {
        &self.builder
    }
}

impl DerefMut for BOPAlgoMakerVolume {
    fn deref_mut(&mut self) -> &mut BOPAlgoBuilder {
        &mut self.builder
    }
}

impl BOPAlgoMakerVolume {
    /// A maker intersecting its arguments and keeping internal shapes
    /// (`BOPAlgo_MakerVolume()`; the allocator overload maps here too);
    /// `BOPAlgoMakerVolume::new()`.
    pub fn new() -> Self {
        Self {
            builder: BOPAlgoBuilder::new(),
            intersect: true,
            bounds: BoundingBox3::default(),
            bounding_solid: Solid::default(),
            faces: List::default(),
            avoid_internal_shapes: false,
        }
    }

    /// Forgets the shapes and restores the default flags (`Clear`); `maker.clear()`.
    pub fn clear(&mut self) {
        self.builder.clear();
        self.intersect = true;
        self.bounds = BoundingBox3::default();
        self.bounding_solid = Solid::default();
        self.faces.clear();
        self.avoid_internal_shapes = false;
    }

    /// Whether the arguments are intersected first (`SetIntersect`);
    /// `maker.set_intersect(false)`.
    pub fn set_intersect(&mut self, intersect: bool) {
        self.intersect = intersect;
    }

    /// Whether the arguments are intersected first (`IsIntersect`); `maker.is_intersect()`.
    pub fn is_intersect(&self) -> bool {
        self.intersect
    }

    /// The enlarged bounding box solid of the last run (`Box`); `maker.bounding_box()`.
    pub fn bounding_box(&self) -> &Solid {
        &self.bounding_solid
    }

    /// The faces given to the solid builder, the box faces last (`Faces`); `maker.faces()`.
    pub fn faces(&self) -> &List<Shape> {
        &self.faces
    }

    /// Whether the edges and vertices of the arguments stay out of the solids
    /// (`SetAvoidInternalShapes`); `maker.set_avoid_internal_shapes(true)`.
    pub fn set_avoid_internal_shapes(&mut self, avoid: bool) {
        self.avoid_internal_shapes = avoid;
    }

    /// Whether internal shapes are avoided (`IsAvoidInternalShapes`);
    /// `maker.is_avoid_internal_shapes()`.
    pub fn is_avoid_internal_shapes(&self) -> bool {
        self.avoid_internal_shapes
    }

    /// Intersects the arguments, or their compound when `is_intersect` is false, with an own
    /// pave filler and builds the volumes (`Perform`); `maker.perform()`.
    pub fn perform(&mut self) {
        let shapes = if self.intersect {
            self.builder.arguments.clone()
        } else {
            match arguments_compound(&self.builder.arguments) {
                Ok(compound) => [compound].into_iter().collect(),
                Err(_) => {
                    // Natively the exception leaves Perform; here it is the builder failure.
                    self.builder.options().add_error(&alert(AlertBuilderFailed));
                    return;
                }
            }
        };
        perform_on(self, shapes);
    }

    /// The volume pipeline (`PerformInternal1`): the splits of vertices, edges, wires and
    /// faces when intersecting, the faces, the box, the solids, the internal shapes, the
    /// result, the history and the post treatment.
    fn run(&mut self, filler: &mut BOPAlgoPaveFiller) -> Result<(), Failure> {
        adopt_filler(&mut self.builder, filler);
        self.check_data(filler);
        if self.builder.options().has_errors() {
            return Ok(());
        }
        self.builder.prepare();
        let structure = filler.data_structure_mut().ok_or_else(no_structure)?;
        if self.intersect && !self.fill_images(structure)? {
            return Ok(());
        }
        if !self.build_volumes(structure)? {
            return Ok(());
        }
        self.builder.prepare_history(structure)?;
        if self.builder.options().has_errors() {
            return Ok(());
        }
        self.builder.post_treat(structure)
    }

    /// The splits of vertices, edges, wires and faces; false after an error.
    fn fill_images(&mut self, structure: &mut DataStructure) -> Result<bool, Failure> {
        let builder = &mut self.builder;
        builder.fill_images_vertices(structure)?;
        if builder.options().has_errors() {
            return Ok(false);
        }
        builder.fill_images_edges(structure)?;
        if builder.options().has_errors() {
            return Ok(false);
        }
        builder.fill_images_containers(rustycad_tkg3d::top_abs::ShapeEnum::Wire, structure)?;
        if builder.options().has_errors() {
            return Ok(false);
        }
        builder.fill_images_faces(structure)?;
        Ok(!builder.options().has_errors())
    }
}

impl BuilderHooks for BOPAlgoMakerVolume {
    fn general_fuse(&mut self) -> &mut BOPAlgoBuilder {
        &mut self.builder
    }

    /// At least one argument, then the filler report merged (`CheckData`).
    fn check_data(&mut self, filler: &BOPAlgoPaveFiller) {
        if self.builder.arguments.is_empty() {
            self.builder
                .options()
                .add_error(&alert(AlertTooFewArguments));
            return;
        }
        self.builder.check_filler(filler);
    }

    fn perform_internal1(&mut self, filler: &mut BOPAlgoPaveFiller) -> Result<(), Failure> {
        self.run(filler)
    }
}

/// One compound of the arguments, intersected as a single argument.
fn arguments_compound(arguments: &List<Shape>) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for argument in arguments.iter() {
        builder.add(&mut compound, argument)?;
    }
    Ok(compound.into_shape())
}
