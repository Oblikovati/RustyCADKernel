//! Named actual drawer subclass observes native map/Value state from its own destructor.
use super::{AisColoredDrawer, AisColoredDrawerCompounds};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkernel::standard::Handle;
use std::{
    fmt::Write,
    sync::{Arc, Mutex, Weak},
};
pub(super) struct ColoredCompoundKeyDropFake {
    parent: AisColoredDrawer,
    map: Weak<AisColoredDrawerCompounds>,
    rows: Arc<Mutex<String>>,
}
rustycad_tkernel::impl_transient!(
    ColoredCompoundKeyDropFake,
    "CompoundKeyDestructorProbe",
    AisColoredDrawer,
    parent
);
impl ColoredCompoundKeyDropFake {
    pub(super) fn new(
        parent: &Handle<Prs3dDrawer>,
        map: &Arc<AisColoredDrawerCompounds>,
        rows: &Arc<Mutex<String>>,
    ) -> Handle<Self> {
        Handle::new(Self {
            parent: AisColoredDrawer::new(parent),
            map: Arc::downgrade(map),
            rows: Arc::clone(rows),
        })
    }
}
impl Drop for ColoredCompoundKeyDropFake {
    fn drop(&mut self) {
        let map = self
            .map
            .upgrade()
            .expect("drop map=expired; expected live caller-owned map");
        let value_count = map.node_ref(1).unwrap().compound_ref_count().unwrap();
        let parent_count = self.parent.link_ref().observe().ref_count();
        writeln!(
            self.rows.lock().unwrap(),
            "key-destructor {} {value_count} {parent_count}",
            map.len()
        )
        .unwrap();
    }
}
