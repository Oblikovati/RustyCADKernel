// Rust translation of ModelingData/TKBRep/BRepTools/BRepTools_ReShape.hxx and the members
// Clear, Remove, replace, IsRecorded, Value, ValueLeaf, Status, CopyVertex and IsNewShape of
// BRepTools_ReShape.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_ReShape`: records replacements and removals of shapes and applies them.
mod apply;
mod history;

use crate::brep::{BRepBuilder, BRepTool};
use crate::topods::{self, Shape, Vertex};
use crate::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient, Transient};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;

pub use apply::ApplyGuard;

/// `TReplacementKind`: how a shape is replaced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplacementKind {
    /// `TReplacementKind_Remove`.
    Remove = 1,
    /// `TReplacementKind_Modify`.
    Modify = 2,
    /// `TReplacementKind_Merge_Main`: the first part of a merge.
    MergeMain = 4,
    /// `TReplacementKind_Merge_Ordinary`: the other parts of a merge.
    MergeOrdinary = 8,
}

/// `TReplacement`: the result of a replacement and its kind.
#[derive(Clone, Debug)]
pub struct Replacement {
    result: Shape,
    kind: ReplacementKind,
}

impl Replacement {
    /// `Result()`: the result, null for an ordinary merged part.
    pub fn result(&self) -> Shape {
        if self.kind == ReplacementKind::MergeOrdinary {
            return Shape::default();
        }
        self.result.clone()
    }

    /// `RelationResult()`: the stored result.
    pub fn relation_result(&self) -> &Shape {
        &self.result
    }

    /// `RelationKind()`: removed for a removal, modified otherwise.
    pub fn is_removal(&self) -> bool {
        self.kind == ReplacementKind::Remove
    }
}

/// The fields of `BRepTools_ReShape`.
#[derive(Default)]
pub struct ReShapeFields {
    /// `myShapeToReplacement`.
    pub replacements: DataMap<Shape, Replacement, ShapeMapHasher>,
    /// `myNewShapes`.
    pub new_shapes: MapOfShape,
    /// `myStatus`.
    pub status: i32,
    /// `myConsiderLocation`.
    pub consider_location: bool,
}

/// The data of a re-shaper: its fields behind a lock, since a re-shaper is shared
/// through handles.
pub struct ReShapeData {
    fields: SharedState<ReShapeFields>,
}

impl Default for ReShapeData {
    /// `BRepTools_ReShape()`: no replacement, status -1, locations not considered.
    fn default() -> Self {
        Self {
            fields: SharedState::new(ReShapeFields {
                status: -1,
                ..ReShapeFields::default()
            }),
        }
    }
}

impl ReShapeData {
    /// Runs `read` on the fields; `data.with(|fields| fields.status)`.
    pub fn with<R>(&self, read: impl FnOnce(&ReShapeFields) -> R) -> R {
        read(&self.fields.read())
    }

    /// Runs `edit` on the fields; `data.with_mut(|fields| fields.status = 0)`.
    pub fn with_mut<R>(&self, edit: impl FnOnce(&mut ReShapeFields) -> R) -> R {
        edit(&mut self.fields.write())
    }
}

/// `BRepTools_ReShape`: the virtual members are provided trait methods with the native
/// bodies, the others are provided methods written once over [`ReShapeData`]; a
/// derived re-shaper (ShapeBuild_ReShape natively) overrides the virtual ones.
///
/// ```
/// use rustycad_tkbrep::brep_tools::{BRepToolsReShape, ReShape};
/// use rustycad_tkbrep::topods::Shape;
/// let reshape = BRepToolsReShape::new();
/// assert!(!reshape.is_recorded(&Shape::default()));
/// ```
pub trait ReShape: Transient {
    /// The embedded data.
    fn reshape_data(&self) -> &ReShapeData;

    /// `Clear()` (virtual): forgets every replacement and new shape; `reshape.clear()`.
    fn clear(&self) {
        self.reshape_data().with_mut(|fields| {
            fields.replacements.clear(true);
            fields.new_shapes.clear(true);
        });
    }

    /// `Remove(shape)` (virtual): records the removal of `shape`; `reshape.remove(&s)?`.
    fn remove(&self, shape: &Shape) -> Result<(), Failure> {
        self.replace_with_kind(shape, &Shape::default(), ReplacementKind::Remove)
    }

    /// `Replace(shape, newshape)` (virtual): records the replacement of `shape`;
    /// `reshape.replace(&old, &new)?`.
    fn replace(&self, shape: &Shape, new_shape: &Shape) -> Result<(), Failure> {
        self.replace_with_kind(shape, new_shape, ReplacementKind::Modify)
    }

    /// `Merge(theParts, theProduct)`: the first part is replaced by the product, the
    /// others are merged into it; `reshape.merge(&parts, &product)?`.
    fn merge(&self, parts: &[Shape], product: &Shape) -> Result<(), Failure> {
        let Some((first, others)) = parts.split_first() else {
            return Ok(());
        };
        self.replace_with_kind(first, product, ReplacementKind::MergeMain)?;
        for part in others {
            self.replace_with_kind(part, product, ReplacementKind::MergeOrdinary)?;
        }
        Ok(())
    }

    /// `replace(shape, newshape, theKind)` (virtual, protected): a REVERSED shape is
    /// stored FORWARD with the new shape reversed, an INTERNAL or EXTERNAL one FORWARD
    /// with the new shape FORWARD when their orientations match (REVERSED otherwise);
    /// with the location mode, the location of the shape moves onto the new shape. A
    /// null shape or one equal to its replacement is ignored; cycles are accepted and
    /// broken by `Apply`; `reshape.replace_with_kind(&old, &new, ReplacementKind::Modify)?`.
    fn replace_with_kind(
        &self,
        shape: &Shape,
        new_shape: &Shape,
        kind: ReplacementKind,
    ) -> Result<(), Failure> {
        let (mut shape, mut new_shape) = (shape.clone(), new_shape.clone());
        if shape.is_null() || shape == new_shape {
            return Ok(());
        }
        let orientation = shape.orientation();
        if orientation == Orientation::Reversed {
            shape.reverse();
            new_shape.reverse();
        } else if matches!(orientation, Orientation::Internal | Orientation::External) {
            let same = new_shape.orientation() == orientation;
            new_shape.set_orientation(if same {
                Orientation::Forward
            } else {
                Orientation::Reversed
            });
            shape.set_orientation(Orientation::Forward);
        }
        if self.mode_consider_location() {
            // sln 29.11.01 Bug22: the new shape takes the location of the shape.
            let location = new_shape
                .location()
                .multiplied(&shape.location().inverted()?)?;
            new_shape.set_location(&location, false)?;
            shape.set_location(&Location::default(), false)?;
        }
        self.reshape_data().with_mut(|fields| {
            fields.replacements.bind(
                shape,
                Replacement {
                    result: new_shape.clone(),
                    kind,
                },
            );
            fields.new_shapes.add(new_shape);
        });
        Ok(())
    }

    /// `IsRecorded(shape)` (virtual): a replacement exists for the shape (located at
    /// the identity with the location mode); `reshape.is_recorded(&s)`.
    fn is_recorded(&self, shape: &Shape) -> bool {
        let mut shape = shape.clone();
        if self.mode_consider_location() && shape.set_location(&Location::default(), false).is_err()
        {
            return false;
        }
        if shape.is_null() {
            return false;
        }
        self.reshape_data()
            .with(|fields| fields.replacements.is_bound(&shape))
    }

    /// `Value(shape)` (virtual): the direct replacement (reversed for a REVERSED shape;
    /// INTERNAL and EXTERNAL orientations are kept), the shape itself when none; with the
    /// location mode, relocated by the location of the shape; `reshape.value(&s)?`.
    fn value(&self, shape: &Shape) -> Result<Shape, Failure> {
        if shape.is_null() {
            return Ok(Shape::default());
        }
        let mut key = shape.clone();
        let consider = self.mode_consider_location();
        if consider {
            key.set_location(&Location::default(), false)?;
        }
        let stored = self
            .reshape_data()
            .with(|fields| fields.replacements.seek(&key).map(Replacement::result));
        let from_map = stored.is_some();
        let mut result = match stored {
            Some(mut result) => {
                if key.orientation() == Orientation::Reversed {
                    result.reverse();
                }
                result
            }
            None => key.clone(),
        };
        if matches!(
            key.orientation(),
            Orientation::Internal | Orientation::External
        ) {
            result.set_orientation(key.orientation());
        }
        if consider {
            let location = if from_map {
                shape.location().multiplied(result.location())?
            } else {
                shape.location().clone()
            };
            result.set_location(&location, false)?;
        }
        Ok(result)
    }

    /// `ValueLeaf(theShape)`: follows the replacements to the end of the chain (null for
    /// a removal), stopping at a TShape met before (a cycle); `reshape.value_leaf(&s)?`.
    fn value_leaf(&self, shape: &Shape) -> Result<Shape, Failure> {
        if shape.is_null() {
            return Ok(Shape::default());
        }
        let mut current = shape.clone();
        let mut visited = vec![current.tshape_address()];
        loop {
            let next = self.value(&current)?;
            if next.is_null() || next.is_same(&current) {
                return Ok(next);
            }
            if visited.contains(&next.tshape_address()) {
                // A cycle in the replacements: the current best result.
                return Ok(next);
            }
            visited.push(next.tshape_address());
            current = next;
        }
    }

    /// `Status(shape, newsh, last)` (virtual): 0 for an unrecorded shape or a
    /// replacement equal to it, 1 for a replacement, -1 for a removal; with `last`, a
    /// replacement is followed through `Apply`. Returns the status and `newsh`;
    /// `let (status, result) = reshape.status(&s, false)?;`.
    fn status(&self, shape: &Shape, last: bool) -> Result<(i32, Shape), Failure> {
        apply::status(self, shape, last)
    }

    /// `Apply(theShape, theUntil)` (virtual): the shape with the replacements applied
    /// recursively down to the type `until` (TopAbs_SHAPE for all levels); a sub-shape
    /// replaced by a shape that contains it is not descended again;
    /// `reshape.apply(&s, ShapeEnum::Shape)?`.
    fn apply(&self, shape: &Shape, until: ShapeEnum) -> Result<Shape, Failure> {
        let mut guard = ApplyGuard::default();
        self.apply_impl(shape, until, &mut guard)
    }

    /// `applyImpl(theShape, theUntil, theInFlight)` (protected); see [`ReShape::apply`].
    fn apply_impl(
        &self,
        shape: &Shape,
        until: ShapeEnum,
        in_flight: &mut ApplyGuard,
    ) -> Result<Shape, Failure> {
        apply::apply_impl(self, shape, until, in_flight)
    }

    /// `ModeConsiderLocation()` (virtual, returns a reference natively); the location
    /// mode; `reshape.mode_consider_location()`.
    fn mode_consider_location(&self) -> bool {
        self.reshape_data().with(|fields| fields.consider_location)
    }

    /// `ModeConsiderLocation() = mode`; `reshape.set_mode_consider_location(true)`.
    fn set_mode_consider_location(&self, mode: bool) {
        self.reshape_data()
            .with_mut(|fields| fields.consider_location = mode);
    }

    /// `CopyVertex(theV, theTol)`: [`ReShape::copy_vertex_at`] at the vertex point;
    /// `reshape.copy_vertex(&v, -1.0)?`.
    fn copy_vertex(&self, vertex: &Vertex, tolerance: f64) -> Result<Vertex, Failure> {
        self.copy_vertex_at(vertex, BRepTool::pnt(vertex)?, tolerance)
    }

    /// `CopyVertex(theV, theNewPos, aTol)`: the recorded replacement of the vertex (else
    /// an empty copy, then recorded) moved to `position` with `tolerance` (the vertex
    /// tolerance when not positive); `reshape.copy_vertex_at(&v, point, 1e-7)?`.
    fn copy_vertex_at(
        &self,
        vertex: &Vertex,
        position: Point3,
        tolerance: f64,
    ) -> Result<Vertex, Failure> {
        let recorded = self.is_recorded(vertex);
        let copy = if recorded {
            topods::vertex(&self.apply(vertex, ShapeEnum::Shape)?)?
        } else {
            topods::vertex(&vertex.empty_copied()?)?
        };
        let new_tolerance = if tolerance > 0.0 {
            tolerance
        } else {
            BRepTool::vertex_tolerance(vertex)?
        };
        BRepBuilder::default().update_vertex_point(&copy, position, new_tolerance)?;
        if !recorded {
            self.replace(vertex, &copy)?;
        }
        Ok(copy)
    }

    /// `IsNewShape(theShape)`: the shape is a recorded replacement;
    /// `reshape.is_new_shape(&s)`.
    fn is_new_shape(&self, shape: &Shape) -> bool {
        self.reshape_data()
            .with(|fields| fields.new_shapes.contains(shape))
    }

    /// `History()`: the history of the recorded replacements;
    /// `reshape.history()?`.
    fn history(&self) -> Result<rustycad_tkernel::standard::Handle<super::History>, Failure> {
        history::history_of(self.reshape_data())
    }
}
impl_transient_interface!(dyn ReShape, "BRepTools_ReShape", StandardTransient);

/// The base class `BRepTools_ReShape` itself.
#[derive(Default)]
pub struct BRepToolsReShape {
    parent: StandardTransient,
    data: ReShapeData,
}
impl_transient!(BRepToolsReShape, "BRepTools_ReShape", StandardTransient, parent, interfaces: [dyn ReShape]);

impl BRepToolsReShape {
    /// `BRepTools_ReShape()`; `BRepToolsReShape::new()`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ReShape for BRepToolsReShape {
    fn reshape_data(&self) -> &ReShapeData {
        &self.data
    }
}
