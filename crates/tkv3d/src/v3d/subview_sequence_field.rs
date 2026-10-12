//! Native Subviews names the actual sequence field, not a cloned vector or cached extent.
use super::{V3dSubviewCursor, V3dSubviewNodeRef, V3dView, V3dViewSource};
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, Handle},
};
type SubviewSequence = Sequence<Handle<dyn V3dViewSource>>;
/// Retained const sequence-field alias; `view.subviews_ref().cursor()`.
#[derive(Clone, Copy)]
pub struct V3dSubviewSequenceRef<'a> {
    parent: &'a V3dView,
}
impl<'a> V3dSubviewSequenceRef<'a> {
    pub(super) fn new(parent: &'a V3dView) -> Self {
        Self { parent }
    }
    /// Observe the actual current length; `reference.len()`.
    pub fn len(&self) -> usize {
        self.parent.state.read().subviews.len()
    }
    /// Observe the empty sequence; `reference.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Read the native one-based lower bound; `reference.lower()`.
    pub fn lower(&self) -> usize {
        self.parent.state.read().subviews.lower()
    }
    /// Read the live upper bound; `reference.upper()`.
    pub fn upper(&self) -> usize {
        self.parent.state.read().subviews.upper()
    }
    /// Identify the actual retained sequence object; `reference.reference_address()`.
    pub fn reference_address(&self) -> *const SubviewSequence {
        let fields = self.parent.state.read();
        std::ptr::from_ref(&fields.subviews)
    }
    /// Start native live-node traversal with no owning view copies; `reference.cursor()`.
    pub fn cursor(&self) -> V3dSubviewCursor<'a> {
        V3dSubviewCursor::new(self.parent)
    }
    /// Borrow an actual one-based node value; `reference.value_ref(1)?`.
    pub fn value_ref(&self, index: i32) -> Result<V3dSubviewNodeRef<'a>, Failure> {
        let fields = self.parent.state.read();
        fields.subviews.value_signed(index)?;
        let mut position = fields.subviews.cursor();
        for _ in 1..index {
            position.next(&fields.subviews);
        }
        Ok(V3dSubviewNodeRef::new(self.parent, position))
    }
    /// Borrow the live first node; `reference.first_ref()?`.
    pub fn first_ref(&self) -> Result<V3dSubviewNodeRef<'a>, Failure> {
        self.endpoint_ref(true)
    }
    /// Borrow the live last node; `reference.last_ref()?`.
    pub fn last_ref(&self) -> Result<V3dSubviewNodeRef<'a>, Failure> {
        self.endpoint_ref(false)
    }
    fn endpoint_ref(&self, first: bool) -> Result<V3dSubviewNodeRef<'a>, Failure> {
        let length = self.len();
        if length == 0 {
            return Err(Failure::with_type(
                "Standard_NoSuchObject",
                &format!(
                    "Subview sequence endpoint-first={first} length={length}; expected nonempty actual sequence"
                ),
                "",
            ));
        }
        self.value_ref(if first { 1 } else { length as i32 })
    }
    /// Explicitly copy the sequence and its view owners; native parity uses this live reference.
    /// Example: `let copied=reference.snapshot()`.
    pub fn snapshot(&self) -> SubviewSequence {
        self.parent.state.read().subviews.clone()
    }
}
impl V3dView {
    /// Borrow the actual native Subviews sequence; `let alias=view.subviews_ref()`.
    pub fn subviews_ref(&self) -> V3dSubviewSequenceRef<'_> {
        V3dSubviewSequenceRef::new(self)
    }
    /// Iterate actual stable subview nodes; `let mut cursor=view.subview_iterator(); cursor.next()`.
    pub fn subview_iterator(&self) -> V3dSubviewCursor<'_> {
        self.subviews_ref().cursor()
    }
}
