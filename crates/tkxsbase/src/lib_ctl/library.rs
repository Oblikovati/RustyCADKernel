// Rust port of DataExchange/TKXSBase/LibCtl/LibCtl_Library.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LibCtl_Library`: a library built from a protocol selects the global nodes of that
//! protocol and of its resources, then dispatches objects to their modules.
use super::{LibCtlGlobalNode, LibCtlNode};
use crate::interface::InterfaceProtocol;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, StandardTransient, exceptions};
use std::sync::{Arc, PoisonError, RwLock};

/// What a library needs from its protocol type: the native `TheProtocol` interface.
pub trait LibCtlProtocol {
    /// `NbResources`.
    fn lib_nb_resources(&self) -> i32;
    /// `Resource(num)`, as a transient for the down cast to the library's protocol type.
    fn lib_resource(&self, num: i32) -> Handle<StandardTransient>;
    /// `CaseNumber(obj)`.
    fn lib_case_number(&self, obj: &Handle<StandardTransient>) -> i32;
}

impl<P: ?Sized + InterfaceProtocol> LibCtlProtocol for P {
    fn lib_nb_resources(&self) -> i32 {
        self.nb_resources()
    }
    fn lib_resource(&self, num: i32) -> Handle<StandardTransient> {
        self.resource(num).cast()
    }
    fn lib_case_number(&self, obj: &Handle<StandardTransient>) -> i32 {
        self.case_number(obj)
    }
}

/// One instance of the template: its module and protocol types and its process-wide
/// registry (the native static list of global nodes).
pub trait LibCtlKind: Sized + 'static {
    /// `TheModule` (`Interface_GeneralModule` for `Interface_GeneralLib`...).
    type Module: ?Sized + RuntimeTyped;
    /// `TheProtocol` (`Interface_Protocol` for the Interface libraries...).
    type Protocol: ?Sized + RuntimeTyped + LibCtlProtocol;
    /// The process-wide registry of this library.
    fn registry() -> &'static LibCtlRegistry<Self>;
}

/// The global list of (module, protocol) pairs of a library (the native `THE_GLOBAL`
/// with its writer lock).
pub struct LibCtlRegistry<K: LibCtlKind> {
    global: RwLock<Option<Arc<LibCtlGlobalNode<K>>>>,
}

impl<K: LibCtlKind> Default for LibCtlRegistry<K> {
    fn default() -> Self {
        Self {
            global: RwLock::new(None),
        }
    }
}

impl<K: LibCtlKind> LibCtlRegistry<K> {
    /// An empty registry; `LibCtlRegistry::new()` for a private one.
    pub fn new() -> Self {
        Self::default()
    }
    /// `SetGlobal(module, protocol)` on this registry.
    pub fn set_global(&self, module: &Handle<K::Module>, protocol: &Handle<K::Protocol>) {
        let head = {
            let mut global = self.global.write().unwrap_or_else(PoisonError::into_inner);
            global
                .get_or_insert_with(|| Arc::new(LibCtlGlobalNode::new()))
                .clone()
        };
        head.add(module, protocol);
    }
    /// The head of the global list.
    pub fn head(&self) -> Option<Arc<LibCtlGlobalNode<K>>> {
        self.global
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

/// `LibCtl_Library`: the selected nodes and the cursor of the iteration. Copies share
/// the nodes, as natively.
pub struct LibCtlLibrary<K: LibCtlKind> {
    thelist: Option<Arc<LibCtlNode<K>>>,
    thecurr: Option<Arc<LibCtlNode<K>>>,
}

impl<K: LibCtlKind> Clone for LibCtlLibrary<K> {
    fn clone(&self) -> Self {
        Self {
            thelist: self.thelist.clone(),
            thecurr: self.thecurr.clone(),
        }
    }
}

impl<K: LibCtlKind> Default for LibCtlLibrary<K> {
    fn default() -> Self {
        Self {
            thelist: None,
            thecurr: None,
        }
    }
}

impl<K: LibCtlKind> LibCtlLibrary<K> {
    /// `SetGlobal(module, protocol)`: records a pair in the process-wide registry.
    pub fn set_global(module: &Handle<K::Module>, protocol: &Handle<K::Protocol>) {
        K::registry().set_global(module, protocol);
    }
    /// `LibCtl_Library(protocol)`: the nodes of `protocol` and of its resources, from the
    /// process-wide registry; empty for a null protocol.
    pub fn new(protocol: &Handle<K::Protocol>) -> Self {
        Self::new_in(K::registry(), protocol)
    }
    /// `LibCtl_Library(protocol)` over an explicit registry.
    pub fn new_in(registry: &LibCtlRegistry<K>, protocol: &Handle<K::Protocol>) -> Self {
        let mut library = Self::default();
        if protocol.is_some() {
            library.add_protocol_in(registry, &protocol.cast());
        }
        library
    }
    /// `LibCtl_Library()`: empty.
    pub fn empty() -> Self {
        Self::default()
    }
    /// `AddProtocol(protocol)`: adds the nodes of a protocol (ignored unless it is of the
    /// library's protocol type) and of its resources.
    pub fn add_protocol(&mut self, protocol: &Handle<StandardTransient>) {
        self.add_protocol_in(K::registry(), protocol);
    }
    /// `AddProtocol` over an explicit registry.
    pub fn add_protocol_in(
        &mut self,
        registry: &LibCtlRegistry<K>,
        protocol: &Handle<StandardTransient>,
    ) {
        add_protocol_impl(registry, &mut self.thelist, protocol);
    }
    /// `Clear`: an empty node list.
    pub fn clear(&mut self) {
        self.thelist = Some(Arc::new(LibCtlNode::new()));
    }
    /// `SetComplete`: every global node with a protocol, from the process-wide registry.
    pub fn set_complete(&mut self) {
        self.set_complete_in(K::registry());
    }
    /// `SetComplete` over an explicit registry.
    pub fn set_complete_in(&mut self, registry: &LibCtlRegistry<K>) {
        let list = Arc::new(LibCtlNode::new());
        let mut current = registry.head();
        while let Some(node) = current {
            if node.protocol().is_some() {
                list.add_node(&node);
            }
            current = node.next();
        }
        self.thelist = Some(list);
    }
    /// `Select(obj, module, CN)`: the module of the first protocol giving `obj` a
    /// positive case number, with that number; `None` (natively a null module and 0)
    /// when no protocol recognises it.
    pub fn select(&self, obj: &Handle<StandardTransient>) -> Option<(Handle<K::Module>, i32)> {
        let mut current = self.thelist.clone();
        while let Some(node) = current {
            let protocol = node.protocol();
            if let Some(live) = protocol.get() {
                let cn = live.lib_case_number(obj);
                if cn > 0 {
                    return Some((node.module(), cn));
                }
            }
            current = node.next();
        }
        None
    }
    /// `Start`.
    pub fn start(&mut self) {
        self.thecurr = self.thelist.clone();
    }
    /// `More`.
    pub fn more(&self) -> bool {
        self.thecurr.is_some()
    }
    /// `Next`.
    pub fn next(&mut self) {
        if let Some(current) = &self.thecurr {
            self.thecurr = current.next();
        }
    }
    /// `Module`: `Standard_NoSuchObject` after the end.
    pub fn module(&self) -> Result<Handle<K::Module>, Failure> {
        Ok(self.current()?.module())
    }
    /// `Protocol`: `Standard_NoSuchObject` after the end.
    pub fn protocol(&self) -> Result<Handle<K::Protocol>, Failure> {
        Ok(self.current()?.protocol())
    }
    fn current(&self) -> Result<&Arc<LibCtlNode<K>>, Failure> {
        self.thecurr.as_ref().ok_or_else(|| {
            exceptions::no_such_object(
                "Library from LibCtl: the iteration is over; expected More() to be true",
            )
        })
    }
}

/// `LibCtl_AddProtocolImpl`: the first global node whose protocol has the dynamic type of
/// `protocol`, then the resources, recursively.
fn add_protocol_impl<K: LibCtlKind>(
    registry: &LibCtlRegistry<K>,
    target: &mut Option<Arc<LibCtlNode<K>>>,
    protocol: &Handle<StandardTransient>,
) {
    let typed: Handle<K::Protocol> = protocol.cast();
    let Some(live) = typed.get() else { return };
    let kind = protocol.dynamic_type().map(|t| t as *const _);
    let mut current = registry.head();
    while let Some(node) = current {
        let node_kind = node.protocol().dynamic_type().map(|t| t as *const _);
        if node_kind.is_some() && node_kind == kind {
            target
                .get_or_insert_with(|| Arc::new(LibCtlNode::new()))
                .add_node(&node);
            break;
        }
        current = node.next();
    }
    for index in 1..=live.lib_nb_resources() {
        add_protocol_impl(registry, target, &live.lib_resource(index));
    }
}
