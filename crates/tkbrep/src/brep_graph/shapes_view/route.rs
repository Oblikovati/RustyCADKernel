// Rust translation of ShapeParentRoute, isCoreParityOrientation, shapeWithoutOwnLocation,
// shouldStoreRootLocationInRef, classifyShapeToParent and attachSupplementToParent of
// ModelingData/TKBRep/BRepGraph/BRepGraph_ShapesView.cxx, with the owner check of
// BRepGraph_SupplementEditor::Attach (BRepGraph_SupplementEditor.cxx),
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Where a shape added under a parent goes: into the core model, into the topology
//! supplement layer, or nowhere.
use super::Options;
use crate::brep_graph::hooks::AttachmentKind;
use crate::brep_graph::layer_topo_supplement::LayerTopoSupplement;
use crate::brep_graph::{BRepGraph, NodeId, NodeKind};
use crate::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::toploc::Location;

/// `ShapeParentRoute`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ShapeParentRoute {
    Reject,
    Core,
    Supplement,
}

fn is_core_parity_orientation(orientation: Orientation) -> bool {
    orientation == Orientation::Forward || orientation == Orientation::Reversed
}

/// `shapeWithoutOwnLocation`: the shape at the identity location.
pub(super) fn shape_without_own_location(shape: &Shape) -> Result<Shape, Failure> {
    let mut result = shape.clone();
    result.set_location(&Location::default(), false)?;
    Ok(result)
}

/// `shouldStoreRootLocationInRef`: under a product or compound parent, or (unparented)
/// with an auto product, the root location goes into the reference.
pub(super) fn should_store_root_location_in_ref(options: &Options, parent: NodeId) -> bool {
    if parent.is_valid() {
        return parent.kind == NodeKind::PRODUCT || parent.kind == NodeKind::COMPOUND;
    }
    options.create_auto_product
}

// The container route: core for the expected child type in a parity orientation.
fn container_route(
    shape_type: ShapeEnum,
    orientation: Orientation,
    core_type: Option<ShapeEnum>,
) -> ShapeParentRoute {
    if shape_type == ShapeEnum::Shape {
        return ShapeParentRoute::Reject;
    }
    let type_ok = core_type.is_none_or(|expected| expected == shape_type);
    if type_ok && is_core_parity_orientation(orientation) {
        ShapeParentRoute::Core
    } else {
        ShapeParentRoute::Supplement
    }
}

/// `classifyShapeToParent(theShapeType, theOrientation, theParent)`.
pub(super) fn classify_shape_to_parent(
    shape_type: ShapeEnum,
    orientation: Orientation,
    parent: NodeId,
) -> ShapeParentRoute {
    if !parent.is_valid() {
        return ShapeParentRoute::Reject;
    }
    match parent.kind {
        NodeKind::PRODUCT if shape_type == ShapeEnum::Shape => ShapeParentRoute::Reject,
        NodeKind::PRODUCT => ShapeParentRoute::Core,
        NodeKind::COMPOUND => container_route(shape_type, orientation, None),
        NodeKind::SOLID => container_route(shape_type, orientation, Some(ShapeEnum::Shell)),
        NodeKind::SHELL => container_route(shape_type, orientation, Some(ShapeEnum::Face)),
        NodeKind::COMP_SOLID => container_route(shape_type, orientation, Some(ShapeEnum::Solid)),
        NodeKind::FACE | NodeKind::EDGE if shape_type == ShapeEnum::Vertex => {
            ShapeParentRoute::Supplement
        }
        _ => ShapeParentRoute::Reject,
    }
}

// The attachment kind of `attachSupplementToParent` per parent kind.
fn parent_attachment_kind(parent: NodeKind) -> Option<AttachmentKind> {
    match parent {
        NodeKind::COMPOUND => Some(AttachmentKind::CompoundAuxShape),
        NodeKind::COMP_SOLID => Some(AttachmentKind::CompSolidAuxShape),
        NodeKind::SOLID => Some(AttachmentKind::SolidAuxShape),
        NodeKind::SHELL => Some(AttachmentKind::ShellAuxShape),
        NodeKind::FACE => Some(AttachmentKind::FaceDirectVertex),
        NodeKind::EDGE => Some(AttachmentKind::EdgeInternalVertex),
        _ => None,
    }
}

/// `attachSupplementToParent(theGraph, theShape, theParent)` (through
/// `Editor().Supplement().AttachTo*`): the attachment uid in the ensured supplement
/// layer, 0 when refused (a removed owner or a null shape).
pub(super) fn attach_supplement_to_parent(
    graph: &mut BRepGraph,
    shape: &Shape,
    parent: NodeId,
) -> Result<u64, Failure> {
    let Some(kind) = parent_attachment_kind(parent.kind) else {
        return Ok(0);
    };
    if graph.topo().generic().is_removed(parent) || shape.is_null() {
        return Ok(0);
    }
    let layer = graph.layer_registry().ensure::<LayerTopoSupplement>()?;
    Ok(layer
        .get()
        .map_or(0, |layer| layer.add_attachment(graph, parent, kind, shape)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_follow_the_native_table() {
        let face = NodeId::new(NodeKind::FACE, 0);
        let shell = NodeId::new(NodeKind::SHELL, 0);
        let fwd = Orientation::Forward;
        assert_eq!(
            classify_shape_to_parent(ShapeEnum::Face, fwd, NodeId::default()),
            ShapeParentRoute::Reject
        );
        assert_eq!(
            classify_shape_to_parent(ShapeEnum::Vertex, fwd, face),
            ShapeParentRoute::Supplement
        );
        assert_eq!(
            classify_shape_to_parent(ShapeEnum::Edge, fwd, face),
            ShapeParentRoute::Reject
        );
        assert_eq!(
            classify_shape_to_parent(ShapeEnum::Face, fwd, shell),
            ShapeParentRoute::Core
        );
        assert_eq!(
            classify_shape_to_parent(ShapeEnum::Face, Orientation::Internal, shell),
            ShapeParentRoute::Supplement
        );
        assert_eq!(
            classify_shape_to_parent(ShapeEnum::Edge, fwd, shell),
            ShapeParentRoute::Supplement
        );
        let product = NodeId::new(NodeKind::PRODUCT, 0);
        assert_eq!(
            classify_shape_to_parent(ShapeEnum::Shape, fwd, product),
            ShapeParentRoute::Reject
        );
        assert_eq!(
            classify_shape_to_parent(ShapeEnum::Wire, fwd, NodeId::new(NodeKind::WIRE, 0)),
            ShapeParentRoute::Reject
        );
        assert!(should_store_root_location_in_ref(
            &Options::default(),
            NodeId::default()
        ));
        assert!(!should_store_root_location_in_ref(
            &Options::default(),
            shell
        ));
        assert_eq!(parent_attachment_kind(NodeKind::WIRE), None);
    }
}
