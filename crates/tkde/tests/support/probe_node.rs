//! `ProbeNode` and `ProbeProvider`: the shape-healing node and the provider of
//! tools/oracle/tkde_configuration_probe.cxx, with a fixed format, vendor, extensions,
//! capabilities and content magic, and one `value` resource of their own.
use rustycad_tkde::de::{
    DeConfigurationContext, DeConfigurationNode, DeConfigurationNodeBase, DeProvider,
    DeProviderBase, DeShapeFixConfigurationNode, DeShapeFixConfigurationNodeBase, live,
    new_configuration_node,
};
use rustycad_tkernel::{
    impl_transient,
    ncollection::{Buffer, List},
    standard::{Failure, Handle, SharedState, StandardTransient},
    tcollection::AsciiString,
};

/// The native `ProbeProvider`: names itself and implements no transfer.
pub struct ProbeProvider {
    transient: StandardTransient,
    base: DeProviderBase,
    format: AsciiString,
    vendor: AsciiString,
}
impl_transient!(ProbeProvider, "ProbeProvider", dyn DeProvider, transient, interfaces: [dyn DeProvider]);
impl DeProvider for ProbeProvider {
    fn provider_base(&self) -> &DeProviderBase {
        &self.base
    }
    fn get_format(&self) -> AsciiString {
        self.format.clone()
    }
    fn get_vendor(&self) -> AsciiString {
        self.vendor.clone()
    }
}

/// The fixed description of a probe node.
#[derive(Clone, Default)]
pub struct ProbeSpec {
    pub format: &'static str,
    pub vendor: &'static str,
    pub extensions: &'static str,
    pub import: bool,
    pub export: bool,
    pub magic: &'static str,
}

/// The native `ProbeNode`.
pub struct ProbeNode {
    transient: StandardTransient,
    base: DeShapeFixConfigurationNodeBase,
    spec: ProbeSpec,
    value: SharedState<i32>,
}
impl_transient!(
    ProbeNode,
    "ProbeNode",
    dyn DeShapeFixConfigurationNode,
    transient,
    interfaces: [dyn DeConfigurationNode, dyn DeShapeFixConfigurationNode]
);

impl ProbeNode {
    /// `new ProbeNode(...)`, bound to its handle.
    pub fn create(spec: ProbeSpec) -> Handle<ProbeNode> {
        new_configuration_node(Self::unbound(spec))
    }
    /// The node before it has a handle (for the plugin holders' `Default`).
    pub fn unbound(spec: ProbeSpec) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: DeShapeFixConfigurationNodeBase::default(),
            spec,
            value: SharedState::new(7),
        }
    }
    /// The public `myValue`.
    pub fn value(&self) -> i32 {
        *self.value.read()
    }
    /// Assigns `myValue`.
    pub fn set_value(&self, value: i32) {
        *self.value.write() = value;
    }
    fn scope(&self) -> String {
        format!("provider.{}.{}", self.spec.format, self.spec.vendor)
    }
}

impl DeConfigurationNode for ProbeNode {
    fn configuration_node_base(&self) -> &DeConfigurationNodeBase {
        self.base.configuration_node_base()
    }
    fn load(&self, resource: &Handle<DeConfigurationContext>) -> Result<bool, Failure> {
        let context = live(resource, "ProbeNode::Load")?;
        self.set_value(context.integer_val("value", self.value(), self.scope()));
        self.shape_fix_load(resource)
    }
    fn save(&self) -> AsciiString {
        let mut result = AsciiString::new(&format!("{}.value :\t ", self.scope()));
        result += self.value();
        result += "\n";
        result += &self.shape_fix_save();
        result
    }
    fn build_provider(&self) -> Handle<dyn DeProvider> {
        let node = self.configuration_node_base().this().unwrap_or_default();
        let provider = ProbeProvider {
            transient: StandardTransient::default(),
            base: DeProviderBase::with_node(&node),
            format: AsciiString::new(self.spec.format),
            vendor: AsciiString::new(self.spec.vendor),
        };
        Handle::new(provider).cast()
    }
    fn copy(&self) -> Handle<dyn DeConfigurationNode> {
        ProbeNode::create(self.spec.clone()).cast()
    }
    fn get_format(&self) -> AsciiString {
        AsciiString::new(self.spec.format)
    }
    fn get_vendor(&self) -> AsciiString {
        AsciiString::new(self.spec.vendor)
    }
    fn get_extensions(&self) -> List<AsciiString> {
        let mut list: List<AsciiString> = List::default();
        let all = AsciiString::new(self.spec.extensions);
        for index in 1.. {
            let token = all.token(" ", index);
            if token.is_empty() {
                break;
            }
            list.append(token);
        }
        list
    }
    fn is_import_supported(&self) -> bool {
        self.spec.import
    }
    fn is_export_supported(&self) -> bool {
        self.spec.export
    }
    fn check_content(&self, buffer: &Handle<Buffer>) -> bool {
        let Some(content) = buffer.get() else {
            return false;
        };
        if self.spec.magic.is_empty() {
            return false;
        }
        let text: &[u8] = content.as_slice();
        let end: usize = text
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(text.len());
        text[..end].starts_with(self.spec.magic.as_bytes())
    }
}

impl DeShapeFixConfigurationNode for ProbeNode {
    fn shape_fix_base(&self) -> &DeShapeFixConfigurationNodeBase {
        &self.base
    }
}

/// `node(format, vendor, extensions, import, export, magic)` of the probe.
pub fn probe_node(
    format: &'static str,
    vendor: &'static str,
    extensions: &'static str,
    import: bool,
    export: bool,
    magic: &'static str,
) -> Handle<ProbeNode> {
    ProbeNode::create(ProbeSpec {
        format,
        vendor,
        extensions,
        import,
        export,
        magic,
    })
}

/// The probe's `PluginVendorA`: format `PLG`, vendor `A`.
pub struct PluginVendorA {
    node: ProbeNode,
}
/// The probe's `PluginVendorB`: format `PLG`, vendor `B`.
pub struct PluginVendorB {
    node: ProbeNode,
}

/// Forwards the node members of a plugin vendor to its probe node.
macro_rules! plugin_vendor {
    ($name:ident, $text:literal, $vendor:literal) => {
        impl Default for $name {
            fn default() -> Self {
                $name {
                    node: ProbeNode::unbound(ProbeSpec {
                    format: "PLG",
                    vendor: $vendor,
                    extensions: "plg",
                    import: true,
                    export: true,
                    magic: "",
                }),
                }
            }
        }
        impl_transient!($name, $text, ProbeNode, node, interfaces: [dyn DeConfigurationNode]);
        impl DeConfigurationNode for $name {
            fn configuration_node_base(&self) -> &DeConfigurationNodeBase {
                self.node.configuration_node_base()
            }
            fn load(&self, resource: &Handle<DeConfigurationContext>) -> Result<bool, Failure> {
                self.node.load(resource)
            }
            fn save(&self) -> AsciiString {
                self.node.save()
            }
            fn build_provider(&self) -> Handle<dyn DeProvider> {
                self.node.build_provider()
            }
            fn copy(&self) -> Handle<dyn DeConfigurationNode> {
                self.node.copy()
            }
            fn get_format(&self) -> AsciiString {
                self.node.get_format()
            }
            fn get_vendor(&self) -> AsciiString {
                self.node.get_vendor()
            }
            fn get_extensions(&self) -> List<AsciiString> {
                self.node.get_extensions()
            }
        }
    };
}
plugin_vendor!(PluginVendorA, "PluginVendorA", "A");
plugin_vendor!(PluginVendorB, "PluginVendorB", "B");
