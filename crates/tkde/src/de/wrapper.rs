// Rust port of DataExchange/TKDE/DE/DE_Wrapper.hxx and the registry members of
// DE_Wrapper.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DE_Wrapper`: the registry of configuration nodes by format and vendor, with the vendor
//! priority of each format (de-design.md section 4).
use super::{DeConfigurationNode, DeSectionGlobal};
use rustycad_tkernel::{
    impl_transient,
    ncollection::{DataMap, IndexedDataMap, List},
    standard::{Handle, SharedState, StandardTransient},
    tcollection::AsciiString,
};
use std::sync::RwLockReadGuard;

/// The vendors of one format in priority order: vendor name to node.
pub type DeVendorMap = IndexedDataMap<AsciiString, Handle<dyn DeConfigurationNode>>;

/// The native `myConfiguration`: format name to its vendors.
pub type DeFormatMap = DataMap<AsciiString, DeVendorMap>;

/// The scalar fields: `GlobalParameters` and `myKeepUpdates`.
#[derive(Clone, Copy, Debug)]
pub(super) struct WrapperSettings {
    pub(super) global_parameters: DeSectionGlobal,
    pub(super) keep_updates: bool,
}

/// `DE_Wrapper`: finds the node and builds the provider for a file, by extension or content,
/// trying the formats in map order and the vendors of each format in priority order.
///
/// The native virtual members (`FindProvider`, `FindReadProvider`, `FindWriteProvider`,
/// `Copy`) have no override in OCCT, so the class is a struct.
///
/// ```
/// use rustycad_tkde::de::DeWrapper;
/// let wrapper = DeWrapper::new();
/// assert!(!wrapper.keep_updates());
/// assert_eq!(wrapper.nodes().extent(), 0);
/// ```
pub struct DeWrapper {
    transient: StandardTransient,
    pub(super) settings: SharedState<WrapperSettings>,
    pub(super) configuration: SharedState<DeFormatMap>,
}
impl_transient!(DeWrapper, "DE_Wrapper", StandardTransient, transient);

impl Default for DeWrapper {
    fn default() -> Self {
        Self::with_parts(DeSectionGlobal::default(), false, DeFormatMap::default())
    }
}

impl DeWrapper {
    fn with_parts(
        global_parameters: DeSectionGlobal,
        keep_updates: bool,
        configuration: DeFormatMap,
    ) -> Self {
        Self {
            transient: StandardTransient::default(),
            settings: SharedState::new(WrapperSettings {
                global_parameters,
                keep_updates,
            }),
            configuration: SharedState::new(configuration),
        }
    }

    /// `DE_Wrapper()`: no node, units 1.0, `KeepUpdates` off; `DeWrapper::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `DE_Wrapper(theWrapper)`: copies the units and binds every node of `wrapper` (shared,
    /// not copied) in its map order. As natively, the source's `KeepUpdates` is then set to
    /// this wrapper's `false`; a null source gives an empty wrapper.
    pub fn from_wrapper(wrapper: &Handle<DeWrapper>) -> Self {
        let result: Self = Self::new();
        let Some(source) = wrapper.get() else {
            return result;
        };
        result.set_global_parameters(source.global_parameters());
        for node in source.node_snapshot() {
            result.bind(&node);
        }
        source.settings.write().keep_updates = result.keep_updates();
        result
    }

    /// `Copy()`: the implicit C++ copy constructor: the units, `KeepUpdates` and a copy of
    /// the maps whose nodes are shared; `wrapper.copy()`.
    pub fn copy(&self) -> Handle<DeWrapper> {
        let settings: WrapperSettings = *self.settings.read();
        let configuration: DeFormatMap = self.configuration.read().clone();
        Handle::new(Self::with_parts(
            settings.global_parameters,
            settings.keep_updates,
            configuration,
        ))
    }

    /// `Bind(theNode)`: adds the node under its format and vendor. A second node of the same
    /// format and vendor is not stored, and the result is still `true` (`IndexedDataMap::Add`
    /// keeps the first); a null node gives `false`. The node's `this` is bound to `node`.
    pub fn bind(&self, node: &Handle<dyn DeConfigurationNode>) -> bool {
        let Some(object) = node.get() else {
            return false;
        };
        object.configuration_node_base().bind_self(node);
        let format: AsciiString = object.get_format();
        let vendor: AsciiString = object.get_vendor();
        let mut configuration = self.configuration.write();
        if configuration.seek(&format).is_none() {
            configuration.bind(format.clone(), DeVendorMap::default());
        }
        let Some(vendors) = configuration.change_seek(&format) else {
            return false;
        };
        vendors.add(vendor, node.clone()) > 0
    }

    /// `UnBind(theNode)`: removes the node of the same format and vendor (the last vendor
    /// moves into its place); `true` when one was removed.
    pub fn unbind(&self, node: &Handle<dyn DeConfigurationNode>) -> bool {
        let Some(object) = node.get() else {
            return false;
        };
        let format: AsciiString = object.get_format();
        let vendor: AsciiString = object.get_vendor();
        let mut configuration = self.configuration.write();
        let Some(vendors) = configuration.change_seek(&format) else {
            return false;
        };
        let previous: usize = vendors.extent();
        vendors.remove_key(&vendor);
        vendors.extent() != previous
    }

    /// `Find(theFormat, theVendor, theNode)`: copies the node into `node` when bound; `node`
    /// is unchanged otherwise; `wrapper.find(&"STEP".into(), &"OCC".into(), &mut node)`.
    pub fn find(
        &self,
        format: &AsciiString,
        vendor: &AsciiString,
        node: &mut Handle<dyn DeConfigurationNode>,
    ) -> bool {
        let configuration = self.configuration.read();
        let Some(found) = configuration
            .seek(format)
            .and_then(|vendors| vendors.seek(vendor))
        else {
            return false;
        };
        *node = found.clone();
        true
    }

    /// `ChangePriority(theFormat, theVendorPriority, theToDisable)`: the listed vendors come
    /// first, in list order, and are enabled; the others follow in their order and are
    /// disabled when `to_disable`. Nothing happens for an unknown format.
    pub fn change_priority(
        &self,
        format: &AsciiString,
        vendor_priority: &List<AsciiString>,
        to_disable: bool,
    ) {
        let Some(vendors) = self.configuration.read().seek(format).cloned() else {
            return;
        };
        let mut prioritized: DeVendorMap = DeVendorMap::default();
        for vendor in vendor_priority.iter() {
            if let Some(node) = vendors.seek(vendor) {
                set_enabled(node, true);
                prioritized.add(vendor.clone(), node.clone());
            }
        }
        for (vendor, node) in vendors.items() {
            if vendor_priority.contains(vendor) {
                continue;
            }
            if to_disable {
                set_enabled(node, false);
            }
            prioritized.add(vendor.clone(), node.clone());
        }
        self.configuration.write().bind(format.clone(), prioritized);
    }

    /// `ChangePriority(theVendorPriority, theToDisable)`: the same priority for every format.
    pub fn change_priority_all(&self, vendor_priority: &List<AsciiString>, to_disable: bool) {
        for format in self.format_keys() {
            self.change_priority(&format, vendor_priority, to_disable);
        }
    }

    /// `UpdateLoad(theToForceUpdate)`: updates every node for import and export (keeping the
    /// result); with `to_force_update` a node is enabled exactly when it supports a direction.
    pub fn update_load(&self, to_force_update: bool) {
        for node in self.node_snapshot() {
            let Some(object) = node.get() else {
                continue;
            };
            object.update_load(true, true);
            object.update_load(false, true);
            if to_force_update {
                object.set_enabled(object.is_export_supported() || object.is_import_supported());
            }
        }
    }

    /// `KeepUpdates()`; `wrapper.keep_updates()`.
    pub fn keep_updates(&self) -> bool {
        self.settings.read().keep_updates
    }
    /// `SetKeepUpdates(theToKeepUpdates)`; `wrapper.set_keep_updates(true)`.
    pub fn set_keep_updates(&self, to_keep_updates: bool) {
        self.settings.write().keep_updates = to_keep_updates;
    }
    /// The public `GlobalParameters` field; `wrapper.global_parameters().length_unit`.
    pub fn global_parameters(&self) -> DeSectionGlobal {
        self.settings.read().global_parameters
    }
    /// Assigns the public `GlobalParameters` field; `wrapper.set_global_parameters(units)`.
    pub fn set_global_parameters(&self, parameters: DeSectionGlobal) {
        self.settings.write().global_parameters = parameters;
    }

    /// `Nodes()`: the format map, read-locked for the guard's lifetime;
    /// `wrapper.nodes().extent()`.
    pub fn nodes(&self) -> RwLockReadGuard<'_, DeFormatMap> {
        self.configuration.read()
    }

    /// The format names in map order.
    pub(super) fn format_keys(&self) -> Vec<AsciiString> {
        self.configuration
            .read()
            .items()
            .map(|(format, _)| format.clone())
            .collect()
    }

    /// Every node in map order (formats, then vendors by priority), taken under the lock and
    /// used after it is released (de-design.md section 4).
    pub(super) fn node_snapshot(&self) -> Vec<Handle<dyn DeConfigurationNode>> {
        let configuration = self.configuration.read();
        configuration
            .items()
            .flat_map(|(_, vendors)| vendors.items().map(|(_, node)| node.clone()))
            .collect()
    }
}

/// `aNode->SetEnabled(...)` on a stored node; a null node is skipped.
fn set_enabled(node: &Handle<dyn DeConfigurationNode>, is_enabled: bool) {
    if let Some(object) = node.get() {
        object.set_enabled(is_enabled);
    }
}
