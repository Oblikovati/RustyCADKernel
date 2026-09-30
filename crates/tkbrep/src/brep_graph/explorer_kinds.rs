// Rust translation of childExplorerKindCanContainDescendant and
// parentExplorerKindCanContainDescendant (ModelingData/TKBRep/BRepGraph/
// BRepGraph_ChildExplorer.cxx and BRepGraph_ParentExplorer.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The helpers both explorers share: the kind containment table (the two native
//! copies are identical) and the location product of one traversal step.
use crate::brep_graph::NodeKind;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::toploc::Location;

/// `*KindCanContainDescendant(theParentKind, theTargetKind)`: whether a node of the
/// parent kind can have a descendant of the target kind.
pub(crate) fn kind_can_contain(parent: NodeKind, target: NodeKind) -> bool {
    const SHAPE_KINDS: [NodeKind; 7] = [
        NodeKind::SOLID,
        NodeKind::SHELL,
        NodeKind::FACE,
        NodeKind::WIRE,
        NodeKind::CO_EDGE,
        NodeKind::EDGE,
        NodeKind::VERTEX,
    ];
    let below = |from: usize| SHAPE_KINDS[from..].contains(&target);
    match parent {
        NodeKind::PRODUCT | NodeKind::OCCURRENCE => {
            target == NodeKind::OCCURRENCE
                || target == NodeKind::PRODUCT
                || NodeKind::is_topology_kind(target)
        }
        NodeKind::COMPOUND => NodeKind::is_topology_kind(target),
        NodeKind::COMP_SOLID => below(0),
        NodeKind::SOLID => below(1),
        NodeKind::SHELL => below(2),
        NodeKind::FACE => below(3),
        NodeKind::WIRE => below(4),
        NodeKind::CO_EDGE => below(5),
        NodeKind::EDGE => below(6),
        _ => false,
    }
}

/// `theAccumulated * theLocal` (`TopLoc_Location::operator*`); the product of two
/// existing locations only raises when a datum power cannot be built, as natively.
pub(crate) fn compose_location(
    accumulated: &Location,
    local: &Location,
) -> Result<Location, Failure> {
    accumulated.multiplied(local)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn containment_follows_the_native_table() {
        assert!(kind_can_contain(NodeKind::COMP_SOLID, NodeKind::SOLID));
        assert!(!kind_can_contain(NodeKind::SOLID, NodeKind::SOLID));
        assert!(kind_can_contain(NodeKind::CO_EDGE, NodeKind::VERTEX));
        assert!(!kind_can_contain(NodeKind::EDGE, NodeKind::CO_EDGE));
        assert!(kind_can_contain(NodeKind::COMPOUND, NodeKind::COMPOUND));
        assert!(!kind_can_contain(NodeKind::COMPOUND, NodeKind::PRODUCT));
        assert!(kind_can_contain(NodeKind::OCCURRENCE, NodeKind::PRODUCT));
        assert!(!kind_can_contain(NodeKind::VERTEX, NodeKind::VERTEX));
        assert!(!kind_can_contain(NodeKind(9), NodeKind::VERTEX));
    }
}
