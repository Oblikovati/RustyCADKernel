// Rust port of TDF_Label::FindOrAddChild (ApplicationFramework/TKLCAF/TDF/TDF_Label.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The child search and creation of `TDF_Label`.
use super::{TdfLabel, label_node::TdfLabelNode};
use std::sync::Arc;

/// Private `FindOrAddChild`: the child of that tag, created in increasing tag order when
/// `create` is set. The search starts at the last found child when its tag is lower (jfa
/// 10.01.2003), and the node remembers the brother before the place found (agv 14.07.2010).
pub(crate) fn find_or_add_child(
    node: &Arc<TdfLabelNode>,
    tag: i32,
    create: bool,
) -> Option<Arc<TdfLabelNode>> {
    let (mut last, mut current) = (None, node.first_child());
    if let Some(found) = node.state.read().last_found_child.upgrade() {
        if found.tag() == tag {
            return Some(found);
        }
        if found.tag() < tag {
            current = found.brother();
            last = Some(found);
        }
    }
    while let Some(candidate) = current.clone().filter(|candidate| candidate.tag() < tag) {
        current = candidate.brother();
        last = Some(candidate);
    }
    let child = match current.filter(|candidate| candidate.tag() == tag) {
        Some(existing) => Some(existing),
        None if create => Some(insert_child(node, tag, last.as_ref())),
        None => None,
    };
    if let Some(last) = last {
        node.state.write().last_found_child = Arc::downgrade(&last);
    }
    child
}

/// Creates the child before the brother that follows `last` (or at the head), imported
/// like its father, and registers it for access by entry.
fn insert_child(
    node: &Arc<TdfLabelNode>,
    tag: i32,
    last: Option<&Arc<TdfLabelNode>>,
) -> Arc<TdfLabelNode> {
    let child = TdfLabelNode::new_child(tag, node);
    child.set_imported(node.is_imported());
    match last {
        None => {
            let mut state = node.state.write();
            child.state.write().brother = state.first_child.take();
            state.first_child = Some(child.clone());
        }
        Some(last) => {
            let mut state = last.state.write();
            child.state.write().brother = state.brother.take();
            state.brother = Some(child.clone());
        }
    }
    if let Some(core) = node.core().filter(|core| core.is_access_by_entries()) {
        core.register_label(&TdfLabel::from_node(Some(child.clone())));
    }
    child
}
