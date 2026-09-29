//! BRepGraph layers and caches oracle, part 1, the Rust twin of
//! tools/oracle/tkbrep_probe_brepgraph_layers.cxx: the readers, the logging layer, the
//! probe cache service and the per-request state.
use super::brep_graph::{node, reference};
use super::common::{Reader, flag, int, long};
use rustycad_tkbrep::brep_graph::cache::{
    BRepGraphCache, CacheBase, CacheType, ItemEntry, NodeEntry, RefEntry,
};
use rustycad_tkbrep::brep_graph::layer::{AttachedGraph, BRepGraphLayer, LayerBase};
use rustycad_tkbrep::brep_graph::layer_deferred::LayerDeferred;
use rustycad_tkbrep::brep_graph::{
    BRepGraph, ItemId, ItemUid, NodeId, NodeKind, RefId, RefKind, Uid,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Guid, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::Mutex;

/// `bl_node`, `bl_ref`, `bl_item`: `kind index` (items `domain kind index`).
pub fn read_node(r: &mut Reader<'_>) -> NodeId {
    let kind = r.integer();
    NodeId::new(NodeKind(kind), r.integer() as u32)
}
pub fn read_ref(r: &mut Reader<'_>) -> RefId {
    let kind = r.integer();
    RefId::new(RefKind(kind), r.integer() as u32)
}
pub fn read_item(r: &mut Reader<'_>) -> ItemId {
    if r.integer() == 0 {
        read_node(r).into()
    } else {
        read_ref(r).into()
    }
}
/// `bl_uid`, `bl_item_uid`: `kind counter`, `domain kind counter`.
pub fn read_uid(r: &mut Reader<'_>) -> Uid {
    let kind = r.integer();
    Uid::new(NodeKind(kind), r.integer() as u32)
}
pub fn read_item_uid(r: &mut Reader<'_>) -> ItemUid {
    let (domain, kind, counter) = (r.integer(), r.integer(), r.integer() as usize);
    if domain == 0 {
        ItemUid::node(NodeKind(kind), counter)
    } else {
        ItemUid::reference(RefKind(kind), counter)
    }
}
/// `bl_list`: a count, then the values.
pub fn read_list<T>(r: &mut Reader<'_>, read: impl Fn(&mut Reader<'_>) -> T) -> Vec<T> {
    let count = r.integer();
    (0..count).map(|_| read(r)).collect()
}
/// `bl_label`: a text given as bytes.
pub fn label(r: &mut Reader<'_>) -> AsciiString {
    AsciiString::from_bytes(r.bytes())
}
pub fn print_uid(u: &Uid) {
    int(u.kind.0);
    long(u64::from(u.counter));
}
pub fn print_item_uid(u: &ItemUid) {
    int(u.item_domain() as i32);
    int(i32::from(u.raw_kind()));
    long(u.counter() as u64);
}
pub fn nodes(v: &[NodeId]) {
    long(v.len() as u64);
    v.iter().copied().for_each(node);
}
pub fn uids(v: &[Uid]) {
    long(v.len() as u64);
    v.iter().for_each(print_uid);
}
pub fn item_uids(v: &[ItemUid]) {
    long(v.len() as u64);
    v.iter().for_each(print_item_uid);
}
pub fn item(i: ItemId) {
    int(i.item_domain() as i32);
    int(i32::from(i.raw_kind()));
    long(u64::from(i.index()));
}

/// `BlEventLayer`: logs its callbacks as codes with its attachment state.
pub struct EventLayer {
    transient: StandardTransient,
    base: LayerBase,
    id: Guid,
    name: AsciiString,
    kinds: i32,
    ref_kinds: i32,
    pub events: Mutex<Vec<i64>>,
}
impl_transient!(EventLayer, "BlEventLayer", dyn BRepGraphLayer, transient,
    interfaces: [dyn BRepGraphLayer]);

impl EventLayer {
    pub fn new(index: i32, kinds: i32, ref_kinds: i32) -> Self {
        let id = format!("e0000000-0000-0000-0000-{index:012}");
        let mut name = AsciiString::new("Events");
        name.append(&AsciiString::new(&index.to_string()));
        Self {
            transient: StandardTransient::default(),
            base: LayerBase::new(),
            id: Guid::from_ascii(id.as_bytes()).unwrap(),
            name,
            kinds,
            ref_kinds,
            events: Mutex::new(Vec::new()),
        }
    }
    fn push(&self, code: i64) {
        let attached = i64::from(self.base.is_attached());
        self.events.lock().unwrap().extend([code, attached]);
    }
    fn node(&self, n: NodeId) {
        self.events
            .lock()
            .unwrap()
            .extend([i64::from(n.kind.0), i64::from(n.index)]);
    }
    fn reference(&self, x: RefId) {
        self.events
            .lock()
            .unwrap()
            .extend([i64::from(x.kind.0), i64::from(x.index)]);
    }
}

impl BRepGraphLayer for EventLayer {
    fn layer_base(&self) -> &LayerBase {
        &self.base
    }
    fn id(&self) -> Guid {
        self.id
    }
    fn name(&self) -> &AsciiString {
        &self.name
    }
    fn subscribed_kinds(&self) -> i32 {
        self.kinds
    }
    fn subscribed_ref_kinds(&self) -> i32 {
        self.ref_kinds
    }
    fn on_node_removed(&self, _: AttachedGraph<'_>, n: NodeId) {
        self.push(1);
        self.node(n);
    }
    fn on_node_replaced(&self, _: AttachedGraph<'_>, old: NodeId, new: NodeId) {
        self.push(2);
        self.node(old);
        self.node(new);
    }
    fn on_node_modified(&self, _: AttachedGraph<'_>, n: NodeId) {
        self.push(3);
        self.node(n);
    }
    fn on_nodes_modified(&self, _: AttachedGraph<'_>, v: &[NodeId]) {
        self.push(4);
        self.events.lock().unwrap().push(v.len() as i64);
        v.iter().for_each(|n| self.node(*n));
    }
    fn on_ref_removed(&self, _: AttachedGraph<'_>, x: RefId) {
        self.push(5);
        self.reference(x);
    }
    fn on_ref_modified(&self, _: AttachedGraph<'_>, x: RefId) {
        self.push(6);
        self.reference(x);
    }
    fn on_refs_modified(&self, _: AttachedGraph<'_>, v: &[RefId]) {
        self.push(7);
        self.events.lock().unwrap().push(v.len() as i64);
        v.iter().for_each(|x| self.reference(*x));
    }
    fn on_attached(&self) {
        self.push(8);
    }
    fn on_detached(&self) {
        self.push(9);
    }
    fn invalidate_all(&self, _: AttachedGraph<'_>) {
        self.push(10);
    }
    fn clear(&self, _: AttachedGraph<'_>) {
        self.push(11);
    }
}

/// The bound entries of `BlCache`.
#[derive(Default)]
pub struct CacheEntries {
    pub nodes: Vec<NodeEntry>,
    pub refs: Vec<RefEntry>,
    pub items: Vec<ItemEntry>,
}

/// `BlCache`: counts its clears and keeps bound entries.
#[derive(Default)]
pub struct ProbeCache {
    transient: StandardTransient,
    pub base: CacheBase,
    pub clears: Mutex<i64>,
    pub entries: Mutex<CacheEntries>,
}
impl_transient!(ProbeCache, "BlCache", dyn BRepGraphCache, transient,
    interfaces: [dyn BRepGraphCache]);

impl CacheType for ProbeCache {
    fn get_id() -> Guid {
        Guid::from_ascii(b"c0000000-0000-0000-0000-000000000001").unwrap()
    }
}

impl BRepGraphCache for ProbeCache {
    fn cache_base(&self) -> &CacheBase {
        &self.base
    }
    fn id(&self) -> Guid {
        Self::get_id()
    }
    fn name(&self) -> &AsciiString {
        static NAME: std::sync::OnceLock<AsciiString> = std::sync::OnceLock::new();
        NAME.get_or_init(|| AsciiString::new("ProbeCache"))
    }
    fn clear(&self) {
        *self.clears.lock().unwrap() += 1;
    }
}

impl ProbeCache {
    /// `Bind(type, reader)`: 0 node OwnGen, 1 node SubtreeGen, 2 reference, 3 item.
    pub fn bind(&self, g: &BRepGraph, kind: i32, r: &mut Reader<'_>) -> bool {
        let mut entries = self.entries.lock().unwrap();
        match kind {
            2 => {
                let mut e = RefEntry::default();
                let ok = e.bind_own_gen(&self.base, g, read_ref(r));
                entries.refs.push(e);
                ok
            }
            3 => {
                let mut e = ItemEntry::default();
                let ok = e.bind_own_gen(&self.base, g, read_item(r));
                entries.items.push(e);
                ok
            }
            _ => {
                let (mut e, n) = (NodeEntry::default(), read_node(r));
                let ok = if kind == 0 {
                    e.bind_own_gen(&self.base, g, n)
                } else {
                    e.bind_subtree_gen(&self.base, g, n)
                };
                entries.nodes.push(e);
                ok
            }
        }
    }

    /// `Report(g)`: the clears, the entries and the generations of every node and
    /// reference (one past each kind's end too).
    pub fn report(&self, g: &BRepGraph) {
        long(*self.clears.lock().unwrap() as u64);
        flag(self.base.is_attached());
        self.report_entries(g);
        for k in 0..12 {
            let kind = NodeKind(k);
            if !NodeKind::is_valid_kind(kind) {
                continue;
            }
            for i in 0..=g.topo().generic().nb(kind) {
                let n = NodeId::new(kind, i);
                generation(self.base.node_own_gen(g, n));
                generation(self.base.node_subtree_gen(g, n));
            }
        }
        for k in 0..7 {
            let kind = RefKind(k);
            for i in 0..g.refs().generic().nb(kind) {
                let x = RefId::new(kind, i);
                generation(self.base.ref_own_gen(g, x));
                flag(self.base.item_own_gen(g, x.into()).is_some());
                let child = self.base.resolve_active_ref_child(g, x);
                flag(child.is_some());
                node(child.unwrap_or_default());
            }
        }
    }

    fn report_entries(&self, g: &BRepGraph) {
        let entries = self.entries.lock().unwrap();
        long(entries.nodes.len() as u64);
        for e in &entries.nodes {
            flag(e.is_bound());
            flag(e.is_fresh(&self.base, g));
            node(e.node());
        }
        long(entries.refs.len() as u64);
        for e in &entries.refs {
            flag(e.is_bound());
            flag(e.is_fresh(&self.base, g));
            reference(e.reference());
        }
        long(entries.items.len() as u64);
        for e in &entries.items {
            flag(e.is_bound());
            flag(e.is_fresh(&self.base, g));
            item(e.item());
        }
    }
}

// A generation read: the flag, then the value (0 when absent, the native default).
fn generation(value: Option<u32>) {
    flag(value.is_some());
    long(u64::from(value.unwrap_or(0)));
}

/// `BlState`: the logging layers, the probe caches (null when `Ensure` found another
/// class) and the standalone deferred layer.
pub struct State {
    pub events: Vec<Handle<EventLayer>>,
    pub caches: Vec<Handle<ProbeCache>>,
    pub deferred: Handle<LayerDeferred>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            caches: Vec::new(),
            deferred: Handle::new(LayerDeferred::new()),
        }
    }
}
