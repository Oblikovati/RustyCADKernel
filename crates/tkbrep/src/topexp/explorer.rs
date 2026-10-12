// Rust translation of ModelingData/TKBRep/TopExp/TopExp_Explorer.hxx and
// TopExp_Explorer.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopExp_Explorer`: a depth-first search for the sub-shapes of one type
//! (docs/topology-design.md section 6).
use crate::topods::errors::null_shape;
use crate::topods::{Shape, ShapeIterator};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Explores the sub-shapes of type `to_find` of a shape, skipping those under a shape
/// of type `to_avoid` (no effect when `to_avoid` is SHAPE or not more complex than
/// `to_find`).
///
/// ```
/// use rustycad_tkbrep::topexp::Explorer;
/// use rustycad_tkbrep::topods::{Builder, Compound, Wire};
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// let (mut compound, mut wire) = (Compound::default(), Wire::default());
/// Builder.make_compound(&mut compound);
/// Builder.make_wire(&mut wire);
/// Builder.add(&mut compound, &wire).unwrap();
/// let mut explorer = Explorer::new(&compound, ShapeEnum::Wire, ShapeEnum::Shape).unwrap();
/// assert!(explorer.more() && explorer.current().is_same(&wire));
/// explorer.next().unwrap();
/// assert!(!explorer.more());
/// ```
#[derive(Clone)]
pub struct Explorer {
    stack: Vec<ShapeIterator>,
    active_depth: usize,
    shape: Shape,
    to_find: ShapeEnum,
    to_avoid: ShapeEnum,
    has_more: bool,
}

impl Default for Explorer {
    /// `TopExp_Explorer()`: an empty explorer, useful after `init`.
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            active_depth: 0,
            shape: Shape::default(),
            to_find: ShapeEnum::Shape,
            to_avoid: ShapeEnum::Shape,
            has_more: false,
        }
    }
}

impl Explorer {
    /// `TopExp_Explorer(S, ToFind, ToAvoid)`; the native default `ToAvoid` is
    /// `ShapeEnum::Shape`; `Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape)?`.
    pub fn new(shape: &Shape, to_find: ShapeEnum, to_avoid: ShapeEnum) -> Result<Self, Failure> {
        let mut explorer = Self::default();
        explorer.init(shape, to_find, to_avoid)?;
        Ok(explorer)
    }

    /// `Init(S, ToFind, ToAvoid)`: restarts the exploration of `shape`; the shape itself
    /// is found when it has the type `to_find`; `explorer.init(&shape, find, avoid)?`.
    pub fn init(
        &mut self,
        shape: &Shape,
        to_find: ShapeEnum,
        to_avoid: ShapeEnum,
    ) -> Result<(), Failure> {
        self.clear();
        let Some(kind) = self.initialize_search(shape, to_find, to_avoid) else {
            return Ok(());
        };
        self.has_more = true;
        if kind != to_find {
            self.next()?;
        }
        Ok(())
    }

    fn initialize_search(
        &mut self,
        shape: &Shape,
        to_find: ShapeEnum,
        to_avoid: ShapeEnum,
    ) -> Option<ShapeEnum> {
        self.shape = shape.clone();
        self.to_find = to_find;
        self.to_avoid = to_avoid;
        // Nothing is found in a null shape, for SHAPE, or in a shape simpler than `to_find`.
        let kind = shape.tshape().get().map(|tshape| tshape.shape_type());
        kind.filter(|kind| to_find != ShapeEnum::Shape && *kind <= to_find)
    }

    /// `More()`: true while a shape remains; `explorer.more()`.
    pub fn more(&self) -> bool {
        self.has_more
    }

    /// `Next()`: moves to the next shape of the searched type. After the end the stack
    /// is empty and a new call restarts the exploration from the root, as natively;
    /// Standard_NullObject when the explored shape is null (a crash natively);
    /// `explorer.next()?`.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol; Iterator is implemented too.
    pub fn next(&mut self) -> Result<(), Failure> {
        if !self.advance_search_level()? {
            return Ok(());
        }
        self.descend();
        Ok(())
    }

    /// The loop of `Next`: goes down into more complex shapes that are not avoided,
    /// stops on a shape of the searched type, pops exhausted levels.
    fn descend(&mut self) {
        while self.active_depth > 0 {
            if self.advance_descent_step() {
                self.has_more = true;
                return;
            }
        }
        self.has_more = false;
    }
    fn advance_search_level(&mut self) -> Result<bool, Failure> {
        if self.active_depth > 0 {
            self.stack[self.active_depth - 1].next();
            return Ok(true);
        }
        let kind = self
            .shape
            .shape_type()
            .map_err(|_| null_shape("TopExp_Explorer::Next"))?;
        if kind == self.to_find || self.should_avoid(kind) {
            self.has_more = false;
            return Ok(false);
        }
        self.push_iterator(ShapeIterator::new(&self.shape, true, true));
        Ok(true)
    }
    fn advance_descent_step(&mut self) -> bool {
        let top = &mut self.stack[self.active_depth - 1];
        let step = match top.value() {
            Err(_) => Step::Pop,
            Ok(current) => classify(current, self.to_find, self.to_avoid),
        };
        match step {
            Step::Found => return true,
            Step::Push(child) => self.push_iterator(child),
            Step::Skip => top.next(),
            Step::Pop => self.pop_iterator(),
        }
        false
    }

    /// `popIterator` followed by `myStack[myStackTop].Next()` when a level remains.
    fn pop_iterator(&mut self) {
        self.active_depth -= 1;
        // OCCT 3d097a0 resets in place: retained Current references observe null then slot reuse.
        self.stack[self.active_depth] = ShapeIterator::default();
        if self.active_depth > 0 {
            self.stack[self.active_depth - 1].next();
        }
    }
    fn push_iterator(&mut self, child: ShapeIterator) {
        if self.active_depth == self.stack.len() {
            self.stack.push(child);
        } else {
            self.stack[self.active_depth] = child;
        }
        self.active_depth += 1;
    }

    fn should_avoid(&self, kind: ShapeEnum) -> bool {
        should_avoid(kind, self.to_avoid)
    }

    /// `Current()` (and `Value()`): the current shape; at the end, and before any
    /// descent, the explored shape itself; `explorer.current()`.
    pub fn current(&self) -> &Shape {
        match self.stack[..self.active_depth].last() {
            // A level on the stack always has a current shape (exhausted levels are popped).
            Some(top) => top.value().unwrap_or(&self.shape),
            None => &self.shape,
        }
    }

    /// `Value()`: the same as [`Explorer::current`]; `explorer.value()`.
    pub fn value(&self) -> &Shape {
        self.current()
    }

    /// `ReInit()`: restarts with the original arguments; `explorer.re_init()?`.
    pub fn re_init(&mut self) -> Result<(), Failure> {
        let shape = self.shape.clone();
        self.init(&shape, self.to_find, self.to_avoid)
    }

    /// `ExploredShape()`: the shape given to `init`; `explorer.explored_shape()`.
    pub fn explored_shape(&self) -> &Shape {
        &self.shape
    }

    /// `Depth()`: the stack depth, 0 for the explored shape itself; `explorer.depth()`.
    pub fn depth(&self) -> i32 {
        self.active_depth as i32
    }

    /// `Clear()`: empties the stack and ends the exploration; `explorer.clear()`.
    pub fn clear(&mut self) {
        for slot in &mut self.stack[..self.active_depth] {
            *slot = ShapeIterator::default();
        }
        self.active_depth = 0;
        self.has_more = false;
    }
}

/// What the loop of `Next` does with the top level of the stack.
enum Step {
    /// The current shape has the searched type.
    Found,
    /// Descend into the current shape.
    Push(ShapeIterator),
    /// Move the top iterator to its next shape.
    Skip,
    /// The top level is exhausted.
    Pop,
}

/// One turn of the native loop on the current shape of the top iterator. A child
/// always has a TShape (`Builder::add` refuses a null component).
fn classify(current: &Shape, to_find: ShapeEnum, to_avoid: ShapeEnum) -> Step {
    let Some(tshape) = current.tshape().get() else {
        return Step::Skip;
    };
    let kind = tshape.shape_type();
    if kind == to_find {
        return Step::Found;
    }
    // `isMoreComplex`: the enumeration runs from COMPOUND (0) to VERTEX (7).
    if to_find > kind && !should_avoid(kind, to_avoid) {
        return Step::Push(ShapeIterator::new(current, true, true));
    }
    Step::Skip
}

/// `shouldAvoid`: `toAvoid` is not SHAPE and equals the type.
fn should_avoid(kind: ShapeEnum, to_avoid: ShapeEnum) -> bool {
    to_avoid != ShapeEnum::Shape && to_avoid == kind
}

/// The native `begin()`/`end()` range: yields each found shape. An error of `next`
/// (a null explored shape) ends the iteration.
impl Iterator for Explorer {
    type Item = Shape;
    fn next(&mut self) -> Option<Shape> {
        if !self.more() {
            return None;
        }
        let current = self.current().clone();
        Explorer::next(self).ok()?;
        Some(current)
    }
}

/// Identifies native Current storage rather than its detached value; `explorer.current_shape_slot()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExplorerShapeSlot {
    /// The explorer's own stable root-shape member.
    Root,
    /// An inline iterator slot retained after pop/reset and reused on later descent.
    Iterator(usize),
}
impl Explorer {
    /// Bind the actual current source slot; `let slot = explorer.current_shape_slot()`.
    pub fn current_shape_slot(&self) -> ExplorerShapeSlot {
        match self.active_depth.checked_sub(1) {
            Some(index) => ExplorerShapeSlot::Iterator(index),
            None => ExplorerShapeSlot::Root,
        }
    }
    /// Read live storage, including inactive reset slots; `explorer.reference_shape(slot)?`.
    pub fn reference_shape(&self, slot: ExplorerShapeSlot) -> Result<&Shape, Failure> {
        let ExplorerShapeSlot::Iterator(index) = slot else {
            return Ok(&self.shape);
        };
        let iterator = self
            .stack
            .get(index)
            .ok_or_else(|| invalid_shape_slot(index, self.stack.len()))?;
        Ok(iterator.reference_shape())
    }
}
fn invalid_shape_slot(index: usize, count: usize) -> Failure {
    let message =
        format!("TopExp_Explorer reference slot={index}; expected retained index below {count}");
    Failure::with_type("Standard_OutOfRange", &message, "")
}
