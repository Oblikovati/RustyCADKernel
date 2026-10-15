//! The graph-node links the GD&T and view tools repeat natively: the node of a GUID on a
//! label (found, else a new default node), the unlinking of all the fathers of a node, and
//! the linking of a list of father labels to a child node.
use super::XCAFDocGraphNode;
use super::support::{find, live};
use rustycad_tkernel::standard::{Failure, Guid, Handle};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};

/// `if (!L.FindAttribute(guid, aNode)) aNode = XCAFDoc_GraphNode::Set(L);`: the node of the
/// GUID on the label, else the label's default node (created when absent).
pub(crate) fn node_or_set(
    label: &TdfLabel,
    guid: &Guid,
) -> Result<Handle<XCAFDocGraphNode>, Failure> {
    let node = find::<XCAFDocGraphNode>(label, guid)?;
    if !node.is_null() {
        return Ok(node);
    }
    XCAFDocGraphNode::set(label)
}

/// The node of the GUID on the label, or null.
pub(crate) fn node_of(label: &TdfLabel, guid: &Guid) -> Result<Handle<XCAFDocGraphNode>, Failure> {
    find::<XCAFDocGraphNode>(label, guid)
}

/// Unlinks every father of the label's node of the GUID (forgetting the fathers left
/// without children), then forgets the node.
pub(crate) fn detach_fathers(label: &TdfLabel, guid: &Guid) -> Result<(), Failure> {
    unlink_fathers(label, guid, guid)?;
    label.forget_attribute_id(guid)?;
    Ok(())
}

/// Unlinks every father of the label's node of `guid`, forgetting the attribute of
/// `father_guid` of the fathers left without children (the view tool passes another GUID
/// for its clipping planes, as natively).
pub(crate) fn unlink_fathers(
    label: &TdfLabel,
    guid: &Guid,
    father_guid: &Guid,
) -> Result<(), Failure> {
    let child = node_of(label, guid)?;
    let Some(node) = child.get() else {
        return Ok(());
    };
    while node.nb_fathers() > 0 {
        let father = node.get_father(1)?;
        let father_node = live(&father, "XCAFDoc_GraphNode::UnSetChild")?;
        father_node.unset_child(&child)?;
        if father_node.nb_children() == 0 {
            father_node.label().forget_attribute_id(father_guid)?;
        }
    }
    Ok(())
}

/// The label's node of the GUID, else a new node set to the GUID.
pub(crate) fn child_node(
    label: &TdfLabel,
    guid: &Guid,
) -> Result<Handle<XCAFDocGraphNode>, Failure> {
    let node = node_of(label, guid)?;
    if !node.is_null() {
        return Ok(node);
    }
    default_node(label, guid)
}

/// `aNode = XCAFDoc_GraphNode::Set(L); aNode->SetGraphID(guid)`: the label's default node
/// (added when absent), set to the GUID.
pub(crate) fn default_node(
    label: &TdfLabel,
    guid: &Guid,
) -> Result<Handle<XCAFDocGraphNode>, Failure> {
    let node = XCAFDocGraphNode::set(label)?;
    live(&node, "XCAFDoc_GraphNode::SetGraphID")?.set_graph_id(guid)?;
    Ok(node)
}

/// Links a father label to the child node: the father's node of the GUID (or default node)
/// is set to the GUID and the two nodes refer to each other.
pub(crate) fn link_father(
    father: &TdfLabel,
    child: &Handle<XCAFDocGraphNode>,
    guid: &Guid,
) -> Result<(), Failure> {
    let node = node_or_set(father, guid)?;
    let father_node = live(&node, "XCAFDoc_GraphNode::SetChild")?;
    father_node.set_graph_id(guid)?;
    father_node.set_child(child)?;
    live(child, "XCAFDoc_GraphNode::SetFather")?.set_father(&node)?;
    Ok(())
}

/// Links the two labels with nodes of the GUID (each found or created and set to the GUID),
/// the first the father.
pub(crate) fn link_pair(father: &TdfLabel, child: &TdfLabel, guid: &Guid) -> Result<(), Failure> {
    let father_node = node_or_set(father, guid)?;
    let child_node = node_or_set(child, guid)?;
    live(&father_node, "XCAFDoc_GraphNode::SetGraphID")?.set_graph_id(guid)?;
    live(&child_node, "XCAFDoc_GraphNode::SetGraphID")?.set_graph_id(guid)?;
    live(&father_node, "XCAFDoc_GraphNode::SetChild")?.set_child(&child_node)?;
    live(&child_node, "XCAFDoc_GraphNode::SetFather")?.set_father(&father_node)?;
    Ok(())
}

/// The labels of the fathers of the label's node of the GUID, `None` without that node.
pub(crate) fn father_labels(
    label: &TdfLabel,
    guid: &Guid,
) -> Result<Option<Vec<TdfLabel>>, Failure> {
    let node = node_of(label, guid)?;
    let Some(node) = node.get() else {
        return Ok(None);
    };
    let fathers = node
        .fathers()
        .iter()
        .map(|f| Ok(live(f, "XCAFDoc_GraphNode")?.label()))
        .collect::<Result<_, Failure>>()?;
    Ok(Some(fathers))
}

/// The labels of the children of the label's node of the GUID, `None` without that node.
pub(crate) fn child_labels_of(
    label: &TdfLabel,
    guid: &Guid,
) -> Result<Option<Vec<TdfLabel>>, Failure> {
    let node = node_of(label, guid)?;
    let Some(node) = node.get() else {
        return Ok(None);
    };
    let children = node
        .children()
        .iter()
        .map(|c| Ok(live(c, "XCAFDoc_GraphNode")?.label()))
        .collect::<Result<_, Failure>>()?;
    Ok(Some(children))
}
