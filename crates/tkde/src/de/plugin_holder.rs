// Rust port of DataExchange/TKDE/DE/DE_PluginHolder.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DE_PluginHolder`, `DE_MultiPluginHolder` and `DEPLUGIN`: nodes registered in the global
//! wrapper for the lifetime of a holder (de-design.md section 3).
use super::{DeConfigurationNode, DeWrapper, live, new_configuration_node};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use std::sync::PoisonError;

/// The node classes a plugin holder can create: a default-constructible configuration node.
pub trait DePluginNode: DeConfigurationNode + RuntimeTyped + Default {}
impl<T: DeConfigurationNode + RuntimeTyped + Default> DePluginNode for T {}

/// `DE_PluginHolder<TheConfType>`: creates a node and registers it in the global wrapper
/// under the global load mutex; dropping the holder unregisters it the same way.
///
/// As natively, the drop unregisters by format and vendor, so it also removes another node
/// of the same format and vendor that was bound first (tools/oracle/tkde_configuration_probe.cxx,
/// `plugin.single.released`).
pub struct DePluginHolder<T: DePluginNode> {
    internal_configuration: Handle<T>,
}

impl<T: DePluginNode> DePluginHolder<T> {
    /// `DE_PluginHolder()`; `let holder = DePluginHolder::<MyNode>::new()?;`.
    pub fn new() -> Result<Self, Failure> {
        let _lock = DeWrapper::global_load_mutex()
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let internal_configuration: Handle<T> = new_configuration_node(T::default());
        live(&internal_configuration, "DE_PluginHolder")?.register(&DeWrapper::global_wrapper())?;
        Ok(Self {
            internal_configuration,
        })
    }
}

impl<T: DePluginNode> Drop for DePluginHolder<T> {
    fn drop(&mut self) {
        let _lock = DeWrapper::global_load_mutex()
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(node) = self.internal_configuration.get() {
            // The node was bound in `new`, so `this` exists; a destructor cannot report.
            let _ = node.unregister(&DeWrapper::global_wrapper());
        }
    }
}

/// The node lists of a [`DeMultiPluginHolder`]: tuples of one to four node classes.
pub trait DePluginList {
    /// The tuple of holders, one per node class.
    type Holders;
    /// Creates the holders the way `std::tuple` does in libstdc++: the last class first.
    fn hold() -> Result<Self::Holders, Failure>;
}

/// Implements [`DePluginList`] for a tuple; the second list is the construction order.
macro_rules! plugin_list {
    ([$($node:ident $holder:ident),+], [$($constructed:ident),+]) => {
        impl<$($node: DePluginNode),+> DePluginList for ($($node,)+) {
            type Holders = ($(DePluginHolder<$node>,)+);
            fn hold() -> Result<Self::Holders, Failure> {
                $(let $constructed = DePluginHolder::new()?;)+
                Ok(($($holder,)+))
            }
        }
    };
}
plugin_list!([A a], [a]);
plugin_list!([A a, B b], [b, a]);
plugin_list!([A a, B b, C c], [c, b, a]);
plugin_list!([A a, B b, C c, D d], [d, c, b, a]);

/// `DE_MultiPluginHolder<TheConfTypes...>`: one holder per class of the tuple `L`; the
/// holders are created last class first and dropped first class first, as the native
/// `std::tuple` member (tools/oracle/tkde_configuration_probe.cxx, `plugin.multi`).
pub struct DeMultiPluginHolder<L: DePluginList> {
    holders: L::Holders,
}

impl<L: DePluginList> DeMultiPluginHolder<L> {
    /// `DE_MultiPluginHolder()`; `DeMultiPluginHolder::<(StepNode, IgesNode)>::new()?`.
    pub fn new() -> Result<Self, Failure> {
        Ok(Self {
            holders: L::hold()?,
        })
    }
    /// The holders, in the order of the tuple; `multi.holders()`.
    pub fn holders(&self) -> &L::Holders {
        &self.holders
    }
}

/// The result of a plugin factory of [`deplugin!`](crate::deplugin).
pub type DePluginResult = Result<(), Failure>;

/// `DEPLUGIN(theNodeType, ...)`: defines `fn $factory()` that registers the node classes in
/// the global wrapper once and keeps them registered for the life of the process (the
/// native `static DE_MultiPluginHolder` of `PLUGINFACTORY`).
///
/// ```
/// # use rustycad_tkde::de::*;
/// # use rustycad_tkernel::{impl_transient, ncollection::List, standard::*, tcollection::AsciiString};
/// # #[derive(Default)]
/// # struct DocNode { transient: StandardTransient, base: DeConfigurationNodeBase }
/// # impl_transient!(DocNode, "DocNode", dyn DeConfigurationNode, transient, interfaces: [dyn DeConfigurationNode]);
/// # impl DeConfigurationNode for DocNode {
/// #     fn configuration_node_base(&self) -> &DeConfigurationNodeBase { &self.base }
/// #     fn load(&self, _: &Handle<DeConfigurationContext>) -> Result<bool, Failure> { Ok(true) }
/// #     fn save(&self) -> AsciiString { AsciiString::default() }
/// #     fn build_provider(&self) -> Handle<dyn DeProvider> { Handle::default() }
/// #     fn copy(&self) -> Handle<dyn DeConfigurationNode> { Handle::default() }
/// #     fn get_format(&self) -> AsciiString { AsciiString::new("DOC") }
/// #     fn get_vendor(&self) -> AsciiString { AsciiString::new("ME") }
/// #     fn get_extensions(&self) -> List<AsciiString> { List::default() }
/// # }
/// rustycad_tkde::deplugin!(plugin_factory: DocNode);
/// plugin_factory().unwrap();
/// let mut node = Handle::default();
/// assert!(DeWrapper::global_wrapper().get().unwrap().find(&"DOC".into(), &"ME".into(), &mut node));
/// ```
#[macro_export]
macro_rules! deplugin {
    ($factory:ident : $($node:ty),+ $(,)?) => {
        /// `PLUGINFACTORY`: registers the plugin's configuration nodes in the global wrapper.
        pub fn $factory() -> $crate::de::DePluginResult {
            static HOLDER: std::sync::Mutex<Option<$crate::de::DeMultiPluginHolder<($($node,)+)>>> =
                std::sync::Mutex::new(None);
            let mut holder = HOLDER.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if holder.is_none() {
                *holder = Some($crate::de::DeMultiPluginHolder::new()?);
            }
            Ok(())
        }
    };
}
