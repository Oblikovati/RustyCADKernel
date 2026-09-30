// Rust translation of ModelingAlgorithms/TKShHealing/ShapeUpgrade/ShapeUpgrade_RemoveLocations.hxx/.lxx
// and the members Remove and MakeNewShape of ShapeUpgrade_RemoveLocations.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeUpgrade_RemoveLocations`: moves the locations of sub-shapes into their geometry
//! (docs/shhealing-design.md section 8).
mod rebuild;

use rebuild::{rebuild_edge, rebuild_face, rebuild_vertex};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::toploc::Location;

/// `NCollection_DataMap<TopoDS_Shape, TopoDS_Shape, TopTools_ShapeMapHasher>`: each
/// original sub-shape (without location) and its new shape.
pub type ShapeReplacements = DataMap<Shape, Shape, ShapeMapHasher>;

/// The fields of a `ShapeUpgrade_RemoveLocations`.
struct RemoveLocationsFields {
    level_removing: ShapeEnum,
    shape: Shape,
    map_new_shapes: ShapeReplacements,
}

/// `ShapeUpgrade_RemoveLocations`: rebuilds a shape so that the sub-shapes of the types
/// from the removal level down carry no location: the transformation moves into their
/// surfaces, curves and points.
///
/// ```
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeRemoveLocations;
/// let remover = ShapeUpgradeRemoveLocations::new();
/// assert_eq!(remover.remove_level(), ShapeEnum::Shape);
/// remover.set_remove_level(ShapeEnum::Vertex);
/// assert_eq!(remover.remove_level(), ShapeEnum::Face);
/// ```
pub struct ShapeUpgradeRemoveLocations {
    parent: StandardTransient,
    fields: SharedState<RemoveLocationsFields>,
}
impl_transient!(
    ShapeUpgradeRemoveLocations,
    "ShapeUpgrade_RemoveLocations",
    StandardTransient,
    parent
);

impl Default for ShapeUpgradeRemoveLocations {
    fn default() -> Self {
        Self::new()
    }
}

impl ShapeUpgradeRemoveLocations {
    /// `ShapeUpgrade_RemoveLocations()`: the removal level TopAbs_SHAPE (every level
    /// except compounds); `ShapeUpgradeRemoveLocations::new()`.
    pub fn new() -> Self {
        Self {
            parent: StandardTransient::default(),
            fields: SharedState::new(RemoveLocationsFields {
                level_removing: ShapeEnum::Shape,
                shape: Shape::default(),
                map_new_shapes: ShapeReplacements::default(),
            }),
        }
    }

    /// `Remove(theShape)`: rebuilds the shape; true when a sub-shape was rebuilt (the
    /// result is then [`Self::get_result`]). The map of new shapes is kept from earlier
    /// calls, as natively. Standard_NullObject for a null shape;
    /// `remover.remove(&shape)?`.
    pub fn remove(&self, shape: &Shape) -> Result<bool, Failure> {
        let shape_type = shape.shape_type()?;
        let level = self.remove_level();
        let remove_loc = (shape_type != ShapeEnum::Compound && level == ShapeEnum::Shape)
            || (level as i32) <= (shape_type as i32);
        let mut work = Remover {
            level,
            map: std::mem::take(&mut self.fields.write().map_new_shapes),
        };
        self.fields.write().shape = shape.clone();
        let result = work.make_new_shape(shape, &Shape::default(), remove_loc);
        let mut fields = self.fields.write();
        fields.map_new_shapes = work.map;
        let (done, new_shape) = result?;
        fields.shape = new_shape;
        Ok(done)
    }

    /// `GetResult()`: the rebuilt shape (the shape given to `Remove` while it failed);
    /// `remover.get_result()`.
    pub fn get_result(&self) -> Shape {
        self.fields.read().shape.clone()
    }

    /// `SetRemoveLevel(theLevel)`: the level, capped to TopAbs_FACE for the types below
    /// the face; `remover.set_remove_level(ShapeEnum::Solid)`.
    pub fn set_remove_level(&self, level: ShapeEnum) {
        let level = if (level as i32) > (ShapeEnum::Face as i32) {
            ShapeEnum::Face
        } else {
            level
        };
        self.fields.write().level_removing = level;
    }

    /// `RemoveLevel()`; `remover.remove_level()`.
    pub fn remove_level(&self) -> ShapeEnum {
        self.fields.read().level_removing
    }

    /// `ModifiedShape(theInitShape)`: the new shape of an original sub-shape, the shape
    /// itself when it is not in the map; `remover.modified_shape(&face)`.
    pub fn modified_shape(&self, init_shape: &Shape) -> Shape {
        let fields = self.fields.read();
        fields
            .map_new_shapes
            .seek(init_shape)
            .cloned()
            .unwrap_or_else(|| init_shape.clone())
    }

    /// `GetModifiedShapesMap()`: lends the map of the original sub-shapes and their new
    /// shapes (natively a reference to the field); `remover.get_modified_shapes_map(|m| m.len())`.
    pub fn get_modified_shapes_map<R>(&self, read: impl FnOnce(&ShapeReplacements) -> R) -> R {
        read(&self.fields.read().map_new_shapes)
    }
}

/// The state `MakeNewShape` works on: the removal level and the map of new shapes.
struct Remover {
    level: ShapeEnum,
    map: ShapeReplacements,
}

/// The shape and its new shape when the map binds it (natively `aNewShape` after the
/// `isBound` block).
struct BoundShape {
    bound: bool,
    new_shape: Shape,
}

impl Remover {
    /// `isRemoveLoc` of a sub-shape of type `shape_type`.
    fn removes_location(&self, shape_type: ShapeEnum) -> bool {
        (shape_type != ShapeEnum::Compound && self.level == ShapeEnum::Shape)
            || (self.level as i32) <= (shape_type as i32)
    }

    /// `MakeNewShape(theShape, theAncShape, theNewShape, theRemoveLoc)`: the new shape of
    /// `shape` (with `anc_shape`, the face it lies on) and whether something was rebuilt.
    fn make_new_shape(
        &mut self,
        shape: &Shape,
        anc_shape: &Shape,
        remove_loc: bool,
    ) -> Result<(bool, Shape), Failure> {
        let shape_type = shape.shape_type()?;
        let mut a_shape = shape.clone();
        if !remove_loc && !shape.location().is_identity() {
            a_shape.set_location(&Location::default(), false)?;
        }
        let found = self.bound_shape(shape, &a_shape, remove_loc)?;
        if found.bound && shape_type != ShapeEnum::Edge {
            return Ok((true, found.new_shape));
        }
        let is_remove_loc = remove_loc || self.removes_location(shape_type);
        let anc_shape = if shape_type == ShapeEnum::Face {
            a_shape.clone()
        } else {
            anc_shape.clone()
        };
        let mut new_shape = found.new_shape;
        let mut rebuilt = false;
        if is_remove_loc
            && (!a_shape.location().is_identity()
                || shape_type == ShapeEnum::Edge
                || shape_type == ShapeEnum::Face)
        {
            rebuilt = self.rebuild(&a_shape, &anc_shape, found.bound, &mut new_shape)?;
        }
        if found.bound {
            return Ok((true, new_shape));
        }
        let flags = LocationFlags {
            remove_loc,
            is_remove_loc,
            rebuilt,
        };
        let done = self.rebuild_children(shape, &a_shape, &anc_shape, flags, &mut new_shape)?;
        Ok((done, new_shape))
    }

    /// The `isBound` block: the bound new shape with the orientation of `shape` and, when
    /// the location is kept, its location.
    fn bound_shape(
        &self,
        shape: &Shape,
        a_shape: &Shape,
        remove_loc: bool,
    ) -> Result<BoundShape, Failure> {
        let Some(bound) = self.map.seek(a_shape) else {
            return Ok(BoundShape {
                bound: false,
                new_shape: Shape::default(),
            });
        };
        let mut new_shape = bound.clone();
        new_shape.set_orientation(shape.orientation());
        if !remove_loc && !shape.location().is_identity() {
            new_shape.set_location(shape.location(), false)?;
        }
        Ok(BoundShape {
            bound: true,
            new_shape,
        })
    }

    /// "Rebuild geometry for shape with location": the face, the edge (on the face it
    /// lies on) or the vertex without location.
    fn rebuild(
        &mut self,
        a_shape: &Shape,
        anc_shape: &Shape,
        bound: bool,
        new_shape: &mut Shape,
    ) -> Result<bool, Failure> {
        match a_shape.shape_type()? {
            ShapeEnum::Face => {
                let old_face = topods::face(a_shape)?;
                let mut new_face = Face::default();
                let rebuilt = rebuild_face(&old_face, &mut new_face)?;
                if rebuilt {
                    *new_shape = new_face.into_shape();
                    self.map.bind(old_face.into_shape(), new_shape.clone());
                }
                Ok(rebuilt)
            }
            ShapeEnum::Edge => self.rebuild_edge_on(a_shape, anc_shape, bound, new_shape),
            ShapeEnum::Vertex => {
                let mut new_vertex = topods::Vertex::default();
                let rebuilt = rebuild_vertex(&topods::vertex(a_shape)?, &mut new_vertex)?;
                if rebuilt {
                    *new_shape = new_vertex.into_shape();
                }
                Ok(rebuilt)
            }
            _ => Ok(false),
        }
    }

    /// The edge branch: the new edge (the bound one, when bound) with its p-curves on
    /// the new face of `anc_shape`.
    fn rebuild_edge_on(
        &mut self,
        a_shape: &Shape,
        anc_shape: &Shape,
        bound: bool,
        new_shape: &mut Shape,
    ) -> Result<bool, Failure> {
        let old_edge = topods::edge(a_shape)?;
        let (mut face, mut new_face) = (Face::default(), Face::default());
        if !anc_shape.is_null() {
            face = topods::face(anc_shape)?;
            new_face = face.clone();
            if let Some(found) = self.map.seek(&face) {
                new_face = topods::face(found)?;
            }
        }
        let mut new_edge = if bound {
            topods::edge(new_shape)?
        } else {
            Edge::default()
        };
        let rebuilt = rebuild_edge(&old_edge, &mut new_edge, &face, &new_face, bound)?;
        *new_shape = new_edge.into_shape();
        Ok(rebuilt)
    }

    /// "Removing location from sub-shapes in dependence of LevelRemoving and re-building
    /// shape": the children rebuilt into an empty copy (the shape itself when nothing
    /// changed), bound in the map; the new shape gets the original location back when it
    /// is kept.
    fn rebuild_children(
        &mut self,
        shape: &Shape,
        a_shape: &Shape,
        anc_shape: &Shape,
        flags: LocationFlags,
        new_shape: &mut Shape,
    ) -> Result<bool, Failure> {
        let mut done = flags.rebuilt;
        if !flags.rebuilt {
            *new_shape = shape.empty_copied()?;
            // "it is safe to simply copy Closed flag since this operation does not change topology"
            new_shape.set_closed(shape.closed()?)?;
        }
        let old_loc = shape.location().clone();
        if !old_loc.is_identity() {
            new_shape.set_location(&Location::default(), false)?;
        }
        let orient = shape.orientation();
        new_shape.set_orientation(Orientation::Forward);
        let builder = BRepBuilder::default();
        for sub_shape in ShapeIterator::new(a_shape, false, flags.is_remove_loc) {
            let (sub_done, new_sub_shape) =
                self.make_new_shape(&sub_shape, anc_shape, flags.is_remove_loc)?;
            done = done || sub_done;
            builder.add(new_shape, &new_sub_shape)?;
        }
        if done {
            new_shape.set_orientation(orient);
        } else {
            *new_shape = a_shape.clone();
        }
        self.map.bind(a_shape.clone(), new_shape.clone());
        if !flags.remove_loc && !old_loc.is_identity() {
            new_shape.set_location(&old_loc, false)?;
        }
        Ok(done)
    }
}

/// The flags of one `MakeNewShape` call: the caller's `theRemoveLoc` (which decides
/// whether the location comes back), the effective `isRemoveLoc` and whether the shape
/// itself was rebuilt.
#[derive(Clone, Copy)]
struct LocationFlags {
    remove_loc: bool,
    is_remove_loc: bool,
    rebuilt: bool,
}
