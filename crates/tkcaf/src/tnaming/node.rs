// Rust port of the file-local class TNaming_Node of
// ApplicationFramework/TKCAF/TNaming/TNaming_NamedShape.cxx, and of TNaming_RefShape.hxx,
// TNaming_RefShape.lxx (with its members in TNaming_NamedShape.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The evolution graph of TNaming (ocaf-design.md section 10): a `TNaming_Node` is one
//! (old shape, new shape) pair of a named shape; a `TNaming_RefShape` is one shape of the
//! `TNaming_UsedShapes` map with the first node that uses it.
use super::TNamingNamedShape;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkcdf::json_dump::{class_begin, field_dumped};
use rustycad_tkernel::standard::{
    Failure, Handle, SharedState, WeakHandle, dump::DumpStream, exceptions,
};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};
use std::io;
use std::sync::Arc;

/// A native `TNaming_Node*` or `TNaming_RefShape*`: `None` is `nullptr`.
pub type TNamingNodePtr = Option<Arc<TNamingNode>>;
/// A native `TNaming_RefShape*`.
pub type TNamingRefShapePtr = Option<Arc<TNamingRefShape>>;

/// The native pointer comparison `p == q` (two null pointers are equal);
/// `same_pointer(&None::<Arc<i32>>, &None)`.
pub(crate) fn same_pointer<T>(first: &Option<Arc<T>>, second: &Option<Arc<T>>) -> bool {
    match (first, second) {
        (Some(first), Some(second)) => Arc::ptr_eq(first, second),
        (None, None) => true,
        _ => false,
    }
}

/// The fields of `TNaming_Node`.
#[derive(Clone, Default)]
pub(crate) struct NodeLinks {
    pub(crate) old: TNamingRefShapePtr,
    pub(crate) new: TNamingRefShapePtr,
    pub(crate) att: WeakHandle<TNamingNamedShape>,
    pub(crate) next_same_attribute: TNamingNodePtr,
    pub(crate) next_same_old: TNamingNodePtr,
    pub(crate) next_same_new: TNamingNodePtr,
}

/// `TNaming_Node`: one evolution of a named shape, linked in three chains (the nodes of
/// the same attribute, the nodes that use the same old shape, the nodes that use the same
/// new shape). The native `delete` of a node is [`TNamingNode::release`].
pub struct TNamingNode {
    links: SharedState<NodeLinks>,
}

impl TNamingNode {
    /// `new TNaming_Node(Old, New)`.
    pub(crate) fn new(old: TNamingRefShapePtr, new: TNamingRefShapePtr) -> Arc<Self> {
        Arc::new(Self {
            links: SharedState::new(NodeLinks {
                old,
                new,
                ..NodeLinks::default()
            }),
        })
    }
    /// A copy of the node's fields.
    pub(crate) fn links(&self) -> NodeLinks {
        self.links.read().clone()
    }
    /// Changes the node's fields.
    pub(crate) fn edit(&self, change: impl FnOnce(&mut NodeLinks)) {
        change(&mut self.links.write());
    }
    /// `myOld`: the old shape of the evolution (null for a primitive).
    pub fn old(&self) -> TNamingRefShapePtr {
        self.links.read().old.clone()
    }
    /// `myNew`: the new shape of the evolution.
    pub fn new_shape(&self) -> TNamingRefShapePtr {
        self.links.read().new.clone()
    }
    /// `myAtt`: the named shape that holds the node (null once it is destroyed).
    pub fn attribute(&self) -> Handle<TNamingNamedShape> {
        self.links.read().att.upgrade()
    }
    /// `myAtt->...` where the native pointer must be valid: `Standard_NullObject`
    /// for a destroyed attribute (a dangling pointer natively).
    pub(crate) fn live_attribute(&self, who: &str) -> Result<Handle<TNamingNamedShape>, Failure> {
        let attribute = self.attribute();
        if attribute.is_null() {
            return Err(exceptions::null_object(&format!(
                "{who}: the node's named shape is destroyed; expected a node of a live TNaming_NamedShape"
            )));
        }
        Ok(attribute)
    }
    /// `nextSameAttribute`.
    pub fn next_same_attribute(&self) -> TNamingNodePtr {
        self.links.read().next_same_attribute.clone()
    }
    /// `NextSameShape(prs)`: the next node of the chain of `prs` (the new-shape chain when
    /// the node uses `prs` as neither shape, as natively).
    pub fn next_same_shape(&self, shape: &TNamingRefShapePtr) -> TNamingNodePtr {
        let links = self.links.read();
        if same_pointer(&links.old, shape) {
            return links.next_same_old.clone();
        }
        links.next_same_new.clone()
    }
    /// `Label`: the label of the node's named shape.
    pub fn label(&self) -> Result<TdfLabel, Failure> {
        Ok(self.live_attribute("TNaming_Node::Label")?.label())
    }
    /// `IsValidInTrans(Trans)`: the transaction is in the life span of the named shape.
    pub fn is_valid_in_trans(&self, transaction: i32) -> Result<bool, Failure> {
        let attribute = self.live_attribute("TNaming_Node::IsValidInTrans")?;
        let attribute = rustycad_tklcaf::live(&attribute, "TNaming_Node::IsValidInTrans")?;
        Ok(attribute.transaction() <= transaction
            && transaction <= attribute.until_transaction()?)
    }
    /// The native `delete` of a node: every link is dropped, so that no reference cycle
    /// outlives it.
    pub(crate) fn release(&self) {
        *self.links.write() = NodeLinks::default();
    }
    /// `DumpJson`: the six links. A negative depth recurses without end on the node graph
    /// (a node's shape points back to it); the native dump overflows the stack, the port
    /// returns an error instead.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let links = self.links();
        class_begin(stream, "TNaming_Node")?;
        dump_ref_shape(stream, depth, "myOld", &links.old)?;
        dump_ref_shape(stream, depth, "myNew", &links.new)?;
        let attribute = links.att.upgrade();
        if let Some(attribute) = attribute.get() {
            let child = |s: &mut DumpStream, d: i32| {
                rustycad_tklcaf::tdf::TdfAttribute::dump_json(attribute, s, d)
            };
            field_dumped(stream, depth, "myAtt", Some(&child))?;
        }
        dump_node(
            stream,
            depth,
            "nextSameAttribute",
            &links.next_same_attribute,
        )?;
        dump_node(stream, depth, "nextSameOld", &links.next_same_old)?;
        dump_node(stream, depth, "nextSameNew", &links.next_same_new)
    }
}

/// The refused unbounded dump of a graph link.
fn unbounded_dump(field: &str) -> io::Error {
    io::Error::other(format!(
        "TNaming DumpJson of {field} with a negative depth recurses without end on the node graph; expected a depth >= 0"
    ))
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED` of a node pointer.
fn dump_node(
    stream: &mut DumpStream,
    depth: i32,
    field: &str,
    node: &TNamingNodePtr,
) -> io::Result<()> {
    let Some(node) = node else { return Ok(()) };
    if depth < 0 {
        return Err(unbounded_dump(field));
    }
    let child = |s: &mut DumpStream, d: i32| node.dump_json(s, d);
    field_dumped(stream, depth, field, Some(&child))
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED` of a ref-shape pointer.
fn dump_ref_shape(
    stream: &mut DumpStream,
    depth: i32,
    field: &str,
    shape: &TNamingRefShapePtr,
) -> io::Result<()> {
    let Some(shape) = shape else { return Ok(()) };
    if depth < 0 {
        return Err(unbounded_dump(field));
    }
    let child = |s: &mut DumpStream, d: i32| shape.dump_json(s, d);
    field_dumped(stream, depth, field, Some(&child))
}

/// The fields of `TNaming_RefShape`.
#[derive(Clone, Default)]
struct RefShapeState {
    shape: Shape,
    first_use: TNamingNodePtr,
}

/// `TNaming_RefShape`: a shape of the `TNaming_UsedShapes` map and the first node that uses
/// it (the head of the chain `TNaming_Node::NextSameShape` walks).
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkcaf::tnaming::TNamingRefShape;
/// let reference = TNamingRefShape::new(&Shape::default());
/// assert!(reference.shape().is_null() && reference.first_use().is_none());
/// ```
#[derive(Default)]
pub struct TNamingRefShape {
    state: SharedState<RefShapeState>,
}

impl TNamingRefShape {
    /// `TNaming_RefShape(S)` (the default constructor is `Default`).
    pub fn new(shape: &Shape) -> Self {
        Self {
            state: SharedState::new(RefShapeState {
                shape: shape.clone(),
                first_use: None,
            }),
        }
    }
    /// `new TNaming_RefShape(S)`: the shared pointer the map and the nodes hold.
    pub(crate) fn shared(shape: &Shape) -> Arc<Self> {
        Arc::new(Self::new(shape))
    }
    /// `Shape(S)`.
    pub fn set_shape(&self, shape: &Shape) {
        self.state.write().shape = shape.clone();
    }
    /// `FirstUse(aPtr)`.
    pub fn set_first_use(&self, node: &TNamingNodePtr) {
        self.state.write().first_use = node.clone();
    }
    /// `FirstUse()`.
    pub fn first_use(&self) -> TNamingNodePtr {
        self.state.read().first_use.clone()
    }
    /// `Shape()`.
    pub fn shape(&self) -> Shape {
        self.state.read().shape.clone()
    }
    /// `Label`: the label of the first use (`Standard_NullObject` without one, a null
    /// dereference natively).
    pub fn label(&self) -> Result<TdfLabel, Failure> {
        self.first_node("TNaming_RefShape::Label")?.label()
    }
    /// `NamedShape`: the named shape of the first use.
    pub fn named_shape(&self) -> Result<Handle<TNamingNamedShape>, Failure> {
        Ok(self.first_node("TNaming_RefShape::NamedShape")?.attribute())
    }
    fn first_node(&self, who: &str) -> Result<Arc<TNamingNode>, Failure> {
        self.first_use().ok_or_else(|| {
            exceptions::null_object(&format!(
                "{who}: the shape has no first use; expected a shape used by a node"
            ))
        })
    }
    /// The native `delete` of a ref shape: the first-use link is dropped.
    pub(crate) fn release(&self) {
        self.state.write().first_use = None;
    }
    /// `DumpJson`: the native class name is `TNaming_NamedShape`; then the shape and the
    /// first use (see [`TNamingNode::dump_json`] for a negative depth).
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state = self.state.read().clone();
        class_begin(stream, "TNaming_NamedShape")?;
        let shape = |s: &mut DumpStream, d: i32| state.shape.dump_json(s, d);
        field_dumped(stream, depth, "&myShape", Some(&shape))?;
        dump_node(stream, depth, "myFirstUse", &state.first_use)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_equality_matches_the_native_compare() {
        let shape = TNamingRefShape::shared(&Shape::default());
        assert!(same_pointer::<TNamingRefShape>(&None, &None));
        assert!(same_pointer(&Some(shape.clone()), &Some(shape.clone())));
        assert!(!same_pointer(&Some(shape), &None));
    }

    #[test]
    fn next_same_shape_picks_the_chain_of_the_shape() {
        let old = Some(TNamingRefShape::shared(&Shape::default()));
        let new = Some(TNamingRefShape::shared(&Shape::default()));
        let node = TNamingNode::new(old.clone(), new.clone());
        let after_old = TNamingNode::new(None, None);
        let after_new = TNamingNode::new(None, None);
        node.edit(|links| {
            links.next_same_old = Some(after_old.clone());
            links.next_same_new = Some(after_new.clone());
        });
        assert!(Arc::ptr_eq(
            &node.next_same_shape(&old).unwrap(),
            &after_old
        ));
        assert!(Arc::ptr_eq(
            &node.next_same_shape(&new).unwrap(),
            &after_new
        ));
        // Neither shape: the native code answers the new-shape chain.
        assert!(Arc::ptr_eq(
            &node.next_same_shape(&None).unwrap(),
            &after_new
        ));
        node.release();
        assert!(node.next_same_attribute().is_none() && node.old().is_none());
    }

    #[test]
    fn a_ref_shape_without_use_has_no_label() {
        let reference = TNamingRefShape::new(&Shape::default());
        let error = reference.label().unwrap_err();
        assert_eq!(error.exception_type(), "Standard_NullObject");
        assert!(reference.named_shape().is_err());
    }

    #[test]
    fn negative_depth_dumps_of_the_graph_are_refused() {
        let reference = TNamingRefShape::shared(&Shape::default());
        let node = TNamingNode::new(None, Some(reference.clone()));
        reference.set_first_use(&Some(node.clone()));
        let mut stream = DumpStream::default();
        assert!(node.dump_json(&mut stream, -1).is_err());
        let mut bounded = DumpStream::default();
        node.dump_json(&mut bounded, 2).unwrap();
        let text = String::from_utf8_lossy(bounded.bytes()).to_string();
        assert!(text.contains("\"className\": \"TNaming_Node\""), "{text}");
        assert!(text.contains("\"New\""), "{text}");
        node.release();
        reference.release();
    }
}
