// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_LoopEnum.hxx,
// TopOpeBRepBuild_BlockIterator.hxx/.lxx/.cxx, TopOpeBRepBuild_Loop.hxx/.cxx,
// TopOpeBRepBuild_LoopSet.hxx/.cxx and TopOpeBRepBuild_LoopClassifier.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The loops of the area builders: a shape (an old wire, shell, ...) or a block of
//! elements of a shape set, the sets of loops, and their classifier.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, Transient};
use rustycad_tkernel::{impl_transient, impl_transient_interface};
use rustycad_tkg3d::top_abs::State;

/// The kind of the loops an area builder iterates (`TopOpeBRepBuild_LoopEnum`);
/// `TopOpeBRepBuildLoopEnum::Block as i32 == 2`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum TopOpeBRepBuildLoopEnum {
    /// `TopOpeBRepBuild_ANYLOOP`.
    #[default]
    AnyLoop,
    /// `TopOpeBRepBuild_BOUNDARY`: shape loops.
    Boundary,
    /// `TopOpeBRepBuild_BLOCK`: block loops.
    Block,
}

/// An iterator over the indices `lower..=upper` of a block
/// (`TopOpeBRepBuild_BlockIterator`); `let mut bi = TopOpeBRepBuildBlockIterator::new(1, 3);`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TopOpeBRepBuildBlockIterator {
    my_lower: i32,
    my_upper: i32,
    my_value: i32,
}

impl Default for TopOpeBRepBuildBlockIterator {
    /// `TopOpeBRepBuild_BlockIterator()`: lower 0, upper 0, value 1.
    fn default() -> Self {
        Self {
            my_lower: 0,
            my_upper: 0,
            my_value: 1,
        }
    }
}

impl TopOpeBRepBuildBlockIterator {
    /// `TopOpeBRepBuild_BlockIterator(Lower, Upper)`; `TopOpeBRepBuildBlockIterator::new(1, 3)`.
    pub fn new(lower: i32, upper: i32) -> Self {
        Self {
            my_lower: lower,
            my_upper: upper,
            my_value: lower,
        }
    }

    /// `Initialize()`: at the lower index; `bi.initialize()`.
    pub fn initialize(&mut self) {
        self.my_value = self.my_lower;
    }

    /// `More()`; `bi.more()`.
    pub fn more(&self) -> bool {
        self.my_value <= self.my_upper
    }

    /// `Next()`; `bi.next()`.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol.
    pub fn next(&mut self) {
        self.my_value += 1;
    }

    /// `Value()`; `bi.value()`.
    pub fn value(&self) -> i32 {
        self.my_value
    }

    /// `Extent()`: the index count, 0 for a lower index 0; `bi.extent()`.
    pub fn extent(&self) -> i32 {
        if self.my_lower != 0 {
            self.my_upper - self.my_lower + 1
        } else {
            0
        }
    }
}

/// A loop of an area builder (`TopOpeBRepBuild_Loop`, virtual members overridden by
/// `TopOpeBRepBuild_Pave`), reached through `Handle<dyn TopOpeBRepBuildLoop>`;
/// `let shape = l.shape();`.
pub trait TopOpeBRepBuildLoop: Transient {
    /// `IsShape()`: a shape loop (else a block loop); `l.is_shape()`.
    fn is_shape(&self) -> bool;

    /// `Shape()`: the shape of a shape loop; `l.shape()`.
    fn shape(&self) -> Shape;

    /// `BlockIterator()`: the block of a block loop (0..=0 for a shape loop);
    /// `l.block_iterator()`.
    fn block_iterator(&self) -> TopOpeBRepBuildBlockIterator;

    /// `Dump()`: nothing, as natively; `l.dump()`.
    fn dump(&self) {}
}
impl_transient_interface!(
    dyn TopOpeBRepBuildLoop,
    "TopOpeBRepBuild_Loop",
    StandardTransient
);

/// The loop of a shape or of a block (the concrete `TopOpeBRepBuild_Loop`);
/// `TopOpeBRepBuildLoopBase::from_shape(&wire)`.
pub struct TopOpeBRepBuildLoopBase {
    parent: StandardTransient,
    /// `myIsShape` (protected).
    pub(crate) my_is_shape: bool,
    /// `myShape` (protected).
    pub(crate) my_shape: Shape,
    /// `myBlockIterator` (protected).
    pub(crate) my_block_iterator: TopOpeBRepBuildBlockIterator,
}
impl_transient!(TopOpeBRepBuildLoopBase, "TopOpeBRepBuild_Loop", StandardTransient, parent, interfaces: [dyn TopOpeBRepBuildLoop]);

impl TopOpeBRepBuildLoopBase {
    /// `TopOpeBRepBuild_Loop(S)`: a shape loop; `TopOpeBRepBuildLoopBase::from_shape(&wire)`.
    pub fn from_shape(s: &Shape) -> Self {
        Self {
            parent: StandardTransient::default(),
            my_is_shape: true,
            my_shape: s.clone(),
            my_block_iterator: TopOpeBRepBuildBlockIterator::new(0, 0),
        }
    }

    /// `TopOpeBRepBuild_Loop(BI)`: a block loop; `TopOpeBRepBuildLoopBase::from_block(&bi)`.
    pub fn from_block(bi: &TopOpeBRepBuildBlockIterator) -> Self {
        Self {
            parent: StandardTransient::default(),
            my_is_shape: false,
            my_shape: Shape::default(),
            my_block_iterator: *bi,
        }
    }
}

impl TopOpeBRepBuildLoop for TopOpeBRepBuildLoopBase {
    fn is_shape(&self) -> bool {
        self.my_is_shape
    }

    fn shape(&self) -> Shape {
        self.my_shape.clone()
    }

    fn block_iterator(&self) -> TopOpeBRepBuildBlockIterator {
        self.my_block_iterator
    }
}

/// The data of a loop set: the loops and the iteration over them.
#[derive(Clone, Default)]
pub struct TopOpeBRepBuildLoopSetBase {
    my_list_of_loop: List<Handle<dyn TopOpeBRepBuildLoop>>,
    my_items: Vec<Handle<dyn TopOpeBRepBuildLoop>>,
    my_position: usize,
    my_loop_index: i32,
    my_nb_loop: i32,
}

/// A set of loops (`TopOpeBRepBuild_LoopSet`, virtual members overridden by
/// `TopOpeBRepBuild_PaveSet`); `ls.init_loop(); while ls.more_loop() { ls.next_loop(); }`.
pub trait TopOpeBRepBuildLoopSet {
    /// The loops and their iteration.
    fn loop_set_base(&self) -> &TopOpeBRepBuildLoopSetBase;

    /// The loops and their iteration, changed.
    fn loop_set_base_mut(&mut self) -> &mut TopOpeBRepBuildLoopSetBase;

    /// `ChangeListOfLoop()`; `ls.change_list_of_loop().append(l)`.
    fn change_list_of_loop(&mut self) -> &mut List<Handle<dyn TopOpeBRepBuildLoop>> {
        &mut self.loop_set_base_mut().my_list_of_loop
    }

    /// `InitLoop()`: at the first loop (the iteration walks a copy of the list);
    /// `ls.init_loop()?`.
    fn init_loop(&mut self) -> Result<(), Failure> {
        let base = self.loop_set_base_mut();
        base.my_items = base.my_list_of_loop.iter().cloned().collect();
        (base.my_position, base.my_loop_index, base.my_nb_loop) =
            (0, 1, base.my_items.len() as i32);
        Ok(())
    }

    /// `MoreLoop()`; `ls.more_loop()`.
    fn more_loop(&self) -> bool {
        let base = self.loop_set_base();
        base.my_position < base.my_items.len()
    }

    /// `NextLoop()`; `ls.next_loop()?`.
    fn next_loop(&mut self) -> Result<(), Failure> {
        let base = self.loop_set_base_mut();
        base.my_loop_index += 1;
        base.my_position += 1;
        Ok(())
    }

    /// `Loop()`: the current loop; Standard_NoSuchObject past the end; `ls.loop_()?`.
    fn loop_(&self) -> Result<Handle<dyn TopOpeBRepBuildLoop>, Failure> {
        let base = self.loop_set_base();
        base.my_items.get(base.my_position).cloned().ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                "TopOpeBRepBuild_LoopSet::Loop: no current loop; expected MoreLoop()",
                "TopOpeBRepBuild_LoopSet",
            )
        })
    }
}

/// The plain set of loops (the concrete `TopOpeBRepBuild_LoopSet`);
/// `let ls = TopOpeBRepBuildPlainLoopSet::default();`.
#[derive(Clone, Default)]
pub struct TopOpeBRepBuildPlainLoopSet {
    base: TopOpeBRepBuildLoopSetBase,
}

impl TopOpeBRepBuildPlainLoopSet {
    /// `TopOpeBRepBuild_LoopSet()`: no loop; `TopOpeBRepBuildPlainLoopSet::new()`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl TopOpeBRepBuildLoopSet for TopOpeBRepBuildPlainLoopSet {
    fn loop_set_base(&self) -> &TopOpeBRepBuildLoopSetBase {
        &self.base
    }

    fn loop_set_base_mut(&mut self) -> &mut TopOpeBRepBuildLoopSetBase {
        &mut self.base
    }
}

/// The classifier of a loop relative to another (`TopOpeBRepBuild_LoopClassifier`);
/// `let state = classifier.compare(&l1, &l2)?;`.
pub trait TopOpeBRepBuildLoopClassifier {
    /// `Compare(L1, L2)`: the state of `l1` relative to `l2`.
    fn compare(
        &mut self,
        l1: &Handle<dyn TopOpeBRepBuildLoop>,
        l2: &Handle<dyn TopOpeBRepBuildLoop>,
    ) -> Result<State, Failure>;
}

/// A shape with a list of shapes (`TopOpeBRepBuild_ShapeListOfShape`);
/// `TopOpeBRepBuildShapeListOfShape::with_shape(&face)`.
#[derive(Clone, Default)]
pub struct TopOpeBRepBuildShapeListOfShape {
    my_list: List<Shape>,
    my_shape: Shape,
}

impl TopOpeBRepBuildShapeListOfShape {
    /// `TopOpeBRepBuild_ShapeListOfShape()`; `TopOpeBRepBuildShapeListOfShape::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepBuild_ShapeListOfShape(S)`; `TopOpeBRepBuildShapeListOfShape::with_shape(&s)`.
    pub fn with_shape(s: &Shape) -> Self {
        Self {
            my_list: List::default(),
            my_shape: s.clone(),
        }
    }

    /// `TopOpeBRepBuild_ShapeListOfShape(S, L)`; `TopOpeBRepBuildShapeListOfShape::with_list(&s, &l)`.
    pub fn with_list(s: &Shape, l: &List<Shape>) -> Self {
        Self {
            my_list: l.clone(),
            my_shape: s.clone(),
        }
    }

    /// `List()`; `sl.list()`.
    pub fn list(&self) -> &List<Shape> {
        &self.my_list
    }

    /// `ChangeList()`; `sl.change_list().append(s)`.
    pub fn change_list(&mut self) -> &mut List<Shape> {
        &mut self.my_list
    }

    /// `Shape()`; `sl.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.my_shape
    }

    /// `ChangeShape()`; `*sl.change_shape() = s`.
    pub fn change_shape(&mut self) -> &mut Shape {
        &mut self.my_shape
    }
}
