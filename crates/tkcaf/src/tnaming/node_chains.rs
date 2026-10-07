// Rust port of the file-static chain functions of
// ApplicationFramework/TKCAF/TNaming/TNaming_NamedShape.cxx (RemoveNode,
// UpdateFirstUseOrNextSameShape, SelectSameShape) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The surgery on the shape chains of the evolution graph (ocaf-design.md section 10).
use super::node::same_pointer;
use super::{TNamingNode, TNamingNodePtr, TNamingRefShape, TNamingShapeRefMap, TNamingUsedShapes};
use rustycad_tkernel::standard::{Failure, exceptions};
use rustycad_tklcaf::tdf::TdfAttributeState;
use std::sync::Arc;

/// The body of `Clear` and of `AfterUndo` on an addition: `RemoveNode` on every node of
/// the attribute chain, then the native `delete` of each node.
pub(crate) fn remove_nodes(used: Option<&TNamingUsedShapes>, first: &TNamingNodePtr) {
    let mut map = used.map(TNamingUsedShapes::map);
    let mut current = first.clone();
    while let Some(node) = current {
        remove_node(map.as_deref_mut(), &node);
        current = node.next_same_attribute();
    }
    drop(map);
    let mut current = first.clone();
    while let Some(node) = current {
        current = node.next_same_attribute();
        node.release();
    }
}

/// The file-static `RemoveNode(MapExist, M, N)`: unlinks the node from the chains of its
/// old and new shapes; a shape that no node uses any more leaves the map and is deleted.
fn remove_node(mut map: Option<&mut TNamingShapeRefMap>, node: &Arc<TNamingNode>) {
    let links = node.links();
    if let Some(old) = &links.old {
        if is_first_use(old, node) {
            if links.next_same_old.is_some() {
                old.set_first_use(&links.next_same_old);
            } else {
                // The shape disappears.
                unbind(map.as_deref_mut(), old);
                node.edit(|links| links.old = None);
                if !same_pointer(&links.old, &links.new) {
                    old.release();
                }
            }
        } else {
            unlink(old, node, &links.next_same_old);
        }
    }
    let Some(new) = &links.new else { return };
    if !is_first_use(new, node) {
        unlink(new, node, &links.next_same_new);
        return;
    }
    if links.next_same_new.is_some() {
        new.set_first_use(&links.next_same_new);
        return;
    }
    // The shape disappears.
    unbind(map, new);
    new.release();
    node.edit(|links| links.new = None);
}

/// `pos->FirstUse() == N`.
fn is_first_use(shape: &TNamingRefShape, node: &Arc<TNamingNode>) -> bool {
    shape
        .first_use()
        .is_some_and(|first| Arc::ptr_eq(&first, node))
}

/// `if (MapExist) M.UnBind(pos->Shape())`.
fn unbind(map: Option<&mut TNamingShapeRefMap>, shape: &TNamingRefShape) {
    if let Some(map) = map {
        map.unbind(&shape.shape());
    }
}

/// Bypasses `node` in the chain of `shape`: its predecessor's link of that shape becomes
/// `replacement` (the node's own next link of that shape).
fn unlink(shape: &Arc<TNamingRefShape>, node: &Arc<TNamingNode>, replacement: &TNamingNodePtr) {
    let reference = Some(shape.clone());
    let mut current = shape.first_use();
    while let Some(previous) = current {
        let next = previous.next_same_shape(&reference);
        if next.as_ref().is_some_and(|next| Arc::ptr_eq(next, node)) {
            relink(&previous, &reference, replacement);
            return;
        }
        current = next;
    }
}

/// Writes the link of `shape` of `previous`: its old-shape link when it uses `shape` as old
/// shape, else its new-shape link.
fn relink(previous: &TNamingNode, shape: &Option<Arc<TNamingRefShape>>, next: &TNamingNodePtr) {
    previous.edit(|links| {
        if same_pointer(&links.old, shape) {
            links.next_same_old = next.clone();
        } else {
            links.next_same_new = next.clone();
        }
    });
}

/// The file-static `UpdateFirstUseOrNextSameShape(prs, pdn)` of the builder: the node
/// becomes the first use of the shape, or is appended to the end of its chain.
/// `Standard_ConstructionError` when the chain loops on one node.
pub(crate) fn append_to_chain(
    shape: &Arc<TNamingRefShape>,
    node: &Arc<TNamingNode>,
) -> Result<(), Failure> {
    let Some(mut last) = shape.first_use() else {
        shape.set_first_use(&Some(node.clone()));
        return Ok(());
    };
    let reference = Some(shape.clone());
    let mut current = Some(last.clone());
    while let Some(candidate) = current {
        last = candidate;
        current = last.next_same_shape(&reference);
        if current
            .as_ref()
            .is_some_and(|next| Arc::ptr_eq(next, &last))
        {
            return Err(exceptions::construction_error(
                "UpdateFirstUseOrNextSameShape: the shape chain loops on one node; expected a chain that ends",
            ));
        }
    }
    // A forbidden and useless loop.
    if Arc::ptr_eq(&last, node) {
        return Ok(());
    }
    last.edit(|links| {
        if same_pointer(&links.old, &reference) {
            links.next_same_old = Some(node.clone());
        }
        if same_pointer(&links.new, &reference) {
            links.next_same_new = Some(node.clone());
        }
    });
    Ok(())
}

/// The file-static `SelectSameShape(myNode, Old, RS, Trans)`: from `node` along the chain
/// of `shape`, the first valid node that uses `shape` as its old shape (`old`) or its new
/// shape, with another shape on the other side. A negative transaction checks the current
/// validity of the attribute, else its life span.
pub(crate) fn select_same_shape(
    node: TNamingNodePtr,
    old: bool,
    shape: &Option<Arc<TNamingRefShape>>,
    transaction: i32,
) -> Result<TNamingNodePtr, Failure> {
    let mut current = node;
    while let Some(candidate) = current {
        if is_valid(&candidate, transaction)? && is_selected(&candidate, old, shape) {
            return Ok(Some(candidate));
        }
        current = candidate.next_same_shape(shape);
    }
    Ok(None)
}

/// The validity test of `SelectSameShape`.
fn is_valid(node: &TNamingNode, transaction: i32) -> Result<bool, Failure> {
    if transaction >= 0 {
        return node.is_valid_in_trans(transaction);
    }
    let attribute = node.live_attribute("SelectSameShape")?;
    Ok(rustycad_tklcaf::live(&attribute, "SelectSameShape")?.is_valid())
}

/// The shape test of `SelectSameShape`.
fn is_selected(node: &TNamingNode, old: bool, shape: &Option<Arc<TNamingRefShape>>) -> bool {
    let links = node.links();
    let (this_side, other_side) = if old {
        (&links.old, &links.new)
    } else {
        (&links.new, &links.old)
    };
    same_pointer(this_side, shape) && other_side.is_some() && !same_pointer(other_side, shape)
}
