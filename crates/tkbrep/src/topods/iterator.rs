// Rust translation of ModelingData/TKBRep/TopoDS/TopoDS_Iterator.hxx and
// TopoDS_Iterator.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopoDS_Iterator`: the direct sub-shapes of a shape (docs/topology-design.md
//! section 6).
use super::Shape;
use super::tshape::TShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkmath::toploc::Location;

/// Iterates on the direct sub-shapes of a shape, composing each with the orientation
/// and the location of the shape when asked.
///
/// ```
/// use rustycad_tkbrep::topods::{Builder, Compound, ShapeIterator, Wire};
/// let (mut compound, mut wire) = (Compound::default(), Wire::default());
/// Builder.make_compound(&mut compound);
/// Builder.make_wire(&mut wire);
/// Builder.add(&mut compound, &wire).unwrap();
/// let mut iterator = ShapeIterator::new(&compound, true, true);
/// assert!(iterator.more() && iterator.value().unwrap().is_same(&wire));
/// iterator.next();
/// assert!(!iterator.more());
/// ```
#[derive(Clone)]
pub struct ShapeIterator {
    shape: Shape,
    tshape: Handle<dyn TShape>,
    index: usize,
    orientation: Orientation,
    location: Location,
}

impl Default for ShapeIterator {
    /// `TopoDS_Iterator()`: an empty iterator, FORWARD, at the identity.
    fn default() -> Self {
        Self {
            shape: Shape::default(),
            tshape: Handle::default(),
            index: 0,
            orientation: Orientation::Forward,
            location: Location::default(),
        }
    }
}

impl ShapeIterator {
    /// `TopoDS_Iterator(S, cumOri, cumLoc)`; `ShapeIterator::new(&shape, true, true)`.
    pub fn new(shape: &Shape, cum_ori: bool, cum_loc: bool) -> Self {
        let mut iterator = Self::default();
        iterator.initialize(shape, cum_ori, cum_loc);
        iterator
    }

    /// `Initialize(S, cumOri, cumLoc)`: restarts on the sub-shapes of `shape`, taking its
    /// location (`cum_loc`) and orientation (`cum_ori`) as the cumulated ones;
    /// `iterator.initialize(&shape, true, true)`.
    pub fn initialize(&mut self, shape: &Shape, cum_ori: bool, cum_loc: bool) {
        self.location = if cum_loc {
            shape.location().clone()
        } else {
            Location::default()
        };
        self.orientation = if cum_ori {
            shape.orientation()
        } else {
            Orientation::Forward
        };
        self.tshape = shape.tshape().clone();
        self.index = 0;
        self.update_current_shape();
    }

    /// `More()`: true while a sub-shape remains; `iterator.more()`.
    pub fn more(&self) -> bool {
        self.tshape
            .get()
            .is_some_and(|tshape| self.index < tshape.nb_children() as usize)
    }

    /// `Next()`: moves to the next sub-shape; past the end it does nothing, as the
    /// native list iterator; `iterator.next()`.
    pub fn next(&mut self) {
        if !self.more() {
            return;
        }
        self.index += 1;
        self.update_current_shape();
    }

    /// `Value()`: the current composed sub-shape; Standard_NoSuchObject when there is
    /// none; `iterator.value()?`.
    pub fn value(&self) -> Result<&Shape, Failure> {
        if !self.more() {
            return Err(Failure::with_type(
                "Standard_NoSuchObject",
                &format!(
                    "TopoDS_Iterator::Value: no sub-shape at position {}",
                    self.index
                ),
                "",
            ));
        }
        Ok(&self.shape)
    }

    /// `updateCurrentShape`: the child composed with the cumulated orientation and moved
    /// by the cumulated location.
    fn update_current_shape(&mut self) {
        let Some(child) = self
            .tshape
            .get()
            .and_then(|t| t.tshape_base().child(self.index))
        else {
            return;
        };
        self.shape = child;
        let orientation = top_abs::compose(self.orientation, self.shape.orientation());
        self.shape.set_orientation(orientation);
        if self.location.is_identity() {
            return;
        }
        // `Move(myLocation)`: the product only adds the powers of datums that both
        // locations already carry (a negative power exists only for a datum that was
        // inverted once), so it builds no item that could fail.
        if let Ok(location) = self.location.multiplied(self.shape.location()) {
            let (tshape, orientation) = (self.shape.tshape().clone(), self.shape.orientation());
            self.shape = Shape::from_parts(tshape, location, orientation);
        }
    }
}

/// The native `begin()`/`end()` range: yields each composed sub-shape.
impl Iterator for ShapeIterator {
    type Item = Shape;
    fn next(&mut self) -> Option<Shape> {
        let current = self.value().ok()?.clone();
        ShapeIterator::next(self);
        Some(current)
    }
}
