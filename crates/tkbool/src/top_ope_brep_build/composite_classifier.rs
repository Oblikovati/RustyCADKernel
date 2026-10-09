// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_CompositeClassifier.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepBuild_CompositeClassifier`: the classification of composite loops (a
//! shape or a block of elements of a block builder).
use super::block_builder::TopOpeBRepBuildBlockBuilder;
use super::loops::{TopOpeBRepBuildBlockIterator, TopOpeBRepBuildLoop};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::State;

/// The classifier of composite loops (`TopOpeBRepBuild_CompositeClassifier`, whose
/// `Compare` is [`Self::composite_compare`]); the block builder is borrowed where
/// natively a pointer is kept; `let state = wec.composite_compare(&l1, &l2)?;`.
pub trait TopOpeBRepBuildCompositeClassifier {
    /// `myBlockBuilder` (protected): the blocks of the loops.
    fn block_builder(&self) -> &TopOpeBRepBuildBlockBuilder;

    /// `CompareShapes(B1, B2)`: the state of shape `b1` relative to shape `b2`.
    fn compare_shapes(&mut self, b1: &Shape, b2: &Shape) -> Result<State, Failure>;

    /// `CompareElementToShape(E, B)`: the state of element `e` relative to shape `b`.
    fn compare_element_to_shape(&mut self, e: &Shape, b: &Shape) -> Result<State, Failure>;

    /// `ResetShape(B)`: prepares a classification involving shape `b`.
    fn reset_shape(&mut self, b: &Shape) -> Result<(), Failure>;

    /// `ResetElement(E)`: prepares a classification involving element `e`.
    fn reset_element(&mut self, e: &Shape) -> Result<(), Failure>;

    /// `CompareElement(E)`: adds `e` to the classification; false when already added.
    fn compare_element(&mut self, e: &Shape) -> Result<bool, Failure>;

    /// `State()`: the state of the current classification.
    fn state(&mut self) -> State;

    /// The element of the block builder at the block iterator (`MYBB->Element(Bit)`).
    fn block_element(&self, bi: &TopOpeBRepBuildBlockIterator) -> Result<Shape, Failure> {
        self.block_builder().element_at(bi).cloned()
    }

    /// `TopOpeBRepBuild_CompositeClassifier::Compare(L1, L2)`: dispatches on the shape
    /// or block nature of both loops; `classifier.composite_compare(&l1, &l2)?`.
    fn composite_compare(
        &mut self,
        l1: &Handle<dyn TopOpeBRepBuildLoop>,
        l2: &Handle<dyn TopOpeBRepBuildLoop>,
    ) -> Result<State, Failure> {
        match (l1.is_shape(), l2.is_shape()) {
            (true, true) => self.compare_shapes(&l1.shape(), &l2.shape()),
            (false, true) => self.compare_block_to_shape(l1, &l2.shape()),
            (true, false) => self.compare_shape_to_block(&l1.shape(), l2),
            (false, false) => self.compare_block_to_block(l1, l2),
        }
    }

    /// The block `l1` against shape `s2`: the first element of `l1` whose state is
    /// not UNKNOWN.
    fn compare_block_to_shape(
        &mut self,
        l1: &Handle<dyn TopOpeBRepBuildLoop>,
        s2: &Shape,
    ) -> Result<State, Failure> {
        let mut state = State::Unknown;
        let mut bit1 = l1.block_iterator();
        bit1.initialize();
        while bit1.more() {
            let s1 = self.block_element(&bit1)?;
            state = self.compare_element_to_shape(&s1, s2)?;
            if state != State::Unknown {
                break;
            }
            bit1.next();
        }
        Ok(state)
    }

    /// Shape `s1` against the block `l2`, stopping at an element already added.
    fn compare_shape_to_block(
        &mut self,
        s1: &Shape,
        l2: &Handle<dyn TopOpeBRepBuildLoop>,
    ) -> Result<State, Failure> {
        self.reset_shape(s1)?;
        let mut bit2 = l2.block_iterator();
        bit2.initialize();
        while bit2.more() {
            let s2 = self.block_element(&bit2)?;
            if !self.compare_element(&s2)? {
                break;
            }
            bit2.next();
        }
        Ok(self.state())
    }

    /// The block `l1` against the block `l2`: the first element of `l1` whose state
    /// against all the elements of `l2` is not UNKNOWN.
    fn compare_block_to_block(
        &mut self,
        l1: &Handle<dyn TopOpeBRepBuildLoop>,
        l2: &Handle<dyn TopOpeBRepBuildLoop>,
    ) -> Result<State, Failure> {
        let mut state = State::Unknown;
        let mut bit1 = l1.block_iterator();
        bit1.initialize();
        while bit1.more() {
            let s1 = self.block_element(&bit1)?;
            self.reset_element(&s1)?;
            self.compare_all_elements(l2)?;
            state = self.state();
            if state != State::Unknown {
                break;
            }
            bit1.next();
        }
        Ok(state)
    }

    /// Every element of the block `l2` compared, whatever CompareElement returns.
    fn compare_all_elements(
        &mut self,
        l2: &Handle<dyn TopOpeBRepBuildLoop>,
    ) -> Result<(), Failure> {
        let mut bit2 = l2.block_iterator();
        bit2.initialize();
        while bit2.more() {
            let s2 = self.block_element(&bit2)?;
            self.compare_element(&s2)?;
            bit2.next();
        }
        Ok(())
    }
}
