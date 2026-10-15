// Rust translation of BRepOffsetAPI_MakeOffsetShape::PerformBySimple at OCCT 3d097a0328e7.

//! The simple-algorithm entry point of `BRepOffsetAPI_MakeOffsetShape`.
use crate::brep_offset::BRepOffsetMakeSimpleOffset;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};

#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum LastOffsetAlgorithm {
    #[default]
    None,
    Simple,
}

/// Runs the simple topology-preserving offset algorithm through the builder API.
/// This port covers `PerformBySimple`; `PerformByJoin` and standard offset operations
/// remain unimplemented pending the intersection-based offset algorithm.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIMakeOffsetShape;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let mut maker = BRepOffsetAPIMakeOffsetShape::new();
/// maker.perform_by_simple(&Shape::default(), 1.0)?;
/// assert!(!maker.is_done());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepOffsetAPIMakeOffsetShape {
    base: BRepBuilderAPIMakeShapeBase,
    simple_offset: BRepOffsetMakeSimpleOffset,
    last_algorithm: LastOffsetAlgorithm,
}

impl Default for BRepOffsetAPIMakeOffsetShape {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepBuilderAPIMakeShape for BRepOffsetAPIMakeOffsetShape {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    fn build(&mut self, _range: &ProgressRange<'_>) -> Result<(), Failure> {
        Ok(())
    }

    fn generated(&mut self, shape: &Shape) -> Result<&List<Shape>, Failure> {
        self.base.my_generated.clear();
        if self.last_algorithm == LastOffsetAlgorithm::Simple {
            let image = self.simple_offset.generated(shape)?;
            self.append_nontrivial_image(image, shape);
        }
        Ok(&self.base.my_generated)
    }

    fn modified(&mut self, shape: &Shape) -> Result<&List<Shape>, Failure> {
        self.base.my_generated.clear();
        if self.last_algorithm == LastOffsetAlgorithm::Simple {
            let image = self.simple_offset.modified(shape)?;
            self.append_nontrivial_image(image, shape);
        }
        Ok(&self.base.my_generated)
    }

    fn is_deleted(&mut self, _shape: &Shape) -> Result<bool, Failure> {
        Ok(false)
    }
}

impl BRepOffsetAPIMakeOffsetShape {
    /// `BRepOffsetAPI_MakeOffsetShape()`: no algorithm run and a null result;
    /// `BRepOffsetAPIMakeOffsetShape::new()`.
    pub fn new() -> Self {
        Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            simple_offset: BRepOffsetMakeSimpleOffset::new(),
            last_algorithm: LastOffsetAlgorithm::None,
        }
    }

    /// `PerformBySimple(S, offset)`: initialize and run the simple algorithm, then store
    /// its result only when done; `maker.perform_by_simple(&shape, 10.0)?`.
    pub fn perform_by_simple(&mut self, shape: &Shape, offset: f64) -> Result<(), Failure> {
        self.base.set_not_done();
        self.last_algorithm = LastOffsetAlgorithm::Simple;
        self.simple_offset.initialize(shape, offset);
        self.simple_offset.perform()?;
        self.complete_simple_offset();
        Ok(())
    }

    fn complete_simple_offset(&mut self) {
        if !self.simple_offset.is_done() {
            return;
        }
        self.base.my_shape = self.simple_offset.result_shape().clone();
        self.base.set_done();
    }

    fn append_nontrivial_image(&mut self, image: Shape, source: &Shape) {
        if image.is_null() || image.is_same(source) {
            return;
        }
        self.base.my_generated.append(image);
    }
}
