// Rust translation of ModelingAlgorithms/TKShHealing/ShapeCustom/ShapeCustom_Modification.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeCustom_Modification`: a BRepTools modification that sends its messages to a
//! registrator (docs/shhealing-design.md section 7).
use crate::shape_extend::BasicMsgRegistrator;
use rustycad_tkbrep::brep_tools::BRepToolsModification;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::message::{Gravity, Msg};
use rustycad_tkernel::standard::{Handle, SharedState};

/// The private field `myMsgReg` of a `ShapeCustom_Modification`, embedded in each
/// concrete modification and lent through
/// [`ShapeCustomModification::modification_messages`].
#[derive(Default)]
pub struct ShapeCustomModificationFields {
    msg_reg: SharedState<Handle<dyn BasicMsgRegistrator>>,
}

/// `ShapeCustom_Modification`: the abstract modification of the ShapeCustom tools; it
/// adds a message registrator to `BRepTools_Modification`. A concrete modification
/// implements the `BRepToolsModification` members and lends its
/// [`ShapeCustomModificationFields`].
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkernel::message::{Gravity, Msg};
/// use rustycad_tkshhealing::shape_custom::{ShapeCustomDirectModification, ShapeCustomModification};
/// let modification = ShapeCustomDirectModification::new();
/// assert!(modification.msg_registrator().is_null());
/// modification.send_msg(&Shape::default(), &Msg::new(), Gravity::Info); // no registrator
/// ```
pub trait ShapeCustomModification: BRepToolsModification {
    /// The embedded private fields of the base class.
    fn modification_messages(&self) -> &ShapeCustomModificationFields;

    /// `SetMsgRegistrator(msgreg)` (virtual): the registrator of the messages;
    /// `modification.set_msg_registrator(&registrator)`.
    fn set_msg_registrator(&self, registrator: &Handle<dyn BasicMsgRegistrator>) {
        *self.modification_messages().msg_reg.write() = registrator.clone();
    }

    /// `MsgRegistrator()`: the registrator, null by default;
    /// `modification.msg_registrator()`.
    fn msg_registrator(&self) -> Handle<dyn BasicMsgRegistrator> {
        self.modification_messages().msg_reg.read().clone()
    }

    /// `SendMsg(shape, message, gravity)`: sends the message to the registrator, if any
    /// (the native default gravity is Message_Info);
    /// `modification.send_msg(&shape, &message, Gravity::Info)`.
    fn send_msg(&self, shape: &Shape, message: &Msg, gravity: Gravity) {
        let registrator = self.msg_registrator();
        if let Some(registrator) = registrator.get() {
            registrator.send_shape(shape, message, gravity);
        }
    }
}
impl_transient_interface!(
    dyn ShapeCustomModification,
    "ShapeCustom_Modification",
    dyn BRepToolsModification
);
