//! `BRepAlgoAPI_Algo`: the shape and option root shared by BRep Boolean algorithms.
use crate::bop_algo::Options;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::Report;
use rustycad_tkernel::ncollection::BaseAllocator;
use rustycad_tkernel::standard::{Failure, RuntimeType};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
use std::io::{self, Write};
use std::sync::Arc;

/// The base state used by API Boolean algorithms; `BRepAlgoAPIAlgo::new()`.
pub struct BRepAlgoAPIAlgo {
    pub(crate) base: BRepBuilderAPIMakeShapeBase,
    pub(crate) options: Options,
}

impl BRepAlgoAPIAlgo {
    /// The native protected empty constructor with the common allocator.
    /// Example: `let algo = BRepAlgoAPIAlgo::new(); assert!(algo.shape().is_null());`.
    pub fn new() -> Self {
        Self::with_allocator(Some(BaseAllocator::common()))
    }

    /// Creates the root with the supplied allocator; `BRepAlgoAPIAlgo::with_allocator(None)`.
    pub(crate) fn with_allocator(allocator: Option<BaseAllocator>) -> Self {
        Self::with_options(Options::with_allocator(allocator))
    }

    fn with_options(options: Options) -> Self {
        Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            options,
        }
    }

    /// Returns OCCT `Shape()` directly, without checking done state or invoking `Build`.
    /// Example: `let shape = algo.shape(); assert!(shape.is_null());`.
    pub fn shape(&self) -> &Shape {
        &self.base.my_shape
    }

    /// Clears error and warning alerts without changing configured options; `algo.clear()`.
    pub fn clear(&self) {
        self.options.clear();
    }

    /// Removes warning alerts; `algo.clear_warnings()`.
    pub fn clear_warnings(&self) {
        self.options.clear_warnings();
    }

    /// Whether any error alerts are recorded; `algo.has_errors()`.
    pub fn has_errors(&self) -> bool {
        self.options.has_errors()
    }

    /// Whether an error alert of `kind` is recorded; `algo.has_error(kind)`.
    pub fn has_error(&self, kind: &RuntimeType) -> bool {
        self.options.has_error(kind)
    }

    /// Whether any warning alerts are recorded; `algo.has_warnings()`.
    pub fn has_warnings(&self) -> bool {
        self.options.has_warnings()
    }

    /// Whether a warning alert of `kind` is recorded; `algo.has_warning(kind)`.
    pub fn has_warning(&self, kind: &RuntimeType) -> bool {
        self.options.has_warning(kind)
    }

    /// Returns the alerts collected by this algorithm; `algo.get_report()`.
    pub fn get_report(&self) -> &Arc<Report> {
        self.options.report()
    }

    /// Writes error message keys; `algo.dump_errors(&mut Vec::new())?`.
    pub fn dump_errors(&self, output: &mut dyn Write) -> io::Result<()> {
        self.options.dump_errors(output)
    }

    /// Writes warning message keys; `algo.dump_warnings(&mut Vec::new())?`.
    pub fn dump_warnings(&self, output: &mut dyn Write) -> io::Result<()> {
        self.options.dump_warnings(output)
    }

    /// Returns the run-parallel flag; `algo.run_parallel()`.
    pub fn run_parallel(&self) -> bool {
        self.options.run_parallel()
    }

    /// Sets the run-parallel flag; `algo.set_run_parallel(false)`.
    pub fn set_run_parallel(&mut self, enabled: bool) {
        self.options.set_run_parallel(enabled);
    }

    /// Returns the extra fuzzy tolerance; `algo.fuzzy_value()`.
    pub fn fuzzy_value(&self) -> f64 {
        self.options.fuzzy_value()
    }

    /// Sets the extra fuzzy tolerance; `algo.set_fuzzy_value(0.01)`.
    pub fn set_fuzzy_value(&mut self, tolerance: f64) {
        self.options.set_fuzzy_value(tolerance);
    }

    /// Enables or disables OBB filtering; `algo.set_use_obb(true)`.
    pub fn set_use_obb(&mut self, enabled: bool) {
        self.options.set_use_obb(enabled);
    }
}

impl Default for BRepAlgoAPIAlgo {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepBuilderAPIMakeShape for BRepAlgoAPIAlgo {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    fn shape(&mut self) -> Result<&Shape, Failure> {
        Ok(&self.base.my_shape)
    }
}

#[cfg(test)]
mod tests {
    use super::BRepAlgoAPIAlgo;
    use rustycad_tkernel::ncollection::BaseAllocator;

    #[test]
    fn common_allocator_constructor_uses_shared_allocator() {
        let algorithm = BRepAlgoAPIAlgo::new();
        assert!(algorithm.options.allocator() == Some(&BaseAllocator::common()));
    }

    #[test]
    fn obb_setter_updates_derived_algorithm_option() {
        let mut algorithm = BRepAlgoAPIAlgo::new();
        algorithm.set_use_obb(true);
        assert!(algorithm.options.use_obb());
    }
}
