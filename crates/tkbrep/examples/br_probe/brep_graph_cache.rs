//! BRepGraph lock, derived-state and mesh caches oracle, part 1, the Rust twin of
//! tools/oracle/tkbrep_probe_brepgraph_cache.cxx: the probe driver, the state and the
//! lock, deferred and derived-state actions of `br_bcache`.
use super::brep_graph_layers::{label, read_item, read_node, read_ref};
use super::brep_graph_layers_actions::representations;
use super::brep_graph_views::guid;
use super::common::{Reader, flag, long, word};
use rustycad_tkbrep::brep_graph::cache_derived_state::CacheDerivedState;
use rustycad_tkbrep::brep_graph::cache_mesh::{
    CacheMesh, CacheMeshDriver, DEFAULT_DISPLAY_SLOT, DirtySet, SlotId,
};
use rustycad_tkbrep::brep_graph::layer::BRepGraphLayer;
use rustycad_tkbrep::brep_graph::layer_deferred::LayerDeferred;
use rustycad_tkbrep::brep_graph::layer_lock::{LayerLock, ScopedOwnerEdit};
use rustycad_tkbrep::brep_graph::tool;
use rustycad_tkbrep::brep_graph::{BRepGraph, CoEdgeId, EdgeId, ShellId, WireId};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Guid, Handle, StandardTransient};
use rustycad_tkmath::poly::{Polygon3D, Triangulation};
use std::sync::Mutex;

/// `bc_guid`: owner GUID i is f0000000-0000-0000-0000-00000000000i.
pub fn read_guid(r: &mut Reader<'_>) -> Guid {
    let text = format!("f0000000-0000-0000-0000-{:012}", r.integer());
    Guid::from_ascii(text.as_bytes()).unwrap()
}

/// `BcDriver`: logs every Fill and fills the dirty faces and free edges.
pub struct ProbeDriver {
    transient: StandardTransient,
    id: Guid,
    recipe: u64,
    triangulation: Handle<Triangulation>,
    polygon: Handle<Polygon3D>,
    result: bool,
    pub events: Mutex<Vec<i64>>,
}
impl_transient!(ProbeDriver, "BcDriver", dyn CacheMeshDriver, transient,
    interfaces: [dyn CacheMeshDriver]);

impl ProbeDriver {
    pub fn new(
        index: i32,
        recipe: u64,
        triangulation: Handle<Triangulation>,
        polygon: Handle<Polygon3D>,
        result: bool,
    ) -> Self {
        let id = format!("d0000000-0000-0000-0000-{index:012}");
        Self {
            transient: StandardTransient::default(),
            id: Guid::from_ascii(id.as_bytes()).unwrap(),
            recipe,
            triangulation,
            polygon,
            result,
            events: Mutex::new(Vec::new()),
        }
    }

    fn log(&self, slot: SlotId, dirty: &DirtySet) {
        let mut events = self.events.lock().unwrap();
        events.push(i64::from(slot));
        events.push(dirty.faces.len() as i64);
        events.extend(dirty.faces.iter().map(|f| i64::from(f.index)));
        events.push(dirty.free_edges.len() as i64);
        events.extend(dirty.free_edges.iter().map(|e| i64::from(e.index)));
    }
}

impl CacheMeshDriver for ProbeDriver {
    fn id(&self) -> Guid {
        self.id
    }
    fn recipe_hash(&self) -> u64 {
        self.recipe
    }
    fn fill(
        &self,
        g: &mut BRepGraph,
        slot: SlotId,
        dirty: &DirtySet,
        _range: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        self.log(slot, dirty);
        let cache = g.cache_registry().find::<CacheMesh>();
        let c = cache.get().unwrap();
        for &f in &dirty.faces {
            if slot == DEFAULT_DISPLAY_SLOT {
                g.mesh()
                    .editor()
                    .faces()
                    .set_cached_triangulation(f, &self.triangulation)?;
                continue;
            }
            let tri = self.triangulation.clone();
            let mut e = c.change_face_mesh_in(slot, f, |e| {
                e.triangulation = tri;
                e.clone()
            })?;
            c.bind_fresh_face(g, &mut e, f);
            c.change_face_mesh_in(slot, f, |stored| *stored = e)?;
        }
        for &x in &dirty.free_edges {
            if slot == DEFAULT_DISPLAY_SLOT {
                g.mesh()
                    .editor()
                    .edges()
                    .set_cached_polygon3d(x, &self.polygon)?;
                continue;
            }
            let poly = self.polygon.clone();
            let mut e = c.change_edge_mesh_in(slot, x, |e| {
                e.polygon3d = poly;
                e.clone()
            })?;
            c.bind_fresh_edge(g, &mut e, x);
            c.change_edge_mesh_in(slot, x, |stored| *stored = e)?;
        }
        Ok(self.result)
    }
}

/// `BcState`: the drivers of a request.
#[derive(Default)]
pub struct State {
    pub drivers: Vec<Handle<ProbeDriver>>,
}

fn lock(g: &BRepGraph) -> Result<Handle<LayerLock>, Failure> {
    g.layer_registry().ensure::<LayerLock>()
}

/// 61-73: the lock layer (`bc_lock_action`).
pub fn lock_action(g: &BRepGraph, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    if !(61..=73).contains(&code) {
        return Ok(false);
    }
    let handle = lock(g)?;
    let l = handle.get().unwrap();
    match code {
        61 => {
            let (i, o) = (read_item(r), read_guid(r));
            flag(l.set_owner_with_revision(g, i, &o, r.flag())?);
        }
        62 => {
            let i = read_item(r);
            l.set_owner(g, i, &read_guid(r))?;
        }
        63 => l.unset_owner(g, read_item(r))?,
        64 => {
            let i = read_item(r);
            l.unset_owner_matching(g, i, &read_guid(r))?;
        }
        65 => l.reserve_owners(r.integer() as usize),
        66 => l.touch_owners(),
        _ => dispatch_or_scope(g, l, r, code)?,
    }
    long(l.revision());
    Ok(true)
}

// 67-73 of `bc_lock_action`.
fn dispatch_or_scope(
    g: &BRepGraph,
    l: &LayerLock,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<(), Failure> {
    let reg = g.layer_registry();
    match code {
        67 => reg.dispatch_on_node_removed(Some(g), read_node(r)),
        68 => {
            let old = read_node(r);
            reg.dispatch_on_node_replaced(Some(g), old, read_node(r));
        }
        69 => reg.dispatch_on_ref_removed(Some(g), read_ref(r)),
        70 => reg.invalidate_all(Some(g)),
        71 => l.clear(Some(g)),
        72 => {
            let (i, o) = (read_item(r), read_guid(r));
            {
                let _scope = ScopedOwnerEdit::new(l, g, i, &o)?;
                flag(l.has_owner(g, i));
            }
            flag(l.has_owner(g, i));
        }
        _ => detached_lock(g, r)?,
    }
    Ok(())
}

// 73: a detached lock layer.
fn detached_lock(g: &BRepGraph, r: &mut Reader<'_>) -> Result<(), Failure> {
    let d = LayerLock::new();
    let (i, o) = (read_item(r), read_guid(r));
    flag(d.set_owner_with_revision(g, i, &o, true)?);
    let found = d.find_owner_id(g, i)?;
    flag(found.is_some());
    guid(found.unwrap_or_default());
    flag(d.has_owner(g, i));
    long(d.revision());
    Ok(())
}

/// 74-79: the attached deferred layer (`bc_deferred_action`).
pub fn deferred_action(g: &BRepGraph, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    if !(74..=79).contains(&code) {
        return Ok(false);
    }
    let handle = g.layer_registry().ensure::<LayerDeferred>()?;
    let d = handle.get().unwrap();
    match code {
        74 | 75 => {
            let i = read_item(r);
            let (p, k) = (label(r), label(r));
            let v = representations(r);
            if code == 74 {
                d.register_deferred_representations(g, i, &p, &k, &v)?;
            } else {
                d.register_deferred_representations_direct(g, i, &p, &k, &v)?;
            }
        }
        76 => d.unregister_deferred(g, read_item(r)),
        77 => d.begin_bulk_registration(),
        78 => d.end_bulk_registration(g)?,
        _ => d.reserve_deferred_items(g, r.integer() as usize)?,
    }
    long(d.revision());
    Ok(true)
}

/// 80-89: the derived-state cache (`bc_derived_action`).
pub fn derived_action(g: &BRepGraph, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    if !(80..=89).contains(&code) {
        return Ok(false);
    }
    let handle = g.cache_registry().ensure::<CacheDerivedState>()?;
    let c = handle.get().unwrap();
    let index = |r: &mut Reader<'_>| r.integer() as u32;
    match code {
        80 => {
            let e = EdgeId::new(index(r));
            flag(c.is_degenerated(g, e)?);
            flag(c.is_closed(g, e)?);
        }
        81 => flag(c.same_range(g, CoEdgeId::new(index(r)))?),
        82 => flag(c.same_parameter(g, CoEdgeId::new(index(r)))?),
        83 => {
            let closed = c.get_wire_is_closed(g, WireId::new(index(r)))?;
            flag(closed.is_some());
            flag(closed.unwrap_or(false));
        }
        84 => {
            let w = WireId::new(index(r));
            c.set_wire_is_closed(g, w, r.flag());
        }
        85 => flag(c.is_shell_closed(g, ShellId::new(index(r)))?),
        86 => rustycad_tkbrep::brep_graph::cache::BRepGraphCache::clear(c),
        87 => flag(tool::wire::is_closed(g, WireId::new(index(r)))),
        88 => flag(tool::shell::is_closed(g, ShellId::new(index(r)))),
        _ => {
            let e = EdgeId::new(index(r));
            flag(tool::edge::degenerated(g, e));
            flag(tool::edge::is_closed(g, e));
        }
    }
    word("-");
    Ok(true)
}
