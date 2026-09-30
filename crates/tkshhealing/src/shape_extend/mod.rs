// Rust translation of ModelingAlgorithms/TKShHealing/ShapeExtend at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package ShapeExtend: the tools and data structures the other Shape Healing
//! packages share.
//!
//! - [`ShapeExtendStatus`] codes the flags of the tools' statuses as bits
//!   ([`ShapeExtend::encode_status`]), and [`ShapeExtend::init`] loads the Shape Healing
//!   messages (the SHMessage resource) into the message catalog;
//! - [`ShapeExtendWireData`] is a wire as an ordered list of edges;
//! - [`ShapeExtendExplorer`] converts collections of shapes and sorts shapes by type;
//! - [`ShapeExtendComplexCurve`] is the abstract curve of several segments and
//!   [`ShapeExtendCompositeSurface`] a grid of surface patches
//!   ([`ShapeExtendParametrisation`] chooses its global parametrisation);
//! - [`BasicMsgRegistrator`] receives the messages of the tools, and
//!   [`ShapeExtendMsgRegistrator`] keeps them per object and per shape.
//!
//! docs/shhealing-design.md section 2 fixes how the transient classes map.
mod complex_curve;
mod composite_surface;
mod explorer;
mod msg_registrator;
mod package;
mod status;
mod wire_data;

pub use complex_curve::ShapeExtendComplexCurve;
pub use composite_surface::{JointValues, ShapeExtendCompositeSurface, SurfaceGrid};
pub use explorer::{ShapeDispatch, ShapeExtendExplorer, ShapeHSequence};
pub use msg_registrator::{
    BasicMsgRegistrator, ShapeExtendBasicMsgRegistrator, ShapeExtendMsgRegistrator,
    ShapeMessageMap, TransientKey, TransientKeyHasher, TransientMessageMap,
};
pub use package::ShapeExtend;
pub use status::{ShapeExtendParametrisation, ShapeExtendStatus};
pub use wire_data::ShapeExtendWireData;
