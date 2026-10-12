//! Actual BRep vertex destruction observes still-live inherited drawer ownership.
use super::{AisColoredShape, AisShapeMembers};
use crate::{prs_mgr::PrsMgrPresentableObjectMembers, prs3d::Prs3dDrawer};
use rustycad_tkbrep::{
    brep::BRepTVertex,
    topods::{Shape, TShape, TShapeBase},
};
use rustycad_tkernel::standard::{Failure, Handle};
use std::{
    fmt::Write,
    sync::{Arc, Mutex},
};
struct ColoredParentDropVertexFake {
    retained_drawer: Handle<Prs3dDrawer>,
    vertex: BRepTVertex,
    rows: Arc<Mutex<String>>,
}
rustycad_tkernel::impl_transient!(ColoredParentDropVertexFake,"MartensiteColoredParentDropVertex",BRepTVertex,vertex,
    interfaces:[dyn TShape]);
impl TShape for ColoredParentDropVertexFake {
    fn tshape_base(&self) -> &TShapeBase {
        // The probe owns actual BRep geometry and topology flags rather than fabricating them.
        self.vertex.tshape_base()
    }
    fn empty_copy(&self) -> Result<Handle<dyn TShape>, Failure> {
        // Preserve the genuine BRep vertex's inherited geometry-copy semantics.
        self.vertex.empty_copy()
    }
}
impl Drop for ColoredParentDropVertexFake {
    fn drop(&mut self) {
        let count = self.retained_drawer.ref_count();
        writeln!(self.rows.lock().unwrap(), "parent-release {count}").unwrap();
    }
}
pub(super) fn record_parent_destruction(rows: &Arc<Mutex<String>>) {
    let object = AisColoredShape::new(&Shape::default());
    let topology = parent_drop_vertex(&object.attributes(), rows);
    let mut key = Shape::default();
    key.set_tshape(&topology.cast());
    object.set_shape(&key);
    drop(topology);
    drop(key);
    drop(object);
}
fn parent_drop_vertex(
    drawer: &Handle<Prs3dDrawer>,
    rows: &Arc<Mutex<String>>,
) -> Handle<ColoredParentDropVertexFake> {
    Handle::new(ColoredParentDropVertexFake {
        vertex: BRepTVertex::new(),
        retained_drawer: drawer.clone(),
        rows: rows.clone(),
    })
}
#[test]
fn named_parent_vertex_probe_keeps_inherited_real_geometry() {
    let rows = Arc::new(Mutex::new(String::new()));
    let drawer = Handle::new(Prs3dDrawer::new());
    let topology = parent_drop_vertex(&drawer, &rows);
    let copied = topology.empty_copy().unwrap();
    assert_eq!(topology.tshape_base().state(), copied.tshape_base().state());
    drop(topology);
    assert_eq!(*rows.lock().unwrap(), "parent-release 2\n");
}
