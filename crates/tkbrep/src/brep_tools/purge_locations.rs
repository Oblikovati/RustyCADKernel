// Rust translation of ModelingData/TKBRep/BRepTools/BRepTools_PurgeLocations.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_PurgeLocations`: removes the negative and scaling locations of a shape by
//! transforming its geometry.
use super::modification::BRepToolsModification;
use super::{BRepToolsReShape, Modifier, ReShape, TrsfModification};
use crate::topods::{Shape, ShapeIterator};
use crate::toptools::{IndexedMapOfShape, LocationSet, ShapeMapHasher};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Transform3;
use rustycad_tkmath::toploc::Location;

/// Whether a transformation is negative or scales by more than `TopLoc_Location::ScalePrec()`.
fn is_bad(trsf: &Transform3) -> bool {
    trsf.is_negative() || (trsf.scale_factor().abs() - 1.0).abs() > Location::scale_prec()
}

/// Removes the bad locations of a shape (`BRepTools_PurgeLocations`): each sub-shape
/// located by a negative or scaling transformation is replaced by a copy whose
/// geometry is transformed instead, and the shape is rebuilt with them.
///
/// ```
/// use rustycad_tkbrep::brep_tools::PurgeLocations;
/// use rustycad_tkbrep::topods::Shape;
/// let mut purge = PurgeLocations::new();
/// assert!(purge.perform(&Shape::default())?);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct PurgeLocations {
    done: bool,
    shape: Shape,
    shapes: IndexedMapOfShape,
    locations: LocationSet,
    new_shapes: DataMap<Shape, Shape, ShapeMapHasher>,
    reshape: Handle<BRepToolsReShape>,
}

impl PurgeLocations {
    /// `BRepTools_PurgeLocations()`; `PurgeLocations::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Perform(theShape)`: collects the sub-shapes and their locations, replaces the
    /// sub-shapes whose location is bad and rebuilds the shape; false when a
    /// modification failed (the location is then moved back instead);
    /// `purge.perform(&shape)?`.
    pub fn perform(&mut self, shape: &Shape) -> Result<bool, Failure> {
        self.shape = shape.clone();
        self.shapes.clear(true);
        self.locations.clear();
        self.done = true;
        self.add_shape(shape)?;
        let bad = self.bad_location_indices();
        if bad.is_empty() {
            return Ok(self.done);
        }
        self.new_shapes.clear(true);
        for index in 1..=self.shapes.extent() {
            let current = self.shapes.find_key(index)?.clone();
            let location_index = self.locations.index(current.location());
            if location_index == 0 || !bad.contains(&location_index) {
                continue;
            }
            let (purged, is_done) = purge_location(&current)?;
            self.done = self.done && is_done;
            self.new_shapes.bind(current, purged);
        }
        self.apply_replacements()?;
        Ok(self.done)
    }

    /// The indices of the bad locations, up to the first identity location.
    fn bad_location_indices(&self) -> Vec<i32> {
        let mut bad = Vec::new();
        for index in 1.. {
            let location = self.locations.location(index);
            if location.is_identity() {
                break;
            }
            if is_bad(&location.transformation()) {
                bad.push(index);
            }
        }
        bad
    }

    /// Replaces the purged sub-shapes in a (new or cleared) re-shaper and applies it.
    fn apply_replacements(&mut self) -> Result<(), Failure> {
        if self.reshape.is_null() {
            self.reshape = Handle::new(BRepToolsReShape::new());
        } else if let Some(reshape) = self.reshape.get() {
            reshape.clear();
        }
        let Some(reshape) = self.reshape.get() else {
            return Ok(());
        };
        let mut cursor = self.new_shapes.cursor();
        while cursor.more() {
            reshape.replace(cursor.key()?, cursor.value()?)?;
            cursor.next();
        }
        self.shape = reshape.apply(&self.shape, ShapeEnum::Shape)?;
        Ok(())
    }

    /// `AddShape(theS)`: the shape, its location, then its sub-shapes (without
    /// cumulated orientation and location).
    fn add_shape(&mut self, shape: &Shape) -> Result<(), Failure> {
        self.shapes.add(shape.clone());
        self.locations.add(shape.location());
        for child in ShapeIterator::new(shape, false, false) {
            self.add_shape(&child)?;
        }
        Ok(())
    }

    /// `GetResult()`; `purge.result()`.
    pub fn result(&self) -> &Shape {
        &self.shape
    }

    /// `IsDone()`; `purge.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `ModifiedShape(theInitShape)`: the purged copy of a sub-shape, the shape itself
    /// when it was kept; `purge.modified_shape(&face)`.
    pub fn modified_shape(&self, shape: &Shape) -> Shape {
        self.new_shapes
            .seek(shape)
            .cloned()
            .unwrap_or_else(|| shape.clone())
    }
}

/// `PurgeLocation(theS, theRes)`: the shape without location, then each elementary
/// location of its chain applied in turn: a bad one through a TrsfModification of the
/// geometry, a good one as a move; false when a modification failed.
fn purge_location(shape: &Shape) -> Result<(Shape, bool), Failure> {
    let mut remaining = shape.location().clone();
    if remaining.is_identity() {
        return Ok((shape.clone(), true));
    }
    let mut result = shape.located(&Location::default(), false)?;
    let mut is_done = true;
    while !remaining.is_identity() {
        let datum = remaining.first_datum()?.clone();
        let power = remaining.first_power()?;
        let bad = datum.get().is_some_and(|d| is_bad(d.trsf()));
        let location = Location::from_datum(datum).powered(power)?;
        if bad {
            let modification: Handle<dyn BRepToolsModification> =
                Handle::new(TrsfModification::new(location.transformation())).cast();
            let modifier = Modifier::with_modification(&result, &modification)?;
            if modifier.is_done() {
                result = modifier.modified_shape(&result)?;
            } else {
                is_done = false;
                result = result.moved(&location, false)?;
            }
        } else {
            result = result.moved(&location, false)?;
        }
        remaining = remaining.next_location().clone();
    }
    Ok((result, is_done))
}
