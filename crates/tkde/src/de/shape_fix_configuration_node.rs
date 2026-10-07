// Rust port of DataExchange/TKDE/DE/DE_ShapeFixConfigurationNode.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DE_ShapeFixConfigurationNode`: the base of the format nodes that heal shapes (STEP,
//! IGES): the `provider.FORMAT.VENDOR.healing.*` resources.
use super::shape_fix_keys::{AVAILABLE_MODES, FIX_MODE_KEYS, TOLERANCE_KEYS};
use super::{
    DeConfigurationContext, DeConfigurationNode, DeConfigurationNodeBase, DeShapeFixParameters,
    DeShapeFixParametersFixMode, live,
};
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{Failure, Handle, SharedState},
    tcollection::AsciiString,
};

/// The native `THE_CONFIGURATION_SCOPE()` of `DE_ShapeFixConfigurationNode`.
const CONFIGURATION_SCOPE: &str = "provider";

/// The fields of `DE_ShapeFixConfigurationNode`: the node fields and `ShapeFixParameters`;
/// `DeShapeFixConfigurationNodeBase::default()` is the default constructor.
#[derive(Default)]
pub struct DeShapeFixConfigurationNodeBase {
    node: DeConfigurationNodeBase,
    parameters: SharedState<DeShapeFixParameters>,
}

impl DeShapeFixConfigurationNodeBase {
    /// `DE_ShapeFixConfigurationNode(theConfigurationNode)`: the node fields and the healing
    /// parameters of another node; a null node raises `Standard_NullObject`;
    /// `DeShapeFixConfigurationNodeBase::copied_from(&node)?`.
    pub fn copied_from(node: &Handle<dyn DeShapeFixConfigurationNode>) -> Result<Self, Failure> {
        let source = live(node, "DE_ShapeFixConfigurationNode(theConfigurationNode)")?;
        Ok(Self {
            node: DeConfigurationNodeBase::copied_from(&node.cast())?,
            parameters: SharedState::new(source.shape_fix_parameters()),
        })
    }
    /// The embedded `DE_ConfigurationNode` fields; `base.configuration_node_base()`.
    pub fn configuration_node_base(&self) -> &DeConfigurationNodeBase {
        &self.node
    }
    /// The public `ShapeFixParameters` field; `base.shape_fix_parameters().tolerance3d`.
    pub fn shape_fix_parameters(&self) -> DeShapeFixParameters {
        *self.parameters.read()
    }
    /// Assigns the public `ShapeFixParameters` field; `base.set_shape_fix_parameters(p)`.
    pub fn set_shape_fix_parameters(&self, parameters: DeShapeFixParameters) {
        *self.parameters.write() = parameters;
    }
}

/// `DE_ShapeFixConfigurationNode`: a configuration node with shape-healing parameters.
///
/// A concrete node returns `shape_fix_base().configuration_node_base()` from
/// `configuration_node_base`, and its `load` and `save` call
/// [`DeShapeFixConfigurationNode::shape_fix_load`] and
/// [`DeShapeFixConfigurationNode::shape_fix_save`] where the C++ calls the base class
/// (de-design.md section 1).
pub trait DeShapeFixConfigurationNode: DeConfigurationNode {
    /// The embedded base with the native fields.
    fn shape_fix_base(&self) -> &DeShapeFixConfigurationNodeBase;

    /// `DE_ShapeFixConfigurationNode::Load(theResource)`: reads the healing resources of
    /// `provider.FORMAT.VENDOR.healing`; a mode takes any integer. Always `true`; a null
    /// context raises `Standard_NullObject`.
    fn shape_fix_load(&self, resource: &Handle<DeConfigurationContext>) -> Result<bool, Failure> {
        let context = live(resource, "DE_ShapeFixConfigurationNode::Load")?;
        let scope: AsciiString = healing_scope(self, "");
        let mut parameters: DeShapeFixParameters = self.shape_fix_parameters();
        for entry in &TOLERANCE_KEYS {
            let value: &mut f64 = (entry.field)(&mut parameters);
            *value = context.real_val(entry.key, *value, &scope);
        }
        for entry in &FIX_MODE_KEYS {
            let mode: &mut DeShapeFixParametersFixMode = (entry.field)(&mut parameters);
            *mode = DeShapeFixParametersFixMode(context.integer_val(entry.key, mode.0, &scope));
        }
        self.set_shape_fix_parameters(parameters);
        Ok(true)
    }

    /// `DE_ShapeFixConfigurationNode::Save()`: the healing resources with their comments.
    fn shape_fix_save(&self) -> AsciiString {
        let scope: AsciiString = healing_scope(self, ".");
        let mut parameters: DeShapeFixParameters = self.shape_fix_parameters();
        let mut result: AsciiString = AsciiString::new("!\n!Shape healing parameters:\n!\n");
        for entry in &TOLERANCE_KEYS {
            let value: f64 = *(entry.field)(&mut parameters);
            append_resource(
                &mut result,
                entry.comment,
                &scope,
                entry.key,
                AsciiString::from_real(value),
            );
        }
        for entry in &FIX_MODE_KEYS {
            let mode: DeShapeFixParametersFixMode = *(entry.field)(&mut parameters);
            let comment: String = format!(
                "{}!Default value: {}. {AVAILABLE_MODES}",
                entry.description, entry.default_label
            );
            append_resource(
                &mut result,
                &comment,
                &scope,
                entry.key,
                AsciiString::from_integer(mode.0),
            );
        }
        result
    }

    /// The public `ShapeFixParameters` field.
    fn shape_fix_parameters(&self) -> DeShapeFixParameters {
        self.shape_fix_base().shape_fix_parameters()
    }
    /// Assigns the public `ShapeFixParameters` field.
    fn set_shape_fix_parameters(&self, parameters: DeShapeFixParameters) {
        self.shape_fix_base().set_shape_fix_parameters(parameters);
    }
}
impl_transient_interface!(
    dyn DeShapeFixConfigurationNode,
    "DE_ShapeFixConfigurationNode",
    dyn DeConfigurationNode
);

/// `provider.FORMAT.VENDOR.healing` followed by `suffix`.
fn healing_scope<N: DeShapeFixConfigurationNode + ?Sized>(node: &N, suffix: &str) -> AsciiString {
    let mut scope: AsciiString = AsciiString::new(CONFIGURATION_SCOPE);
    scope += ".";
    scope += &node.get_format();
    scope += ".";
    scope += &node.get_vendor();
    scope += ".healing";
    scope += suffix;
    scope
}

/// One saved resource: `!`, the comment, `SCOPE.KEY :\t VALUE`, `!`.
fn append_resource(
    result: &mut AsciiString,
    comment: &str,
    scope: &AsciiString,
    key: &str,
    value: AsciiString,
) {
    *result += "!\n";
    *result += comment;
    *result += scope;
    *result += key;
    *result += " :\t ";
    *result += &value;
    *result += "\n!\n";
}
