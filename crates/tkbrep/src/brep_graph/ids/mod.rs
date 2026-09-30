// Rust translation of the BRepGraph id headers at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The BRepGraph ids: node, reference and representation addresses, their typed
//! wrappers, the generic item address and the persistent UIDs.
pub(crate) mod hashing;
mod item_id;
mod item_uid;
mod node_id;
mod ref_id;
mod rep_id;
mod typed;
mod uid;

pub use item_id::{ItemDomain, ItemId};
pub use item_uid::ItemUid;
pub use node_id::{
    CoEdgeId, CoEdgeTag, CompSolidId, CompSolidTag, CompoundId, CompoundTag, EdgeId, EdgeTag,
    FaceId, FaceTag, NodeFamily, NodeId, NodeKind, NodeVisitor, OccurrenceId, OccurrenceTag,
    ProductId, ProductTag, ShellId, ShellTag, SolidId, SolidTag, VertexId, VertexTag, WireId,
    WireTag,
};
pub use ref_id::{
    ChildRefId, ChildRefTag, FaceRefId, FaceRefTag, OccurrenceRefId, OccurrenceRefTag, RefFamily,
    RefId, RefKind, RefVisitor, ShellRefId, ShellRefTag, SolidRefId, SolidRefTag, VertexRefId,
    VertexRefTag, WireRefId, WireRefTag,
};
pub use rep_id::{
    CoEdgeCurve2DRepId, CoEdgeCurve2DTag, CoEdgePolygon2DRepId, CoEdgePolygon2DTag,
    CoEdgePolygonOnTriRepId, CoEdgePolygonOnTriTag, EdgeCurve3DRepId, EdgeCurve3DTag,
    EdgePolygon3DRepId, EdgePolygon3DTag, FaceSurfaceRepId, FaceSurfaceTag, FaceTriangulationRepId,
    FaceTriangulationTag, RepFamily, RepId, RepKind,
};
pub use typed::{CountProvider, IdFamily, KindTag, OwnedFamily, TypedId};
pub use uid::{RefUid, Uid};
