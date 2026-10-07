//! Object-safe adaptation of OCCT 3d097a's inferred private IteratorInterface.
use std::any::Any;

/// Polymorphic iterator customization; `cursor.increment(); let copy = cursor.clone_box();`.
/// The data lifetime outlives every wrapper; cloning never borrows the source wrapper.
/// Payloads need not implement Clone, Any or Send. The parallel loop APIs retain their own bounds.
///
/// An owned clone cannot escape borrowed data:
/// ```compile_fail,E0515
/// use rustycad_tkernel::osd::ParallelIteratorInterface;
/// struct Borrowed<'a>(&'a str);
/// impl<'a> ParallelIteratorInterface<'a> for Borrowed<'a> {
///     fn is_equal(&self, _: &dyn ParallelIteratorInterface<'a>) -> bool { false }
///     fn increment(&mut self) {}
///     fn clone_box(&self) -> Option<Box<dyn ParallelIteratorInterface<'a> + 'a>> {
///         Some(Box::new(Borrowed(self.0)))
///     }
/// }
/// fn escapes() -> Option<Box<dyn ParallelIteratorInterface<'static>>> {
///     let text = String::from("local");
///     let source = Borrowed(&text);
///     source.clone_box()
/// }
/// ```
pub trait ParallelIteratorInterface<'data>: 'data {
    /// Compares wrapped positions through virtual dispatch; `cursor.is_equal(other.as_ref())`.
    fn is_equal(&self, other: &dyn ParallelIteratorInterface<'data>) -> bool;

    /// Advances the wrapped iterator; `cursor.increment()`.
    fn increment(&mut self);

    /// Owns an independent wrapper or native null over the same borrowed data; `let copy = cursor.clone_box()`.
    fn clone_box(&self) -> Option<Box<dyn ParallelIteratorInterface<'data> + 'data>>;

    /// Exposes optional comparison state (Rust extension); `other.comparison_value()?.downcast_ref::<usize>()`.
    /// A borrowed iterator may expose an owned position field without requiring its payload to be Any.
    /// This seam does not reproduce arbitrary C++ RTTI; equality remains the implementer's contract.
    fn comparison_value(&self) -> Option<&dyn Any> {
        None
    }
}

/// Runs virtual destruction and releases an owned wrapper; `destroy_parallel_iterator(cursor)`.
/// This covers the native owned delete path, not an explicit destructor on externally owned storage.
pub fn destroy_parallel_iterator<'data>(
    iterator: Box<dyn ParallelIteratorInterface<'data> + 'data>,
) {
    drop(iterator);
}
